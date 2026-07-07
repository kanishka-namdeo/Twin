pub mod resource;
pub mod shutdown;

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio_util::sync::CancellationToken;

use resource::{CleanupPriority, CleanupFn, ResourceList, ResourceState, ResourceSummary, TrackedResource, new_resource_list};
use shutdown::{ShutdownResult, execute_ordered_shutdown, execute_force_cleanup, get_resource_summary};

/// Central lifecycle manager that tracks all active resources and
/// orchestrates ordered cleanup on shutdown.
///
/// Resources are registered with a priority tier (Critical, Secondary, Persistent)
/// and cleaned up in order: Critical first, then Secondary, then Persistent.
/// Within each tier, cleanup happens in reverse registration order (LIFO).
pub struct AppLifecycleManager {
    resources: ResourceList,
    cancellation_token: CancellationToken,
    shutdown_in_progress: Arc<TokioMutex<bool>>,
}

impl AppLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            resources: new_resource_list(),
            cancellation_token: CancellationToken::new(),
            shutdown_in_progress: Arc::new(TokioMutex::new(false)),
        }
    }

    /// Get a clone of the cancellation token for use by other components
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    /// Register a resource for lifecycle tracking.
    ///
    /// The `name` is used for logging and identification.
    /// The `priority` determines cleanup order.
    /// The `cleanup_fn` is called during shutdown to clean up the resource.
    pub async fn register(
        &self,
        name: impl Into<String>,
        priority: CleanupPriority,
        cleanup_fn: CleanupFn,
    ) {
        let name = name.into();
        let resource = TrackedResource::new(name.clone(), priority, cleanup_fn);

        let mut resources = self.resources.lock().await;
        // Check for duplicate registration
        if resources.iter().any(|r| r.name == name && r.state == ResourceState::Active) {
            log::warn!("Resource '{}' is already registered and active, replacing", name);
            resources.retain(|r| r.name != name || r.state != ResourceState::Active);
        }
        resources.push(resource);
        log::info!("Registered resource '{}' with {} priority", name, priority);
    }

    /// Unregister a resource (marks it as cleaned without running cleanup).
    /// Use this when a resource has already been cleaned up by its owner.
    pub async fn unregister(&self, name: &str) {
        let mut resources = self.resources.lock().await;
        for resource in resources.iter_mut() {
            if resource.name == name && resource.state == ResourceState::Active {
                resource.state = ResourceState::Cleaned;
                resource.cleaned_at = Some(std::time::Instant::now());
                resource.cleanup_fn = None;
                log::info!("Unregistered resource '{}'", name);
                return;
            }
        }
        log::debug!("Resource '{}' not found for unregistration", name);
    }

    /// Execute ordered shutdown of all registered resources.
    ///
    /// This is idempotent — calling it multiple times is safe.
    /// The first call performs cleanup; subsequent calls return immediately.
    pub async fn shutdown(&self) -> ShutdownResult {
        // Guard against concurrent shutdowns
        {
            let mut in_progress = self.shutdown_in_progress.lock().await;
            if *in_progress {
                log::warn!("Shutdown already in progress, skipping duplicate request");
                return ShutdownResult {
                    success: false,
                    cleaned: Vec::new(),
                    failed: vec![("shutdown".to_string(), "duplicate shutdown request".to_string())],
                    timed_out: Vec::new(),
                    elapsed: std::time::Duration::ZERO,
                };
            }
            *in_progress = true;
        }

        log::info!("=== Lifecycle shutdown initiated ===");

        // Execute ordered shutdown
        let mut result = execute_ordered_shutdown(&self.resources, &self.cancellation_token).await;

        // If any resources failed or timed out, attempt force cleanup
        if !result.failed.is_empty() || !result.timed_out.is_empty() {
            log::warn!("Some resources failed graceful cleanup, attempting force cleanup...");
            let force_cleaned = execute_force_cleanup(&self.resources).await;
            result.cleaned.extend(force_cleaned);
        }

        // Log final resource state summary
        let summary = get_resource_summary(&self.resources).await;
        log::info!("=== Shutdown resource summary ===");
        for res in &summary {
            log::info!("  [{}] {} ({})", res.name, res.state, res.priority);
        }

        let still_active = summary.iter().filter(|r| r.state == ResourceState::Active).count();
        if still_active > 0 {
            log::error!("{} resources still active after shutdown!", still_active);
        }

        log::info!("=== Lifecycle shutdown complete: {} ===", result.summary());
        result
    }

    /// Get a summary of all tracked resources
    pub async fn resource_summary(&self) -> Vec<ResourceSummary> {
        get_resource_summary(&self.resources).await
    }

    /// Check if shutdown is currently in progress
    pub async fn is_shutting_down(&self) -> bool {
        *self.shutdown_in_progress.lock().await
    }

    /// Get count of active resources
    pub async fn active_resource_count(&self) -> usize {
        let resources = self.resources.lock().await;
        resources.iter().filter(|r| r.state == ResourceState::Active).count()
    }
}

impl Default for AppLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AppLifecycleManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppLifecycleManager")
            .field("cancellation_token", &"<CancellationToken>")
            .finish()
    }
}
