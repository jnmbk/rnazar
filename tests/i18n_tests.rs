use rnazar::i18n::{Language, ResolvedLanguage};

#[test]
fn test_crash_title_translations() {
    assert_eq!(Language::English.crash_title(), "Evil eye!");
    assert_eq!(Language::Turkish.crash_title(), "Nazar değdi!");
}

#[test]
fn test_turkish_crash_body_exact_match() {
    let lang = Language::Turkish;
    assert_eq!(lang.crash_body(None), "Nazar değdi ve icabına bakıldı.");
}

#[test]
fn test_turkish_crash_body_with_app() {
    let lang = Language::Turkish;
    assert_eq!(
        lang.crash_body(Some("firefox")),
        "firefox uygulamasına nazar değdi ve icabına bakıldı."
    );
}

#[test]
fn test_english_crash_body() {
    let lang = Language::English;
    assert_eq!(
        lang.crash_body(None),
        "An evil eye struck and it was taken care of."
    );
    assert_eq!(
        lang.crash_body(Some("firefox")),
        "An evil eye struck firefox and it was taken care of."
    );
}

#[test]
fn test_menu_strings_non_empty() {
    for lang in [Language::English, Language::Turkish, Language::Auto] {
        assert!(!lang.menu_language().is_empty());
        assert!(!lang.menu_about().is_empty());
        assert!(!lang.menu_exit().is_empty());
        assert!(!lang.about_title().is_empty());
        assert!(!lang.about_body().is_empty());
        assert!(!lang.crash_title().is_empty());
        assert!(!lang.menu_lang_auto().is_empty());
    }
}

#[test]
fn test_language_resolution() {
    assert_eq!(Language::English.resolve(), ResolvedLanguage::English);
    assert_eq!(Language::Turkish.resolve(), ResolvedLanguage::Turkish);
}

#[test]
fn test_about_metadata() {
    assert!(!rnazar::about::APP_VERSION.is_empty());
    assert!(!rnazar::about::CREATOR_INFO.is_empty());
    assert!(!rnazar::about::LICENSE_NAME.is_empty());
    assert!(!rnazar::about::LICENSE_TEXT.is_empty());
    assert!(!rnazar::about::WEBSITE_URL.is_empty());
    assert!(rnazar::about::CREATOR_INFO.contains("Uğur Çetin"));
    assert!(rnazar::about::LICENSE_TEXT.contains("MIT License"));
    assert!(rnazar::about::INSPIRATION_INFO.contains("knazar from Pardus 1.0"));
    assert!(Language::Turkish.about_body().contains("Elemtere fiş"));
    assert!(Language::Turkish
        .about_body()
        .contains(rnazar::about::APP_VERSION));
}
