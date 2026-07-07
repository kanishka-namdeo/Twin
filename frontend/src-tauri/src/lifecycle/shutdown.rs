use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use super::resource::{CleanupPriority, ResourceList, ResourceState, ResourceSummary};

/// Total timeout for the entire shutdown sequence
pub const TOTAL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Per-resource-type timeout for individual cleanup steps
pub const PER_RESOURCE_TIMEOUT: Duration = Duration::from_secs(5);

/// Result of a shutdown operation
#[derive(Debug)]
pub struct ShutdownResult {
    pub success: bool,
    pub cleaned: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub timed_out: Vec<String>,
    pub elapsed: Duration,
}

impl ShutdownResult {
    pub fn new() -> Self {
        Self {
            success: true,
            cleaned: Vec::new(),
            failed: Vec::new(),
            timed_out: Vec::new(),
            elapsed: Duration::ZERO,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Shutdown: {} cleaned, {} failed, {} timed out ({:?} total)",
            self.cleaned.len(),
            self.failed.len(),
            self.timed_out.len(),
            self.elapsed,
        )
    }
}

/// Execute ordered shutdown of all resources.
///
/// Cleanup order: Critical → Secondary → Persistent.
/// Within each priority tier, resources are cleaned up in reverse registration order (LIFO).
/// Each resource gets at most `PER_RESOURCE_TIMEOUT` for cleanup.
/// The entire shutdown is bounded by `TOTAL_SHUTDOWN_TIMEOUT`.
pub async fn execute_ordered_shutdown(
    resources: &ResourceList,
    cancellation_token: &CancellationToken,
) -> ShutdownResult {
    let start = Instant::now();
    let mut result = ShutdownResult::new();

    log::info!("Starting ordered shutdown sequence...");

    // Cancel all token-tracked operations first
    log::info!("Cancelling all tracked operations via CancellationToken...");
    cancellation_token.cancel();

    // Process each priority tier in order
    let priorities = [
        CleanupPriority::Critical,
        CleanupPriority::Secondary,
        CleanupPriority::Persistent,
    ];

    for priority in &priorities {
        // Check if we've exceeded total timeout
        if start.elapsed() >= TOTAL_SHUTDOWN_TIMEOUT {
            log::warn!(
                "Total shutdown timeout ({:?}) exceeded before completing {} priority tier",
                TOTAL_SHUTDOWN_TIMEOUT,
                priority
            );
            result.success = false;
            break;
        }

        log::info!("Cleaning up {} priority resources...", priority);

        // Collect resources for this priority tier (reverse order = LIFO)
        let tier_indices: Vec<usize> = {
            let res_lock = resources.lock().await;
            let mut tier: Vec<usize> = res_lock
                .iter()
                .enumerate()
                .filter(|(_, r)| r.priority == *priority && r.state == ResourceState::Active)
                .map(|(i, _)| i)
                .collect();
            tier.reverse();
            tier
        };

        for idx in tier_indices {
            if start.elapsed() >= TOTAL_SHUTDOWN_TIMEOUT {
                log::warn!("Total shutdown timeout exceeded during resource cleanup");
                result.success = false;
                break;
            }

            let (resource_name, cleanup_fn) = {
                let mut res_lock = resources.lock().await;
                let resource = &mut res_lock[idx];
                let resource_name = resource.name.clone();

                if resource.cleanup_fn.is_none() {
                    resource.state = ResourceState::Cleaned;
                    resource.cleaned_at = Some(Instant::now());
                    result.cleaned.push(resource_name.clone());
                    log::debug!("Resource '{}' already cleaned (no cleanup fn)", resource_name);
                    continue;
                }

                (resource_name, resource.cleanup_fn.take().unwrap())
            };

            log::info!("Cleaning up resource '{}' ({} priority)...", resource_name, priority);

            // Run cleanup with timeout
            let cleanup_future = cleanup_fn();
            match tokio::time::timeout(PER_RESOURCE_TIMEOUT, cleanup_future).await {
                Ok(Ok(())) => {
                    log::info!("Resource '{}' cleaned up successfully", resource_name);
                    let mut res_lock = resources.lock().await;
                    res_lock[idx].state = ResourceState::Cleaned;
                    res_lock[idx].cleaned_at = Some(Instant::now());
                    result.cleaned.push(resource_name);
                }
                Ok(Err(e)) => {
                    log::error!("Resource '{}' cleanup failed: {}", resource_name, e);
                    let mut res_lock = resources.lock().await;
                    res_lock[idx].state = ResourceState::CleanupFailed;
                    result.failed.push((resource_name.clone(), e));
                }
                Err(_) => {
                    log::error!(
                        "Resource '{}' cleanup timed out after {:?}",
                        resource_name,
                        PER_RESOURCE_TIMEOUT
                    );
                    let mut res_lock = resources.lock().await;
                    res_lock[idx].state = ResourceState::TimedOut;
                    result.timed_out.push(resource_name);
                }
            }
        }
    }

    result.elapsed = start.elapsed();

    if result.success && result.failed.is_empty() && result.timed_out.is_empty() {
        log::info!("All resources cleaned up successfully: {}", result.summary());
    } else {
        log::warn!("Shutdown completed with issues: {}", result.summary());
        result.success = false;
    }

    result
}

/// Force cleanup — attempt to clean up any remaining active resources
/// without timeout enforcement. Used as a fallback when graceful shutdown fails.
pub async fn execute_force_cleanup(resources: &ResourceList) -> Vec<String> {
    let mut force_cleaned = Vec::new();

    log::warn!("Executing force cleanup for remaining active resources...");

    let mut res_lock = resources.lock().await;
    for resource in res_lock.iter_mut() {
        if resource.state == ResourceState::Active {
            let resource_name = resource.name.clone();
            log::warn!("Force cleaning resource '{}'...", resource_name);

            if let Some(cleanup_fn) = resource.cleanup_fn.take() {
                let cleanup_future = cleanup_fn();
                // Give force cleanup a short 2-second window
                match tokio::time::timeout(Duration::from_secs(2), cleanup_future).await {
                    Ok(Ok(())) => {
                        resource.state = ResourceState::Cleaned;
                        resource.cleaned_at = Some(Instant::now());
                        force_cleaned.push(resource_name);
                    }
                    Ok(Err(e)) => {
                        log::error!("Force cleanup failed for '{}': {}", resource_name, e);
                        resource.state = ResourceState::CleanupFailed;
                    }
                    Err(_) => {
                        log::error!("Force cleanup timed out for '{}'", resource_name);
                        resource.state = ResourceState::TimedOut;
                    }
                }
            } else {
                resource.state = ResourceState::Cleaned;
                resource.cleaned_at = Some(Instant::now());
                force_cleaned.push(resource_name);
            }
        }
    }

    log::info!("Force cleanup completed: {} resources cleaned", force_cleaned.len());
    force_cleaned
}

/// Get a summary of all resource states for verification
pub async fn get_resource_summary(resources: &ResourceList) -> Vec<ResourceSummary> {
    let res_lock = resources.lock().await;
    res_lock
        .iter()
        .map(|r| ResourceSummary {
            name: r.name.clone(),
            priority: r.priority,
            state: r.state,
        })
        .collect()
}
