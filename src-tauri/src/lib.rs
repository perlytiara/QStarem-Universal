mod commands;
mod extensions;
mod icons;
mod settings;
mod updater;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, RunEvent};

use commands::{
    clear_browsing_data, get_settings, go_home, inject_runtime, navigate_back, navigate_forward,
    open_settings, reload_page, save_settings,
};
use settings::AppSettings;
use updater::{
    check_for_updates, dismiss_update_prompt, get_app_version, get_update_status,
    install_pending_update, spawn_startup_update_check, UpdateController,
};

use icons::apply_app_icon;

fn build_menu(app: &AppHandle) -> tauri::Result<()> {
    let app_menu = Submenu::with_items(
        app,
        "QStarem",
        true,
        &[
            &PredefinedMenuItem::about(app, None, None)?,
            &MenuItem::with_id(app, "check_updates", "Check for Updates…", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, None)?,
        ],
    )?;

    let navigation = Submenu::with_items(
        app,
        "Navigation",
        true,
        &[
            &MenuItem::with_id(app, "back", "Back", true, Some("CmdOrCtrl+["))?,
            &MenuItem::with_id(app, "forward", "Forward", true, Some("CmdOrCtrl+]"))?,
            &MenuItem::with_id(app, "reload", "Reload", true, Some("CmdOrCtrl+R"))?,
            &MenuItem::with_id(app, "home", "Home", true, Some("CmdOrCtrl+Shift+H"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "settings", "Settings…", true, Some("CmdOrCtrl+,"))?,
        ],
    )?;

    let edit = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let view = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &MenuItem::with_id(app, "clear_data", "Clear Browsing Data", true, None::<&str>)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
        ],
    )?;

    let menu = Menu::with_items(app, &[&app_menu, &navigation, &edit, &view])?;
    app.set_menu(menu)?;
    Ok(())
}

fn handle_menu_event(app: &AppHandle, event_id: &str) {
    if event_id == "check_updates" {
        let handle = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(error) = crate::updater::UpdateController::check_and_download(handle, true).await {
                log::error!("Menu update check failed: {error}");
            }
        });
        return;
    }

    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let result = match event_id {
        "back" => navigate_back(window),
        "forward" => navigate_forward(window),
        "reload" => reload_page(window),
        "home" => go_home(window),
        "settings" => open_settings(app.clone()),
        "clear_data" => clear_browsing_data(app.clone()),
        _ => Ok(()),
    };

    if let Err(error) = result {
        log::error!("Menu action failed: {error}");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(UpdateController::new())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            navigate_back,
            navigate_forward,
            reload_page,
            go_home,
            clear_browsing_data,
            open_settings,
            get_app_version,
            get_update_status,
            check_for_updates,
            install_pending_update,
            dismiss_update_prompt,
        ])
        .setup(|app| {
            let settings = AppSettings::load();

            if let Err(error) = apply_app_icon(app.handle(), settings.app_icon_id) {
                log::warn!("Failed to apply app icon: {error}");
            }

            if let Some(window) = app.get_webview_window("main") {
                if settings.home_url != settings::DEFAULT_HOME_URL {
                    if let Ok(url) = settings.home_url.parse() {
                        let _ = window.navigate(url);
                    }
                }

                if let Err(error) = inject_runtime(&window, &settings) {
                    log::warn!("Initial runtime injection failed: {error}");
                }

                #[cfg(target_os = "macos")]
                if let Err(error) = extensions::apply_macos_window_theme(&window) {
                    log::warn!("Failed to apply macOS window theme: {error}");
                }
            }

            build_menu(app.handle())?;
            let _ = extensions::refresh_cached_userscript();
            spawn_startup_update_check(app.handle());

            Ok(())
        })
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id().0.as_str());
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::ExitRequested { .. } = event {
                app.exit(0);
            }
        });
}
