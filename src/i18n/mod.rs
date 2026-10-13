use gettext::Catalog;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

include!(concat!(env!("OUT_DIR"), "/locales_registry.rs"));

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Auto,
    #[serde(alias = "en")]
    English,
    #[serde(alias = "tr")]
    Turkish,
    #[serde(untagged)]
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedLanguage {
    English,
    Turkish,
    Other(String),
}

impl ResolvedLanguage {
    pub fn code(&self) -> &str {
        match self {
            ResolvedLanguage::English => "en",
            ResolvedLanguage::Turkish => "tr",
            ResolvedLanguage::Other(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AvailableLanguage {
    pub language: Language,
    pub code: &'static str,
    pub native_name: &'static str,
}

fn native_language_name(code: &str) -> &'static str {
    match code.to_lowercase().as_str() {
        "en" => "English",
        "tr" => "Türkçe",
        "de" => "Deutsch",
        "fr" => "Français",
        "es" => "Español",
        "it" => "Italiano",
        "pt" => "Português",
        "ru" => "Русский",
        "ja" => "日本語",
        "zh" | "zh_cn" | "zh-cn" => "简体中文",
        "zh_tw" | "zh-tw" => "繁體中文",
        "ar" => "العربية",
        "az" => "Azərbaycanca",
        "nl" => "Nederlands",
        "pl" => "Polski",
        "uk" => "Українська",
        _ => "Custom",
    }
}

pub fn available_languages() -> Vec<AvailableLanguage> {
    COMPILED_LOCALES
        .iter()
        .map(|&(code, _)| {
            let lang = match code {
                "en" => Language::English,
                "tr" => Language::Turkish,
                other => Language::Custom(other.to_string()),
            };
            AvailableLanguage {
                language: lang,
                code,
                native_name: native_language_name(code),
            }
        })
        .collect()
}

static EMPTY_CATALOG: OnceLock<Catalog> = OnceLock::new();
static CACHE: OnceLock<std::sync::RwLock<HashMap<String, &'static Catalog>>> = OnceLock::new();

fn get_catalog(lang: ResolvedLanguage) -> &'static Catalog {
    let empty = EMPTY_CATALOG.get_or_init(Catalog::empty);
    let lang_code = lang.code();

    let cache = CACHE.get_or_init(|| std::sync::RwLock::new(HashMap::new()));

    if let Ok(r) = cache.read() {
        if let Some(&cat) = r.get(lang_code) {
            return cat;
        }
    }

    if let Ok(mut w) = cache.write() {
        if let Some(&cat) = w.get(lang_code) {
            return cat;
        }
        if let Some(&(_, mo_bytes)) = COMPILED_LOCALES
            .iter()
            .find(|&&(code, _)| code == lang_code)
        {
            let cat: &'static Catalog = Box::leak(Box::new(
                Catalog::parse(mo_bytes).unwrap_or_else(|_| Catalog::empty()),
            ));
            w.insert(lang_code.to_string(), cat);
            return cat;
        }
    }

    empty
}

impl Language {
    pub fn get_system_locale_string() -> String {
        sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string())
    }

    pub fn detect_system_language() -> ResolvedLanguage {
        let loc = Self::get_system_locale_string().to_lowercase();
        for &(code, _) in COMPILED_LOCALES {
            if loc == code
                || loc.starts_with(&format!("{}-", code))
                || loc.starts_with(&format!("{}_", code))
            {
                return match code {
                    "en" => ResolvedLanguage::English,
                    "tr" => ResolvedLanguage::Turkish,
                    other => ResolvedLanguage::Other(other.to_string()),
                };
            }
        }
        ResolvedLanguage::English
    }

    pub fn resolve(&self) -> ResolvedLanguage {
        match self {
            Language::Auto => Self::detect_system_language(),
            Language::English => ResolvedLanguage::English,
            Language::Turkish => ResolvedLanguage::Turkish,
            Language::Custom(ref s) => match s.to_lowercase().as_str() {
                "en" | "english" => ResolvedLanguage::English,
                "tr" | "turkish" => ResolvedLanguage::Turkish,
                other => ResolvedLanguage::Other(other.to_string()),
            },
        }
    }

    /// Translate a msg_id using the gettext catalog for the resolved language
    pub fn gettext<'a>(&self, msg_id: &'a str) -> &'a str {
        get_catalog(self.resolve()).gettext(msg_id)
    }

    /// Shorthand translation method alias
    pub fn tr<'a>(&self, msg_id: &'a str) -> &'a str {
        self.gettext(msg_id)
    }

    /// Translate with named argument replacement e.g. {app}
    pub fn tr_fmt(&self, msg_id: &str, args: &[(&str, &str)]) -> String {
        let mut template = self.gettext(msg_id).to_string();
        for (k, v) in args {
            template = template.replace(&format!("{{{}}}", k), v);
        }
        template
    }

    pub fn menu_protection(&self) -> &str {
        self.gettext("Protection")
    }

    pub fn menu_language(&self) -> &str {
        self.gettext("Language")
    }

    pub fn menu_lang_auto(&self) -> String {
        let system_loc = Self::get_system_locale_string();
        format!("{} ({})", self.gettext("Auto"), system_loc)
    }

    pub fn menu_lang_en(&self) -> &str {
        self.gettext("English")
    }

    pub fn menu_lang_tr(&self) -> &str {
        self.gettext("Turkish")
    }

    pub fn menu_about(&self) -> &str {
        self.gettext("About rnazar")
    }

    pub fn menu_exit(&self) -> &str {
        self.gettext("Exit")
    }

    pub fn about_title(&self) -> &str {
        self.gettext("About rnazar")
    }

    pub fn about_body(&self) -> String {
        format!(
            "rnazar v{}\n{}",
            crate::about::APP_VERSION,
            self.gettext("A thorn in the heart, a stab in the eye.\nTakes care of your apps.")
        )
    }

    pub fn inspiration_info(&self) -> &str {
        self.gettext("Inspired by knazar from Pardus 1.0")
    }

    pub fn inspiration_section(&self) -> &str {
        self.gettext("Inspiration")
    }

    pub fn crash_title(&self) -> &str {
        self.gettext("Evil eye!")
    }

    pub fn crash_body(&self, app: Option<&str>) -> String {
        match app {
            Some(app_name) if !app_name.trim().is_empty() => {
                let trimmed = app_name.trim();
                let template = self.gettext("An evil eye struck {app} and it was taken care of.");
                if template.contains("{app}") {
                    template.replace("{app}", trimmed)
                } else {
                    self.gettext("An evil eye struck {} and it was taken care of.")
                        .replace("{}", trimmed)
                }
            }
            _ => self
                .gettext("An evil eye struck and it was taken care of.")
                .to_string(),
        }
    }

    pub fn exit_confirm_title(&self) -> &str {
        "rnazar"
    }

    pub fn exit_confirm_message(&self) -> &str {
        self.gettext("Are you sure? Your computer will be vulnerable to the evil eye.")
    }
}
