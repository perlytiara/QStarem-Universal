use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::UpdaterExt;

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
}

impl UpdateController {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(UpdateStatus {
                current_version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            }),
            pending: Mutex::new(None),
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

    pub fn has_pending_install(&self) -> bool {
        self.pending.lock().map(|guard| guard.is_some()).unwrap_or(false)
    }

    pub async fn check_and_download(app: AppHandle, manual: bool) -> Result<(), String> {
        let controller = app.state::<UpdateController>();
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

        let Some(update) = app
            .updater()
            .map_err(|error| error.to_string())?
            .check()
            .await
            .map_err(|error| error.to_string())?
        else {
            if let Ok(mut pending) = controller.pending.lock() {
                *pending = None;
            }
            controller.set_status(
                &app,
                UpdateStatus {
                    phase: UpdatePhase::Idle,
                    message: Some("You're up to date.".into()),
                    available_version: None,
                    notes: None,
                    progress: 0.0,
                    ..controller.snapshot()
                },
            );
            return Ok(());
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
        let bytes = update
            .download(
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
            )
            .await
            .map_err(|error| error.to_string())?;

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
    UpdateController::check_and_download(app.clone(), true).await?;
    Ok(app.state::<UpdateController>().snapshot())
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
