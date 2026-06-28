use tauri::{AppHandle, Manager, WebviewWindow};

use crate::extensions::{injection_runner, reset_injection_flag_script};
use crate::icons::apply_app_icon;
use crate::settings::AppSettings;

#[tauri::command]
pub fn get_settings() -> AppSettings {
    AppSettings::load()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    settings.save()?;
    apply_runtime_settings(&app, &settings)?;
    Ok(())
}

#[tauri::command]
pub fn navigate_back(window: WebviewWindow) -> Result<(), String> {
    window
        .eval("window.history.back()")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn navigate_forward(window: WebviewWindow) -> Result<(), String> {
    window
        .eval("window.history.forward()")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reload_page(window: WebviewWindow) -> Result<(), String> {
    window
        .eval("window.location.reload()")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn go_home(window: WebviewWindow) -> Result<(), String> {
    let home = AppSettings::load().home_url;
    let escaped = home.replace('\\', "\\\\").replace('"', "\\\"");
    window
        .eval(format!("window.location.href = \"{escaped}\";"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_browsing_data(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    window
        .eval(
            r#"
try {
  localStorage.clear();
  sessionStorage.clear();
  if (window.indexedDB && indexedDB.databases) {
    indexedDB.databases().then((dbs) => {
      dbs.forEach((db) => indexedDB.deleteDatabase(db.name));
    });
  }
  if (window.caches) {
    caches.keys().then((keys) => keys.forEach((key) => caches.delete(key)));
  }
} catch (error) {
  console.error('[QStarem] clear browsing data failed', error);
}
"#,
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        &app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into()),
    )
    .title("QStarem Settings")
    .inner_size(520.0, 680.0)
    .resizable(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn apply_runtime_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    apply_app_icon(app, settings.app_icon_id)?;

    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    window
        .eval(reset_injection_flag_script())
        .map_err(|e| e.to_string())?;

    let script = injection_runner(settings.p_stream_enabled);
    if !script.is_empty() {
        window.eval(&script).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn inject_runtime(window: &WebviewWindow, settings: &AppSettings) -> Result<(), String> {
    let script = injection_runner(settings.p_stream_enabled);
    if script.is_empty() {
        return Ok(());
    }

    window.eval(&script).map_err(|e| e.to_string())
}
