use gettext::Catalog;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static TR_CATALOG: OnceLock<Catalog> = OnceLock::new();
static EN_CATALOG: OnceLock<Catalog> = OnceLock::new();

const TR_MO_BYTES: &[u8] = include_bytes!("../../po/tr.mo");
const EN_MO_BYTES: &[u8] = include_bytes!("../../po/en.mo");

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

fn get_catalog(lang: ResolvedLanguage) -> &'static Catalog {
    match lang {
        ResolvedLanguage::Turkish => TR_CATALOG
            .get_or_init(|| Catalog::parse(TR_MO_BYTES).unwrap_or_else(|_| Catalog::empty())),
        ResolvedLanguage::English => EN_CATALOG
            .get_or_init(|| Catalog::parse(EN_MO_BYTES).unwrap_or_else(|_| Catalog::empty())),
    }
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

    /// Translate a msg_id using the gettext catalog for the resolved language
    pub fn gettext<'a>(&self, msg_id: &'a str) -> &'a str {
        get_catalog(self.resolve()).gettext(msg_id)
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
                let template = self.gettext("An evil eye struck {} and it was taken care of.");
                template.replace("{}", app_name.trim())
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
