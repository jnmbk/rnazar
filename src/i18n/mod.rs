pub mod en;
pub mod tr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Auto,
    English,
    Turkish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedLanguage {
    English,
    Turkish,
}

impl Language {
    pub fn get_system_locale_string() -> String {
        sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string())
    }

    pub fn detect_system_language() -> ResolvedLanguage {
        let loc = Self::get_system_locale_string().to_lowercase();
        if loc.starts_with("tr") {
            ResolvedLanguage::Turkish
        } else {
            ResolvedLanguage::English
        }
    }

    pub fn resolve(&self) -> ResolvedLanguage {
        match self {
            Language::Auto => Self::detect_system_language(),
            Language::English => ResolvedLanguage::English,
            Language::Turkish => ResolvedLanguage::Turkish,
        }
    }

    pub fn menu_language(&self) -> &'static str {
        match self.resolve() {
            ResolvedLanguage::English => en::MENU_LANGUAGE,
            ResolvedLanguage::Turkish => tr::MENU_LANGUAGE,
        }
    }

    pub fn menu_lang_auto(&self) -> String {
        let system_loc = Self::get_system_locale_string();
        match self.resolve() {
            ResolvedLanguage::English => format!("{} ({})", en::MENU_LANG_AUTO, system_loc),
            ResolvedLanguage::Turkish => format!("{} ({})", tr::MENU_LANG_AUTO, system_loc),
        }
    }

    pub fn menu_lang_en(&self) -> &'static str {
        match self.resolve() {
            ResolvedLanguage::English => en::MENU_LANG_EN,
            ResolvedLanguage::Turkish => tr::MENU_LANG_EN,
        }
    }

    pub fn menu_lang_tr(&self) -> &'static str {
        match self.resolve() {
            ResolvedLanguage::English => en::MENU_LANG_TR,
            ResolvedLanguage::Turkish => tr::MENU_LANG_TR,
        }
    }

    pub fn menu_about(&self) -> &'static str {
        match self.resolve() {
            ResolvedLanguage::English => en::MENU_ABOUT,
            ResolvedLanguage::Turkish => tr::MENU_ABOUT,
        }
    }

    pub fn menu_exit(&self) -> &'static str {
        match self.resolve() {
            ResolvedLanguage::English => en::MENU_EXIT,
            ResolvedLanguage::Turkish => tr::MENU_EXIT,
        }
    }

    pub fn about_title(&self) -> &'static str {
        match self.resolve() {
            ResolvedLanguage::English => en::ABOUT_TITLE,
            ResolvedLanguage::Turkish => tr::ABOUT_TITLE,
        }
    }

    pub fn about_body(&self) -> &'static str {
        match self.resolve() {
            ResolvedLanguage::English => en::ABOUT_BODY,
            ResolvedLanguage::Turkish => tr::ABOUT_BODY,
        }
    }

    pub fn crash_title(&self) -> &'static str {
        match self.resolve() {
            ResolvedLanguage::English => en::CRASH_TITLE,
            ResolvedLanguage::Turkish => tr::CRASH_TITLE,
        }
    }

    pub fn crash_body(&self, app: Option<&str>) -> String {
        match (self.resolve(), app) {
            (ResolvedLanguage::Turkish, Some(app_name)) if !app_name.trim().is_empty() => {
                tr::crash_body_with_app(app_name.trim())
            }
            (ResolvedLanguage::Turkish, _) => tr::CRASH_BODY.to_string(),
            (ResolvedLanguage::English, Some(app_name)) if !app_name.trim().is_empty() => {
                en::crash_body_with_app(app_name.trim())
            }
            (ResolvedLanguage::English, _) => en::CRASH_BODY.to_string(),
        }
    }
}
