//! Meeting Detection Module
//!
//! Lightweight system audio activity monitoring to detect when meetings might be happening.
//! Emits events when system audio is active for extended periods while not recording.

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::Mutex;
use tokio::time;

use crate::database::repositories::setting::SettingsRepository;
use crate::state::AppState;

/// Threshold in seconds before emitting meeting activity detected event
const ACTIVITY_THRESHOLD_SECS: u64 = 30;

/// Check interval in milliseconds
const CHECK_INTERVAL_MS: u64 = 1000;

/// Audio level threshold for "active" detection (0.0 - 1.0)
const AUDIO_LEVEL_THRESHOLD: f32 = 0.01;

/// Global state for meeting detection
static MEETING_DETECTION_ENABLED: AtomicBool = AtomicBool::new(false);
static IS_MONITORING: AtomicBool = AtomicBool::new(false);
static ACTIVITY_START_TIME: AtomicI64 = AtomicBool::new(false);

/// Meeting activity event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingActivityEvent {
    pub detected: bool,
    pub duration_secs: u64,
    pub audio_level: f32,
}

/// Meeting detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingDetectionState {
    pub enabled: bool,
    pub is_monitoring: bool,
    pub activity_detected: bool,
    pub activity_duration_secs: u64,
}

/// Meeting detection manager
pub struct MeetingDetectionManager {
    activity_start: Arc<Mutex<Option<Instant>>>,
    last_audio_level: Arc<Mutex<f32>>,
}

impl MeetingDetectionManager {
    pub fn new() -> Self {
        Self {
            activity_start: Arc::new(Mutex::new(None)),
            last_audio_level: Arc::new(Mutex::new(0.0)),
        }
    }
}

impl Default for MeetingDetectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if meeting detection is enabled
#[tauri::command]
pub async fn get_meeting_detection_enabled<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let pool = &state.db_manager.pool;

    match SettingsRepository::get_meeting_detection_enabled(pool).await {
        Ok(enabled) => {
            MEETING_DETECTION_ENABLED.store(enabled, Ordering::SeqCst);
            Ok(enabled)
        }
        Err(e) => {
            warn!("Failed to get meeting detection setting: {}", e);
            Ok(false)
        }
    }
}

/// Set meeting detection enabled/disabled
#[tauri::command]
pub async fn set_meeting_detection_enabled<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let pool = &state.db_manager.pool;

    MEETING_DETECTION_ENABLED.store(enabled, Ordering::SeqCst);

    SettingsRepository::set_meeting_detection_enabled(pool, enabled)
        .await
        .map_err(|e| format!("Failed to save meeting detection setting: {}", e))?;

    info!("Meeting detection {}", if enabled { "enabled" } else { "disabled" });

    // Start or stop monitoring based on new setting
    if enabled {
        start_meeting_monitoring(app.clone()).await?;
    } else {
        stop_meeting_monitoring().await?;
    }

    Ok(())
}

/// Check current meeting activity status
#[tauri::command]
pub async fn check_meeting_activity() -> Result<MeetingActivityEvent, String> {
    let is_monitoring = IS_MONITORING.load(Ordering::SeqCst);

    if !is_monitoring {
        return Ok(MeetingActivityEvent {
            detected: false,
            duration_secs: 0,
            audio_level: 0.0,
        });
    }

    // Get current audio level from the global state
    let audio_level = get_current_system_audio_level().await;

    // Check if we're currently recording
    let is_recording = crate::audio::recording_commands::is_recording().await;

    // If recording, don't detect as meeting activity
    if is_recording {
        return Ok(MeetingActivityEvent {
            detected: false,
            duration_secs: 0,
            audio_level,
        });
    }

    // Check if audio level exceeds threshold
    if audio_level > AUDIO_LEVEL_THRESHOLD {
        let duration = get_activity_duration().await;

        if duration >= ACTIVITY_THRESHOLD_SECS {
            return Ok(MeetingActivityEvent {
                detected: true,
                duration_secs: duration,
                audio_level,
            });
        }
    }

    Ok(MeetingActivityEvent {
        detected: false,
        duration_secs: 0,
        audio_level,
    })
}

/// Get the current meeting detection state
#[tauri::command]
pub async fn get_meeting_detection_state() -> Result<MeetingDetectionState, String> {
    let enabled = MEETING_DETECTION_ENABLED.load(Ordering::SeqCst);
    let is_monitoring = IS_MONITORING.load(Ordering::SeqCst);
    let activity = check_meeting_activity().await?;

    Ok(MeetingDetectionState {
        enabled,
        is_monitoring,
        activity_detected: activity.detected,
        activity_duration_secs: activity.duration_secs,
    })
}

/// Start monitoring for meeting activity
pub async fn start_meeting_monitoring<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if IS_MONITORING.load(Ordering::SeqCst) {
        debug!("Meeting monitoring already active");
        return Ok(());
    }

    if !MEETING_DETECTION_ENABLED.load(Ordering::SeqCst) {
        debug!("Meeting detection not enabled, skipping monitoring start");
        return Ok(());
    }

    IS_MONITORING.store(true, Ordering::SeqCst);
    info!("Starting meeting activity monitoring");

    // Reset activity tracking
    reset_activity_tracking().await;

    // Spawn monitoring task
    let app_clone = app.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(CHECK_INTERVAL_MS));
        let mut last_event_time: Option<Instant> = None;

        loop {
            interval.tick().await;

            if !IS_MONITORING.load(Ordering::SeqCst) {
                info!("Meeting monitoring stopped");
                break;
            }

            // Skip if currently recording
            let is_recording = crate::audio::recording_commands::is_recording().await;
            if is_recording {
                reset_activity_tracking().await;
                continue;
            }

            // Get current audio level
            let audio_level = get_current_system_audio_level().await;

            // Update activity tracking
            if audio_level > AUDIO_LEVEL_THRESHOLD {
                update_activity_start().await;
            } else {
                reset_activity_tracking().await;
                continue;
            }

            // Check if threshold exceeded
            let duration = get_activity_duration().await;

            if duration >= ACTIVITY_THRESHOLD_SECS {
                // Emit event, but throttle to once per 60 seconds
                let should_emit = match last_event_time {
                    None => true,
                    Some(t) => t.elapsed() >= Duration::from_secs(60),
                };

                if should_emit {
                    info!(
                        "Meeting activity detected: {} seconds of audio activity",
                        duration
                    );

                    let event = MeetingActivityEvent {
                        detected: true,
                        duration_secs: duration,
                        audio_level,
                    };

                    if let Err(e) = app_clone.emit("meeting-activity-detected", event) {
                        error!("Failed to emit meeting activity event: {}", e);
                    }

                    last_event_time = Some(Instant::now());
                }
            }
        }
    });

    Ok(())
}

/// Stop monitoring for meeting activity
pub async fn stop_meeting_monitoring() -> Result<(), String> {
    if !IS_MONITORING.load(Ordering::SeqCst) {
        debug!("Meeting monitoring already stopped");
        return Ok(());
    }

    IS_MONITORING.store(false, Ordering::SeqCst);
    info!("Stopping meeting activity monitoring");

    Ok(())
}

/// Get current system audio level (simplified implementation)
async fn get_current_system_audio_level() -> f32 {
    // This is a simplified implementation that uses the existing audio level monitoring
    // In a full implementation, this would tap into the system audio capture
    // For now, we return a simulated value based on whether audio monitoring is active

    // Check if audio level monitoring is active
    if crate::audio::simple_level_monitor::is_monitoring().await {
        // Return a moderate level to indicate activity
        // In production, this would read from actual system audio
        0.05
    } else {
        0.0
    }
}

/// Activity tracking state
static ACTIVITY_TRACKING: AtomicBool = AtomicBool::new(false);
static ACTIVITY_START_TIMESTAMP: AtomicI64 = AtomicI64::new(0);

/// Update activity start time
async fn update_activity_start() {
    if !ACTIVITY_TRACKING.load(Ordering::SeqCst) {
        ACTIVITY_TRACKING.store(true, Ordering::SeqCst);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        ACTIVITY_START_TIMESTAMP.store(now, Ordering::SeqCst);
        debug!("Activity tracking started at timestamp {}", now);
    }
}

/// Reset activity tracking
async fn reset_activity_tracking() {
    ACTIVITY_TRACKING.store(false, Ordering::SeqCst);
    ACTIVITY_START_TIMESTAMP.store(0, Ordering::SeqCst);
}

/// Get current activity duration in seconds
async fn get_activity_duration() -> u64 {
    if !ACTIVITY_TRACKING.load(Ordering::SeqCst) {
        return 0;
    }

    let start = ACTIVITY_START_TIMESTAMP.load(Ordering::SeqCst);
    if start == 0 {
        return 0;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    (now - start).max(0) as u64
}

/// Initialize meeting detection from database settings
pub async fn initialize<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let state = app.state::<AppState>();
    let pool = &state.db_manager.pool;

    match SettingsRepository::get_meeting_detection_enabled(pool).await {
        Ok(enabled) => {
            MEETING_DETECTION_ENABLED.store(enabled, Ordering::SeqCst);
            info!("Meeting detection initialized: {}", enabled);

            if enabled {
                start_meeting_monitoring(app.clone()).await?;
            }
        }
        Err(e) => {
            warn!("Failed to load meeting detection setting, defaulting to disabled: {}", e);
            MEETING_DETECTION_ENABLED.store(false, Ordering::SeqCst);
        }
    }

    Ok(())
}
