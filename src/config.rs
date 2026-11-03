use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: Language,
    #[serde(default = "default_true")]
    pub protection: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::default(),
            protection: true,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("rnazar");
            p.push("config.json");
            p
        })
    }

    pub fn load_from_path(path: &std::path::Path) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        log::warn!(
                            "Failed to parse config file at {:?}: {}. Using default config.",
                            path,
                            e
                        );
                    }
                },
                Err(e) => {
                    log::warn!(
                        "Failed to read config file at {:?}: {}. Using default config.",
                        path,
                        e
                    );
                }
            }
        }
        AppConfig::default()
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            Self::load_from_path(&path)
        } else {
            AppConfig::default()
        }
    }

    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            let _ = self.save_to_path(&path);
        }
    }
}
