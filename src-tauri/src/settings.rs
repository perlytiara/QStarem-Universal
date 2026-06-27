use serde::{Deserialize, Serialize};

pub const DEFAULT_HOME_URL: &str = "https://zstream.mov";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdBlockerChoice {
    Ublock,
    Adguard,
    None,
}

impl Default for AdBlockerChoice {
    fn default() -> Self {
        Self::Ublock
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_home_url")]
    pub home_url: String,
    #[serde(default)]
    pub ad_blocker: AdBlockerChoice,
    #[serde(default = "default_true")]
    pub p_stream_enabled: bool,
}

fn default_home_url() -> String {
    DEFAULT_HOME_URL.to_string()
}

fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            home_url: DEFAULT_HOME_URL.to_string(),
            ad_blocker: AdBlockerChoice::Ublock,
            p_stream_enabled: true,
        }
    }
}

impl AppSettings {
    pub fn config_path() -> Option<std::path::PathBuf> {
        directories::ProjectDirs::from("com", "QStarem", "QStarem")
            .map(|dirs| dirs.config_dir().join("settings.toml"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };

        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(path) = Self::config_path() else {
            return Err("Could not resolve settings directory".into());
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let contents = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, contents).map_err(|e| e.to_string())
    }
}
