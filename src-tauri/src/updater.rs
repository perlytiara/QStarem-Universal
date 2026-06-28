use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::timeout;

const CHECK_TIMEOUT: Duration = Duration::from_secs(45);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60 * 30);

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePhase {
    #[default]
    Idle,
    Checking,
    Downloading,
    Ready,
    Error,
}

#[derive(Clone, Serialize, Default)]
pub struct UpdateStatus {
    pub phase: UpdatePhase,
    pub current_version: String,
    pub available_version: Option<String>,
    pub notes: Option<String>,
    pub progress: f32,
    pub message: Option<String>,
}

struct PendingUpdate {
    update: tauri_plugin_updater::Update,
    bytes: Vec<u8>,
}

pub struct UpdateController {
    status: Mutex<UpdateStatus>,
    pending: Mutex<Option<PendingUpdate>>,
    operation: AsyncMutex<()>,
}

impl UpdateController {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(UpdateStatus {
                current_version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            }),
            pending: Mutex::new(None),
            operation: AsyncMutex::new(()),
        }
    }

    fn set_status(&self, app: &AppHandle, mut status: UpdateStatus) {
        status.current_version = env!("CARGO_PKG_VERSION").to_string();
        if let Ok(mut guard) = self.status.lock() {
            *guard = status.clone();
        }
        let _ = app.emit("update-status-changed", status);
    }

    pub fn snapshot(&self) -> UpdateStatus {
        self.status
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub async fn check_and_download(app: AppHandle, manual: bool) -> Result<(), String> {
        let controller = app.state::<UpdateController>();
        let _operation_guard = controller.operation.lock().await;

        controller.set_status(
            &app,
            UpdateStatus {
                phase: UpdatePhase::Checking,
                message: Some(if manual {
                    "Checking for updates…".into()
                } else {
                    "Looking for updates…".into()
                }),
                ..controller.snapshot()
            },
        );

        let updater = app
            .updater()
            .map_err(|error| format!("Updater unavailable: {error}"))?;

        let update = match timeout(CHECK_TIMEOUT, updater.check()).await {
            Ok(Ok(Some(update))) => update,
            Ok(Ok(None)) => {
                if let Ok(mut pending) = controller.pending.lock() {
                    *pending = None;
                }
                controller.set_status(
                    &app,
                    UpdateStatus {
                        phase: UpdatePhase::Idle,
                        message: Some(format!(
                            "You're up to date (v{}).",
                            env!("CARGO_PKG_VERSION")
                        )),
                        available_version: None,
                        notes: None,
                        progress: 0.0,
                        ..controller.snapshot()
                    },
                );
                return Ok(());
            }
            Ok(Err(error)) => {
                let message = format!("Update check failed: {error}");
                controller.set_status(
                    &app,
                    UpdateStatus {
                        phase: UpdatePhase::Error,
                        message: Some(message.clone()),
                        ..controller.snapshot()
                    },
                );
                return Err(message);
            }
            Err(_) => {
                let message: String =
                    "Update check timed out. Try again from the QStarem menu.".into();
                controller.set_status(
                    &app,
                    UpdateStatus {
                        phase: UpdatePhase::Error,
                        message: Some(message.clone()),
                        ..controller.snapshot()
                    },
                );
                return Err(message);
            }
        };

        let version = update.version.clone();
        let notes = update.body.clone().unwrap_or_default();

        controller.set_status(
            &app,
            UpdateStatus {
                phase: UpdatePhase::Downloading,
                available_version: Some(version.clone()),
                notes: Some(notes.clone()),
                message: Some(format!("Downloading QStarem {version}…")),
                progress: 0.0,
                ..controller.snapshot()
            },
        );

        let app_for_progress = app.clone();
        let mut downloaded = 0usize;
        let download_future = update.download(
            |chunk_length, content_length| {
                downloaded += chunk_length;
                let controller = app_for_progress.state::<UpdateController>();
                let progress = match content_length {
                    Some(total) if total > 0 => downloaded as f32 / total as f32,
                    _ => 0.0,
                };
                let mut snapshot = controller.snapshot();
                snapshot.phase = UpdatePhase::Downloading;
                snapshot.progress = progress.clamp(0.0, 1.0);
                snapshot.message = Some(format!("Downloading QStarem {version}…"));
                controller.set_status(&app_for_progress, snapshot);
            },
            || {},
        );

        let bytes = match timeout(DOWNLOAD_TIMEOUT, download_future).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(error)) => {
                let message = format!("Update download failed: {error}");
                controller.set_status(
                    &app,
                    UpdateStatus {
                        phase: UpdatePhase::Error,
                        message: Some(message.clone()),
                        ..controller.snapshot()
                    },
                );
                return Err(message);
            }
            Err(_) => {
                let message: String = "Update download timed out. Try again later.".into();
                controller.set_status(
                    &app,
                    UpdateStatus {
                        phase: UpdatePhase::Error,
                        message: Some(message.clone()),
                        ..controller.snapshot()
                    },
                );
                return Err(message);
            }
        };

        if let Ok(mut pending) = controller.pending.lock() {
            *pending = Some(PendingUpdate { update, bytes });
        }

        controller.set_status(
            &app,
            UpdateStatus {
                phase: UpdatePhase::Ready,
                available_version: Some(version.clone()),
                notes: Some(notes),
                progress: 1.0,
                message: Some(format!("QStarem {version} is ready to install.")),
                ..controller.snapshot()
            },
        );

        prompt_install(&app, &version)?;
        Ok(())
    }

    pub async fn install_ready_update(app: AppHandle) -> Result<(), String> {
        let controller = app.state::<UpdateController>();
        let pending = controller
            .pending
            .lock()
            .map_err(|_| "Update state is unavailable.".to_string())?
            .take()
            .ok_or_else(|| "No downloaded update is ready to install.".to_string())?;

        pending
            .update
            .install(&pending.bytes)
            .map_err(|error| error.to_string())?;

        app.restart();
    }
}

fn prompt_install(app: &AppHandle, version: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("update-prompt") {
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let label = "update-prompt";
    let version_js = version.replace('\\', "\\\\").replace('"', "\\\"");
    tauri::WebviewWindowBuilder::new(
        app,
        label,
        tauri::WebviewUrl::App("update-prompt.html".into()),
    )
    .title("Update available")
    .inner_size(420.0, 240.0)
    .resizable(false)
    .always_on_top(true)
    .center()
    .build()
    .map_err(|error| error.to_string())?;

    if let Some(window) = app.get_webview_window(label) {
        window
            .eval(format!(
                "document.getElementById('version').textContent = \"QStarem {version_js} is ready to install.\";"
            ))
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn get_update_status(controller: State<UpdateController>) -> UpdateStatus {
    controller.snapshot()
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateStatus, String> {
    let snapshot = app.state::<UpdateController>().snapshot();
    if matches!(snapshot.phase, UpdatePhase::Checking | UpdatePhase::Downloading) {
        return Ok(snapshot);
    }

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = UpdateController::check_and_download(handle.clone(), true).await {
            log::warn!("Manual update check failed: {error}");
        }
    });

    Ok(UpdateStatus {
        phase: UpdatePhase::Checking,
        message: Some("Checking for updates…".into()),
        ..snapshot
    })
}

#[tauri::command]
pub async fn install_pending_update(app: AppHandle) -> Result<(), String> {
    UpdateController::install_ready_update(app).await
}

#[tauri::command]
pub fn dismiss_update_prompt(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("update-prompt") {
        window.close().map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn spawn_startup_update_check(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = UpdateController::check_and_download(handle, false).await {
            log::warn!("Background update check failed: {error}");
        }
    });
}
