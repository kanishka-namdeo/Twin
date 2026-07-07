use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

use crate::lifecycle::AppLifecycleManager;

static HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Install platform-specific signal handlers for graceful shutdown.
///
/// - Unix: SIGTERM, SIGINT
/// - Windows: Ctrl+C, Ctrl+Break
///
/// These handlers trigger the lifecycle manager's shutdown sequence when
/// the process receives a termination signal from the OS.
pub fn install_signal_handlers(app_handle: &AppHandle) {
    if HANDLER_INSTALLED.swap(true, Ordering::SeqCst) {
        log::warn!("Signal handlers already installed, skipping");
        return;
    }

    log::info!("Installing platform-specific signal handlers...");

    let app_handle = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        #[cfg(unix)]
        {
            install_unix_handlers(app_handle).await;
        }

        #[cfg(windows)]
        {
            install_windows_handlers(app_handle).await;
        }

        #[cfg(not(any(unix, windows)))]
        {
            log::warn!("No signal handler support on this platform");
            let _ = app_handle;
        }
    });
}

#[cfg(unix)]
async fn install_unix_handlers(app_handle: AppHandle) {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to install SIGTERM handler: {}", e);
            return;
        }
    };

    let mut sigint = match signal(SignalKind::interrupt()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to install SIGINT handler: {}", e);
            return;
        }
    };

    log::info!("Unix signal handlers installed (SIGTERM, SIGINT)");

    tokio::select! {
        _ = sigterm.recv() => {
            log::warn!("Received SIGTERM, initiating graceful shutdown...");
        }
        _ = sigint.recv() => {
            log::warn!("Received SIGINT, initiating graceful shutdown...");
        }
    }

    trigger_shutdown(&app_handle, "signal").await;
}

#[cfg(windows)]
async fn install_windows_handlers(app_handle: AppHandle) {
    let mut ctrl_c = match tokio::signal::windows::ctrl_c() {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to install Ctrl+C handler: {}", e);
            return;
        }
    };

    let mut ctrl_break = match tokio::signal::windows::ctrl_break() {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to install Ctrl+Break handler: {}", e);
            return;
        }
    };

    log::info!("Windows signal handlers installed (Ctrl+C, Ctrl+Break)");

    tokio::select! {
        _ = ctrl_c.recv() => {
            log::warn!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = ctrl_break.recv() => {
            log::warn!("Received Ctrl+Break, initiating graceful shutdown...");
        }
    }

    trigger_shutdown(&app_handle, "signal").await;
}

/// Trigger the lifecycle shutdown sequence via the lifecycle manager.
async fn trigger_shutdown(app_handle: &AppHandle, source: &str) {
    log::info!("Signal handler ({}): triggering lifecycle shutdown...", source);

    if let Some(lifecycle_manager) = app_handle.try_state::<std::sync::Arc<AppLifecycleManager>>() {
        let result = lifecycle_manager.shutdown().await;
        if result.success {
            log::info!("Signal handler ({}): shutdown completed successfully", source);
        } else {
            log::warn!(
                "Signal handler ({}): shutdown completed with issues: {}",
                source,
                result.summary()
            );
        }
    } else {
        log::warn!("Signal handler ({}): LifecycleManager not available", source);
    }
}
