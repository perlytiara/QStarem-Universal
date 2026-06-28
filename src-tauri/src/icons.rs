use std::path::{Path, PathBuf};

use tauri::image::Image;
use tauri::{AppHandle, Manager, Runtime};

pub fn apply_app_icon<R: Runtime>(app: &AppHandle<R>, icon_id: u8) -> Result<(), String> {
    let icon_id = icon_id.clamp(1, 6);

    if let Some(window) = app.get_webview_window("main") {
        let png_path = resolve_png_path(app, icon_id)?;
        let image = Image::from_path(&png_path)
            .map_err(|error| error.to_string())?
            .to_owned();
        window.set_icon(image).map_err(|error| error.to_string())?;
    }

    #[cfg(target_os = "macos")]
    set_macos_application_icon(app, icon_id)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn set_macos_application_icon<R: Runtime>(app: &AppHandle<R>, icon_id: u8) -> Result<(), String> {
    use objc2::AnyThread;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSString;

    let path = resolve_icns_path(app, icon_id)?;
    let mtm = MainThreadMarker::new().ok_or("Must run on main thread")?;
    let path_string = path
        .to_str()
        .ok_or_else(|| format!("Invalid icon path: {}", path.display()))?;
    let ns_path = NSString::from_str(path_string);
    let image = NSImage::initWithContentsOfFile(NSImage::alloc(), &ns_path).ok_or_else(|| {
        format!("Failed to load application icon from {}", path.display())
    })?;

    let app = NSApplication::sharedApplication(mtm);
    unsafe {
        app.setApplicationIconImage(Some(&image));
    }

    Ok(())
}

fn resolve_png_path<R: Runtime>(app: &AppHandle<R>, icon_id: u8) -> Result<PathBuf, String> {
    let relative = png_relative_path(icon_id);
    resolve_bundled_path(app, &relative, "PNG")
}

#[cfg(target_os = "macos")]
fn resolve_icns_path<R: Runtime>(app: &AppHandle<R>, icon_id: u8) -> Result<PathBuf, String> {
    let relative = icns_relative_path(icon_id);
    resolve_bundled_path(app, &relative, "ICNS")
}

fn png_relative_path(icon_id: u8) -> PathBuf {
    if icon_id == 1 {
        PathBuf::from("icons/icon.png")
    } else {
        PathBuf::from(format!("icons/variants/{icon_id}.png"))
    }
}

#[cfg(target_os = "macos")]
fn icns_relative_path(icon_id: u8) -> PathBuf {
    if icon_id == 1 {
        PathBuf::from("icons/icon.icns")
    } else {
        PathBuf::from(format!("icons/variants/{icon_id}.icns"))
    }
}

fn resolve_bundled_path<R: Runtime>(
    app: &AppHandle<R>,
    relative: &Path,
    label: &str,
) -> Result<PathBuf, String> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let bundled = resource_dir.join(relative);
        if bundled.is_file() {
            return Ok(bundled);
        }
    }

    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    if dev_path.is_file() {
        return Ok(dev_path);
    }

    Err(format!("{label} asset not found: {}", relative.display()))
}
