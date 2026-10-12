pub const MENU_LANGUAGE: &str = "Language";
pub const MENU_LANG_AUTO: &str = "Auto";
pub const MENU_LANG_EN: &str = "English";
pub const MENU_LANG_TR: &str = "Turkish";
pub const MENU_ABOUT: &str = "About rnazar";
pub const MENU_EXIT: &str = "Exit";

pub const ABOUT_TITLE: &str = "About rnazar";
pub const ABOUT_BODY: &str =
    "rnazar v0.1.0\nThe evil eye ward for your system.\nProtects your apps from crashing.";

pub const CRASH_TITLE: &str = "Nazar Değdi!";
pub const CRASH_BODY: &str = "An evil eye struck and it was taken care of.";

pub fn crash_body_with_app(app: &str) -> String {
    format!("An evil eye struck {} and it was taken care of.", app)
}
