const EMBEDDED_USERSCRIPT: &str = include_str!("../assets/p-stream.user.js");
const EMBEDDED_SHELL_CSS: &str = include_str!("../assets/shell.css");
const EMBEDDED_SHELL_JS: &str = include_str!("../assets/shell.js");

pub fn userscript_source() -> String {
    let cache_path = cached_userscript_path();
    if let Ok(contents) = std::fs::read_to_string(&cache_path) {
        if !contents.trim().is_empty() {
            return contents;
        }
    }

    EMBEDDED_USERSCRIPT.to_string()
}

pub fn cached_userscript_path() -> std::path::PathBuf {
    directories::ProjectDirs::from("com", "QStarem", "QStarem")
        .map(|dirs| dirs.cache_dir().join("p-stream.user.js"))
        .unwrap_or_else(|| std::path::PathBuf::from("p-stream.user.js"))
}

fn escape_for_js_literal(source: &str) -> String {
    source
        .replace('\\', "\\\\")
        .replace('\r', "")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
}

pub fn shell_injection_script() -> String {
    let css = escape_for_js_literal(EMBEDDED_SHELL_CSS);
    let js = escape_for_js_literal(EMBEDDED_SHELL_JS);

    format!(
        r#"(function() {{
  function qstaremInjectShell() {{
    if (window.__qstaremShellInjected) return;
    window.__qstaremShellInjected = true;
    try {{
      const css = "{css}";
      if (!document.getElementById("qstarem-shell-styles")) {{
        const style = document.createElement("style");
        style.id = "qstarem-shell-styles";
        style.textContent = css;
        (document.head || document.documentElement).appendChild(style);
      }}
      const source = "{js}";
      const el = document.createElement("script");
      el.textContent = source;
      (document.head || document.documentElement).appendChild(el);
    }} catch (error) {{
      console.error("[QStarem] shell injection failed", error);
    }}
  }}

  qstaremInjectShell();
  window.addEventListener("load", qstaremInjectShell);
  window.addEventListener("pageshow", qstaremInjectShell);
}})();"#
    )
}

pub fn injection_runner(p_stream_enabled: bool) -> String {
    let mut parts = vec![shell_injection_script()];

    if p_stream_enabled {
        let source = userscript_source();
        let escaped = escape_for_js_literal(&source);
        parts.push(format!(
            r#"(function() {{
  function qstaremInjectPStream() {{
    if (window.__qstaremPStreamInjected) return;
    window.__qstaremPStreamInjected = true;
    try {{
      const source = "{escaped}";
      const el = document.createElement("script");
      el.textContent = source;
      (document.head || document.documentElement).appendChild(el);
    }} catch (error) {{
      console.error("[QStarem] P-Stream injection failed", error);
    }}
  }}

  qstaremInjectPStream();
  window.addEventListener("load", qstaremInjectPStream);
  window.addEventListener("pageshow", qstaremInjectPStream);
}})();"#
        ));
    }

    parts.join("\n")
}

pub fn reset_injection_flag_script() -> &'static str {
    "window.__qstaremPStreamInjected = false; window.__qstaremShellInjected = false;"
}

pub fn refresh_cached_userscript() -> Result<(), String> {
    let url = "https://raw.githubusercontent.com/xp-technologies-dev/Userscript/main/p-stream.user.js";
    let mut response = ureq::get(url)
        .call()
        .map_err(|e| format!("Failed to download P-Stream userscript: {e}"))?;

    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;

    let path = cached_userscript_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, body).map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
pub fn apply_macos_window_theme(
    window: &tauri::WebviewWindow,
) -> Result<(), Box<dyn std::error::Error>> {
    use objc2_app_kit::{NSColor, NSWindow};

    let ns_window_ptr = window.ns_window()? as *mut NSWindow;
    let ns_window = unsafe { &*ns_window_ptr };
    let bg_color = NSColor::colorWithRed_green_blue_alpha(10.0 / 255.0, 10.0 / 255.0, 15.0 / 255.0, 1.0);
    ns_window.setBackgroundColor(Some(&bg_color));
    Ok(())
}
