use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex as StdMutex;
// Removed unused import

// Performance optimization: Conditional logging macros for hot paths
#[cfg(debug_assertions)]
macro_rules! perf_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! perf_debug {
    ($($arg:tt)*) => {};
}

#[cfg(debug_assertions)]
macro_rules! perf_trace {
    ($($arg:tt)*) => {
        log::trace!($($arg)*)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! perf_trace {
    ($($arg:tt)*) => {};
}

// Make these macros available to other modules

// Re-export async logging macros for external use (removed due to macro conflicts)

// Declare audio module
pub mod api;
pub mod audio;
pub mod config;
pub mod console_utils;
pub mod database;
pub mod lifecycle;
pub mod llm_engine;

pub mod ollama;
pub mod onboarding;
pub mod openai;
pub mod anthropic;
pub mod parakeet_engine;
pub mod signal_handler;
pub mod state;
pub mod summary;
pub mod tray;
pub mod utils;
pub mod whisper_engine;

use audio::{list_audio_devices, AudioDevice, trigger_audio_permission};
use log::{error as log_error, info as log_info};

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

static RECORDING_FLAG: AtomicBool = AtomicBool::new(false);

// Global language preference storage (default to "auto-translate" for automatic translation to English)
static LANGUAGE_PREFERENCE: std::sync::LazyLock<StdMutex<String>> =
    std::sync::LazyLock::new(|| StdMutex::new("auto-translate".to_string()));

#[derive(Debug, Deserialize)]
struct RecordingArgs {
    save_path: String,
}

#[derive(Debug, Serialize, Clone)]
struct TranscriptionStatus {
    chunks_in_queue: usize,
    is_processing: bool,
    last_activity_ms: u64,
}

#[derive(Debug, Serialize, Clone)]
struct AppState {
    is_recording: bool,
    is_paused: bool,
    is_audio_level_monitoring: bool,
}

#[tauri::command]
async fn get_app_state() -> AppState {
    AppState {
        is_recording: audio::recording_commands::is_recording().await,
        is_paused: audio::recording_commands::is_recording_paused().await,
        is_audio_level_monitoring: audio::simple_level_monitor::is_monitoring(),
    }
}

#[tauri::command]
async fn start_recording<R: Runtime>(
    app: AppHandle<R>,
    mic_device_name: Option<String>,
    system_device_name: Option<String>,
    meeting_name: Option<String>,
) -> Result<(), String> {
    log_info!("🔥 CALLED start_recording with meeting: {:?}", meeting_name);
    log_info!(
        "📋 Backend received parameters - mic: {:?}, system: {:?}, meeting: {:?}",
        mic_device_name,
        system_device_name,
        meeting_name
    );

    if audio::recording_commands::is_recording().await {
        return Err("Recording already in progress".to_string());
    }

    // Call the actual audio recording system with meeting name
    match audio::recording_commands::start_recording_with_devices_and_meeting(
        app.clone(),
        mic_device_name,
        system_device_name,
        meeting_name.clone(),
    )
    .await
    {
        Ok(_) => {
            RECORDING_FLAG.store(true, Ordering::SeqCst);
            tray::update_tray_menu(&app);

            log_info!("Recording started successfully");

            // Show recording started notification
            if let Err(e) = app.notification()
                .builder()
                .title("Twin")
                .body("Recording started")
                .show()
            {
                log_error!("Failed to show recording started notification: {}", e);
            } else {
                log_info!("Successfully showed recording started notification");
            }

            Ok(())
        }
        Err(e) => {
            log_error!("Failed to start audio recording: {}", e);
            Err(format!("Failed to start recording: {}", e))
        }
    }
}

#[tauri::command]
async fn stop_recording<R: Runtime>(app: AppHandle<R>, args: RecordingArgs) -> Result<(), String> {
    log_info!("Attempting to stop recording...");

    // Check the actual audio recording system state instead of the flag
    if !audio::recording_commands::is_recording().await {
        log_info!("Recording is already stopped");
        return Ok(());
    }

    // Call the actual audio recording system to stop
    match audio::recording_commands::stop_recording(
        app.clone(),
        audio::recording_commands::RecordingArgs {
            save_path: args.save_path.clone(),
        },
    )
    .await
    {
        Ok(_) => {
            RECORDING_FLAG.store(false, Ordering::SeqCst);
            tray::update_tray_menu(&app);

            // Create the save directory if it doesn't exist
            if let Some(parent) = std::path::Path::new(&args.save_path).parent() {
                if !parent.exists() {
                    log_info!("Creating directory: {:?}", parent);
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        let err_msg = format!("Failed to create save directory: {}", e);
                        log_error!("{}", err_msg);
                        return Err(err_msg);
                    }
                }
            }

            // Show recording stopped notification
            if let Err(e) = app.notification()
                .builder()
                .title("Twin")
                .body("Recording stopped")
                .show()
            {
                log_error!("Failed to show recording stopped notification: {}", e);
            } else {
                log_info!("Successfully showed recording stopped notification");
            }

            Ok(())
        }
        Err(e) => {
            log_error!("Failed to stop audio recording: {}", e);
            // Still update the flag even if stopping failed
            RECORDING_FLAG.store(false, Ordering::SeqCst);
            tray::update_tray_menu(&app);
            Err(format!("Failed to stop recording: {}", e))
        }
    }
}

#[tauri::command]
fn get_transcription_status() -> TranscriptionStatus {
    TranscriptionStatus {
        chunks_in_queue: 0,
        is_processing: false,
        last_activity_ms: 0,
    }
}

#[tauri::command]
fn read_audio_file(file_path: String) -> Result<Vec<u8>, String> {
    match std::fs::read(&file_path) {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Failed to read audio file: {}", e)),
    }
}

#[tauri::command]
async fn save_transcript(file_path: String, content: String) -> Result<(), String> {
    log_info!("Saving transcript to: {}", file_path);

    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }

    // Write content to file
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write transcript: {}", e))?;

    log_info!("Transcript saved successfully");
    Ok(())
}

// Audio level monitoring commands
#[tauri::command]
async fn start_audio_level_monitoring<R: Runtime>(
    app: AppHandle<R>,
    device_names: Vec<String>,
) -> Result<(), String> {
    log_info!(
        "Starting audio level monitoring for devices: {:?}",
        device_names
    );

    audio::simple_level_monitor::start_monitoring(app, device_names)
        .await
        .map_err(|e| format!("Failed to start audio level monitoring: {}", e))
}

#[tauri::command]
async fn stop_audio_level_monitoring() -> Result<(), String> {
    log_info!("Stopping audio level monitoring");

    audio::simple_level_monitor::stop_monitoring()
        .await
        .map_err(|e| format!("Failed to stop audio level monitoring: {}", e))
}

// Whisper commands are now handled by whisper_engine::commands module

#[tauri::command]
async fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    list_audio_devices()
        .await
        .map_err(|e| format!("Failed to list audio devices: {}", e))
}

#[tauri::command]
async fn trigger_microphone_permission() -> Result<bool, String> {
    trigger_audio_permission()
        .map_err(|e| format!("Failed to trigger microphone permission: {}", e))
}

#[tauri::command]
async fn set_language_preference(language: String) -> Result<(), String> {
    let mut lang_pref = LANGUAGE_PREFERENCE
        .lock()
        .map_err(|e| format!("Failed to set language preference: {}", e))?;
    log_info!("Setting language preference to: {}", language);
    *lang_pref = language;
    Ok(())
}

// Internal helper function to get language preference (for use within Rust code)
pub fn get_language_preference_internal() -> Option<String> {
    LANGUAGE_PREFERENCE.lock().ok().map(|lang| lang.clone())
}

pub fn run() {
    log::set_max_level(log::LevelFilter::Info);

    let mut builder = tauri::Builder::default();

    #[cfg(any(target_os = "macos", windows, target_os = "linux"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            log_info!(
                "Second app instance requested with args: {:?}, cwd: {:?}",
                args,
                cwd
            );

            tray::focus_main_window(app);
        }));
    }

    builder
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(audio::init_system_audio_state())
        .manage(std::sync::Arc::new(lifecycle::AppLifecycleManager::new()))
        .setup(|app| {
            log::info!("Application setup complete");

            // Install signal handlers for graceful shutdown
            let app_handle = app.handle().clone();
            signal_handler::install_signal_handlers(&app_handle);

            // Register core resources with lifecycle manager (synchronously during setup)
            let lifecycle_manager = app.state::<std::sync::Arc<lifecycle::AppLifecycleManager>>();
            
            // Register database as a persistent resource
            if let Some(app_state) = app.try_state::<state::AppState>() {
                let db_manager = app_state.db_manager.clone();
                tauri::async_runtime::block_on(async {
                    lifecycle_manager
                        .register(
                            "database",
                            lifecycle::resource::CleanupPriority::Persistent,
                            Box::new(move || {
                                Box::pin(async move {
                                    log::info!("Lifecycle: cleaning up database...");
                                    db_manager.cleanup().await.map_err(|e| {
                                        log::error!("Database cleanup failed: {}", e);
                                        format!("Database cleanup failed: {}", e)
                                    })?;
                                    log::info!("Lifecycle: database cleanup complete");
                                    Ok(())
                                })
                            }),
                        )
                        .await;
                });
            }

            tauri::async_runtime::block_on(async {
                // Register Whisper engine as a secondary resource for GPU cleanup
                lifecycle_manager
                    .register(
                        "whisper_engine",
                        lifecycle::resource::CleanupPriority::Secondary,
                        Box::new(|| {
                            Box::pin(async move {
                                log::info!("Lifecycle: unloading Whisper engine GPU context...");
                                let engine = {
                                    let guard = whisper_engine::commands::WHISPER_ENGINE.lock()
                                        .map_err(|e| format!("Whisper engine lock poisoned: {}", e))?;
                                    guard.clone()
                                };
                                if let Some(engine) = engine {
                                    engine.unload_model().await;
                                }
                                log::info!("Lifecycle: Whisper engine GPU context released");
                                Ok(())
                            })
                        }),
                    )
                    .await;

                // Register Parakeet engine as a secondary resource for GPU cleanup
                lifecycle_manager
                    .register(
                        "parakeet_engine",
                        lifecycle::resource::CleanupPriority::Secondary,
                        Box::new(|| {
                            Box::pin(async move {
                                log::info!("Lifecycle: unloading Parakeet engine GPU context...");
                                let engine = {
                                    let guard = parakeet_engine::commands::PARAKEET_ENGINE.lock()
                                        .map_err(|e| format!("Parakeet engine lock poisoned: {}", e))?;
                                    guard.clone()
                                };
                                if let Some(engine) = engine {
                                    engine.unload_model().await;
                                }
                                log::info!("Lifecycle: Parakeet engine GPU context released");
                                Ok(())
                            })
                        }),
                    )
                    .await;

                // Register LLM engine as a secondary resource for GPU cleanup
                lifecycle_manager
                    .register(
                        "llm_engine",
                        lifecycle::resource::CleanupPriority::Secondary,
                        Box::new(|| {
                            Box::pin(async move {
                                log::info!("Lifecycle: unloading LLM engine GPU context...");
                                let engine = {
                                    let guard = llm_engine::commands::LLM_ENGINE.lock()
                                        .map_err(|e| format!("LLM engine lock poisoned: {}", e))?;
                                    guard.clone()
                                };
                                if let Some(engine) = engine {
                                    let mut engine_guard = engine.lock().await;
                                    engine_guard.unload_model().await;
                                }
                                log::info!("Lifecycle: LLM engine GPU context released");
                                Ok(())
                            })
                        }),
                    )
                    .await;
            });

            // Initialize system tray
            if let Err(e) = tray::create_tray(app.handle()) {
                log::error!("Failed to create system tray: {}", e);
            }

            // Set models directory to use app_data_dir (unified storage location)
            whisper_engine::commands::set_models_directory(&app.handle());

            // Initialize Whisper engine on startup
            tauri::async_runtime::spawn(async {
                if let Err(e) = whisper_engine::commands::whisper_init().await {
                    log::error!("Failed to initialize Whisper engine on startup: {}", e);
                }
            });

            // Set Parakeet models directory
            parakeet_engine::commands::set_models_directory(&app.handle());

            // Initialize Parakeet engine on startup
            tauri::async_runtime::spawn(async {
                if let Err(e) = parakeet_engine::commands::parakeet_init().await {
                    log::error!("Failed to initialize Parakeet engine on startup: {}", e);
                }
            });

            // Set LLM models directory
            llm_engine::commands::set_models_directory(&app.handle());

            // Initialize LLM engine on startup
            tauri::async_runtime::spawn(async {
                if let Err(e) = llm_engine::commands::llm_init().await {
                    log::error!("Failed to initialize LLM engine on startup: {}", e);
                }
            });

            // Trigger system audio permission request on startup (similar to microphone permission)
            // #[cfg(target_os = "macos")]
            // {
            //     tauri::async_runtime::spawn(async {
            //         if let Err(e) = audio::permissions::trigger_system_audio_permission() {
            //             log::warn!("Failed to trigger system audio permission: {}", e);
            //         }
            //     });
            // }

            // Initialize database (handles first launch detection and conditional setup)
            tauri::async_runtime::block_on(async {
                database::setup::initialize_database_on_startup(&app.handle()).await
            })
            .expect("Failed to initialize database");

            // Initialize bundled templates directory for dynamic template discovery
            log::info!("Initializing bundled templates directory...");
            if let Ok(resource_path) = app.handle().path().resource_dir() {
                let templates_dir = resource_path.join("templates");
                log::info!("Setting bundled templates directory to: {:?}", templates_dir);
                summary::templates::set_bundled_templates_dir(templates_dir);
            } else {
                log::warn!("Failed to resolve resource directory for templates");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    log::info!("Window close requested — hiding to tray (use tray menu 'Quit' to fully exit)");
                    api.prevent_close();
                    if let Err(e) = window.hide() {
                        log::error!("Failed to hide main window on close request: {}", e);
                    } else {
                        log::info!("Main window hidden to tray successfully");
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            get_transcription_status,
            get_app_state,
            read_audio_file,
            save_transcript,
            whisper_engine::commands::whisper_init,
            whisper_engine::commands::whisper_get_available_models,
            whisper_engine::commands::whisper_load_model,
            whisper_engine::commands::whisper_get_current_model,
            whisper_engine::commands::whisper_is_model_loaded,
            whisper_engine::commands::whisper_has_available_models,
            whisper_engine::commands::whisper_validate_model_ready,
            whisper_engine::commands::whisper_transcribe_audio,
            whisper_engine::commands::whisper_get_models_directory,
            whisper_engine::commands::whisper_download_model,
            whisper_engine::commands::whisper_cancel_download,
            whisper_engine::commands::whisper_delete_corrupted_model,
            whisper_engine::commands::whisper_get_acceleration_info,
            // Parakeet engine commands
            parakeet_engine::commands::parakeet_init,
            parakeet_engine::commands::parakeet_get_available_models,
            parakeet_engine::commands::parakeet_load_model,
            parakeet_engine::commands::parakeet_get_current_model,
            parakeet_engine::commands::parakeet_is_model_loaded,
            parakeet_engine::commands::parakeet_has_available_models,
            parakeet_engine::commands::parakeet_validate_model_ready,
            parakeet_engine::commands::parakeet_transcribe_audio,
            parakeet_engine::commands::parakeet_get_models_directory,
            parakeet_engine::commands::parakeet_download_model,
            parakeet_engine::commands::parakeet_retry_download,
            parakeet_engine::commands::parakeet_cancel_download,
            parakeet_engine::commands::parakeet_delete_corrupted_model,
            parakeet_engine::commands::open_parakeet_models_folder,
            // LLM engine commands
            llm_engine::commands::llm_init,
            llm_engine::commands::llm_get_available_models,
            llm_engine::commands::llm_download_model,
            llm_engine::commands::llm_delete_model,
            llm_engine::commands::llm_load_model,
            llm_engine::commands::llm_unload_model,
            llm_engine::commands::llm_is_model_loaded,
            llm_engine::commands::llm_get_current_model,
            llm_engine::commands::llm_get_models_directory,
            llm_engine::commands::llm_get_gpu_info,
            llm_engine::commands::open_llm_models_folder,
            llm_engine::commands::llm_recommend_model,
            get_audio_devices,
            trigger_microphone_permission,
            start_audio_level_monitoring,
            stop_audio_level_monitoring,
            // Recording pause/resume commands
            audio::recording_commands::pause_recording,
            audio::recording_commands::resume_recording,
            audio::recording_commands::is_recording_paused,
            audio::recording_commands::get_recording_state,
            audio::recording_commands::get_meeting_folder_path,
            // Reload sync commands (retrieve transcript history and meeting name)
            audio::recording_commands::get_transcript_history,
            audio::recording_commands::get_recording_meeting_name,
            // Device monitoring commands (AirPods/Bluetooth disconnect/reconnect)
            audio::recording_commands::poll_audio_device_events,
            audio::recording_commands::get_reconnection_status,
            audio::recording_commands::attempt_device_reconnect,
            // Audio recovery commands (for transcript recovery feature)
            audio::incremental_saver::recover_audio_from_checkpoints,
            audio::incremental_saver::cleanup_checkpoints,
            audio::incremental_saver::has_audio_checkpoints,
            console_utils::show_console,
            console_utils::hide_console,
            console_utils::toggle_console,
            ollama::get_ollama_models,
            ollama::pull_ollama_model,
            ollama::delete_ollama_model,
            ollama::get_ollama_model_context,
            openai::openai::get_openai_models,
            anthropic::anthropic::get_anthropic_models,
            api::api_get_meetings,
            api::api_search_transcripts,
            api::api_get_profile,
            api::api_save_profile,
            api::api_update_profile,
            api::api_get_model_config,
            api::api_save_model_config,
            api::api_get_api_key,
            // api::api_get_auto_generate_setting,
            // api::api_save_auto_generate_setting,
            api::api_get_transcript_config,
            api::api_save_transcript_config,
            api::api_get_transcript_api_key,
            api::api_delete_meeting,
            api::api_get_meeting,
            api::api_get_meeting_metadata,
            api::api_get_meeting_transcripts,
            api::api_save_meeting_title,
            api::api_save_transcript,
            api::open_meeting_folder,
            api::test_backend_connection,
            api::debug_backend_connection,
            api::open_external_url,
            // Custom OpenAI commands
            api::api_save_custom_openai_config,
            api::api_get_custom_openai_config,
            api::api_test_custom_openai_connection,
            // Summary commands
            summary::commands::api_process_transcript,
            summary::commands::api_get_summary,
            summary::commands::api_save_meeting_summary,
            summary::commands::api_get_meeting_summary_language,
            summary::commands::api_save_meeting_summary_language,
            summary::commands::api_get_meeting_detected_summary_language,
            summary::commands::api_save_meeting_detected_summary_language,
            summary::commands::api_detect_transcript_summary_language,
            summary::commands::api_cancel_summary,
            // Template commands
            summary::template_commands::api_list_templates,
            summary::template_commands::api_get_template_details,
            summary::template_commands::api_validate_template,
            audio::recording_preferences::get_recording_preferences,
            audio::recording_preferences::set_recording_preferences,
            audio::recording_preferences::get_default_recordings_folder_path,
            audio::recording_preferences::open_recordings_folder,
            audio::recording_preferences::select_recording_folder,
            audio::recording_preferences::get_available_audio_backends,
            audio::recording_preferences::get_current_audio_backend,
            audio::recording_preferences::set_audio_backend,
            audio::recording_preferences::get_audio_backend_info,
            // Language preference commands
            set_language_preference,
            // System audio capture commands
            audio::system_audio_commands::start_system_audio_capture_command,
            audio::system_audio_commands::list_system_audio_devices_command,
            audio::system_audio_commands::check_system_audio_permissions_command,
            audio::system_audio_commands::start_system_audio_monitoring,
            audio::system_audio_commands::stop_system_audio_monitoring,
            audio::system_audio_commands::get_system_audio_monitoring_status,
            // Screen Recording permission commands
            audio::permissions::check_screen_recording_permission_command,
            audio::permissions::request_screen_recording_permission_command,
            audio::permissions::trigger_system_audio_permission_command,
            // Database import commands
            database::commands::check_first_launch,
            database::commands::import_and_initialize_database,
            database::commands::initialize_fresh_database,
            // Database and Models path commands
            database::commands::get_database_directory,
            database::commands::open_database_folder,
            whisper_engine::commands::open_models_folder,
            // Speaker diarization commands
            database::commands_speaker::get_speakers,
            database::commands_speaker::rename_speaker,
            // Onboarding commands
            onboarding::get_onboarding_status,
            onboarding::save_onboarding_status_cmd,
            onboarding::reset_onboarding_status_cmd,
            onboarding::complete_onboarding,
            onboarding::skip_onboarding_cmd,
            // System settings commands
            #[cfg(target_os = "macos")]
            utils::open_system_settings,
            // Retranscription commands
            audio::retranscription::start_retranscription_command,
            audio::retranscription::cancel_retranscription_command,
            audio::retranscription::is_retranscription_in_progress_command,
            // Import audio commands
            audio::import::select_and_validate_audio_command,
            audio::import::validate_audio_file_command,
            audio::import::start_import_audio_command,
            audio::import::cancel_import_command,
            audio::import::is_import_in_progress_command,
            // Action items commands
            api::api_get_all_action_items,
            api::api_update_action_item,
            // FTS5 search commands
            api::api_search_meetings_fts,
            // Export commands
            api::api_export_meeting_transcript,
            api::api_export_meeting_bundle,
            // Meeting context commands
            api::api_save_meeting_context,
            api::api_get_meeting_context,
            // Meeting notes commands
            api::api_get_meeting_notes,
            api::api_save_meeting_notes,
            api::api_get_meetings_with_notes,
            // Meeting audio commands
            api::api_get_meeting_audio_path,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            match event {
                #[cfg(target_os = "macos")]
                tauri::RunEvent::Reopen { .. } => {
                    tray::focus_main_window(_app_handle);
                }
                tauri::RunEvent::Exit => {
                    log::info!("=== Application exit detected, starting lifecycle shutdown ===");
                    tauri::async_runtime::block_on(async {
                        // Use lifecycle manager for ordered shutdown
                        if let Some(lifecycle_manager) = _app_handle.try_state::<std::sync::Arc<lifecycle::AppLifecycleManager>>() {
                            log::info!("Executing ordered shutdown via lifecycle manager...");
                            let result = lifecycle_manager.shutdown().await;
                            if result.success {
                                log::info!("Lifecycle shutdown completed successfully: {}", result.summary());
                            } else {
                                log::warn!("Lifecycle shutdown completed with issues: {}", result.summary());
                            }
                        } else {
                            log::warn!("LifecycleManager not available, falling back to manual cleanup");
                            
                            // Fallback: manual cleanup (legacy behavior)
                            if let Some(app_state) = _app_handle.try_state::<state::AppState>() {
                                log::info!("Starting database cleanup...");
                                if let Err(e) = app_state.db_manager.cleanup().await {
                                    log::error!("Failed to cleanup database: {}", e);
                                } else {
                                    log::info!("Database cleanup completed successfully");
                                }
                            } else {
                                log::warn!("AppState not available for database cleanup (likely first launch)");
                            }

                            log::info!("Cleaning up Whisper engine GPU context...");
                            {
                                let engine = {
                                    let guard = whisper_engine::commands::WHISPER_ENGINE.lock();
                                    if let Ok(guard) = guard {
                                        guard.clone()
                                    } else {
                                        None
                                    }
                                };
                                if let Some(engine) = engine {
                                    engine.unload_model().await;
                                }
                            }
                            log::info!("Cleaning up Parakeet engine GPU context...");
                            {
                                let engine = {
                                    let guard = parakeet_engine::commands::PARAKEET_ENGINE.lock();
                                    if let Ok(guard) = guard {
                                        guard.clone()
                                    } else {
                                        None
                                    }
                                };
                                if let Some(engine) = engine {
                                    engine.unload_model().await;
                                }
                            }
                            log::info!("Cleaning up LLM engine GPU context...");
                            {
                                let engine = {
                                    let guard = llm_engine::commands::LLM_ENGINE.lock();
                                    if let Ok(guard) = guard {
                                        guard.clone()
                                    } else {
                                        None
                                    }
                                };
                                if let Some(engine) = engine {
                                    let mut engine_guard = engine.lock().await;
                                    engine_guard.unload_model().await;
                                }
                            }
                        }
                    });
                    log::info!("=== Application cleanup complete ===");
                }
                _ => {}
            }
        });
}
