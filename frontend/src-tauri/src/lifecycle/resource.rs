use std::fmt;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// Priority tiers for cleanup ordering.
/// Lower priority value = cleaned up first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CleanupPriority {
    /// Audio streams, recording state — must be released first to free hardware
    Critical = 0,
    /// Transcription workers, model downloads, sidecar processes
    Secondary = 1,
    /// Database connections, app state — cleaned up last
    Persistent = 2,
}

impl fmt::Display for CleanupPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CleanupPriority::Critical => write!(f, "critical"),
            CleanupPriority::Secondary => write!(f, "secondary"),
            CleanupPriority::Persistent => write!(f, "persistent"),
        }
    }
}

/// Current state of a registered resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    /// Resource is registered and active
    Active,
    /// Resource has been cleaned up successfully
    Cleaned,
    /// Cleanup failed for this resource
    CleanupFailed,
    /// Cleanup timed out for this resource
    TimedOut,
}

impl fmt::Display for ResourceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceState::Active => write!(f, "active"),
            ResourceState::Cleaned => write!(f, "cleaned"),
            ResourceState::CleanupFailed => write!(f, "cleanup_failed"),
            ResourceState::TimedOut => write!(f, "timed_out"),
        }
    }
}

/// A cleanup function that can be called during shutdown.
/// Returns Ok(()) on success, Err with description on failure.
pub type CleanupFn = Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> + Send + Sync>;

/// A tracked resource in the lifecycle manager
pub struct TrackedResource {
    pub name: String,
    pub priority: CleanupPriority,
    pub state: ResourceState,
    pub cleanup_fn: Option<CleanupFn>,
    pub registered_at: Instant,
    pub cleaned_at: Option<Instant>,
}

impl TrackedResource {
    pub fn new(
        name: String,
        priority: CleanupPriority,
        cleanup_fn: CleanupFn,
    ) -> Self {
        Self {
            name,
            priority,
            state: ResourceState::Active,
            cleanup_fn: Some(cleanup_fn),
            registered_at: Instant::now(),
            cleaned_at: None,
        }
    }
}

impl fmt::Debug for TrackedResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrackedResource")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("state", &self.state)
            .field("has_cleanup", &self.cleanup_fn.is_some())
            .finish()
    }
}

/// Summary of all resources for logging/debugging
#[derive(Debug, Clone)]
pub struct ResourceSummary {
    pub name: String,
    pub priority: CleanupPriority,
    pub state: ResourceState,
}

/// Thread-safe wrapper for the resource list
pub type ResourceList = Arc<Mutex<Vec<TrackedResource>>>;

pub fn new_resource_list() -> ResourceList {
    Arc::new(Mutex::new(Vec::new()))
}
