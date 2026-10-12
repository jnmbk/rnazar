use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub language: Language,
}

impl AppConfig {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("rnazar");
            p.push("config.json");
            p
        })
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        AppConfig::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, content);
            }
        }
    }
}
