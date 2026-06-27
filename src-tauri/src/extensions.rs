const EMBEDDED_USERSCRIPT: &str = include_str!("../assets/p-stream.user.js");

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

pub fn injection_runner(enabled: bool) -> String {
    if !enabled {
        return String::new();
    }

    let source = userscript_source();
    let escaped = source
        .replace('\\', "\\\\")
        .replace('\r', "")
        .replace('\n', "\\n")
        .replace('"', "\\\"");

    format!(
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
    )
}

pub fn reset_injection_flag_script() -> &'static str {
    "window.__qstaremPStreamInjected = false;"
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
