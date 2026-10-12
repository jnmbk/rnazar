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
        assert!(!lang.exit_confirm_title().is_empty());
        assert!(!lang.exit_confirm_message().is_empty());
    }
}

#[test]
fn test_exit_confirm_message_translations() {
    assert_eq!(
        Language::Turkish.exit_confirm_message(),
        "Emin misiniz? Bilgisayarınız nazara karşı savunmasız kalacak."
    );
    assert_eq!(
        Language::English.exit_confirm_message(),
        "Are you sure? Your computer will be vulnerable to the evil eye."
    );
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

#[test]
fn test_tr_fmt_named_interpolation() {
    let lang = Language::Turkish;
    let formatted = lang.tr_fmt(
        "An evil eye struck {app} and it was taken care of.",
        &[("app", "test_process")],
    );
    assert_eq!(
        formatted,
        "test_process uygulamasına nazar değdi ve icabına bakıldı."
    );

    let en_formatted = Language::English.tr_fmt(
        "An evil eye struck {app} and it was taken care of.",
        &[("app", "test_process")],
    );
    assert_eq!(
        en_formatted,
        "An evil eye struck test_process and it was taken care of."
    );
}

#[test]
fn test_available_languages_metadata() {
    let langs = rnazar::i18n::available_languages();
    assert!(langs.len() >= 2);

    let en = langs
        .iter()
        .find(|l| l.code == "en")
        .expect("en locale exists");
    assert_eq!(en.native_name, "English");
    assert_eq!(en.language, Language::English);

    let tr = langs
        .iter()
        .find(|l| l.code == "tr")
        .expect("tr locale exists");
    assert_eq!(tr.native_name, "Türkçe");
    assert_eq!(tr.language, Language::Turkish);
}

#[test]
fn test_language_serde_backward_compatibility() {
    use rnazar::config::AppConfig;

    // Test legacy "turkish"
    let json_tr = r#"{"language": "turkish"}"#;
    let cfg_tr: AppConfig = serde_json::from_str(json_tr).expect("parse legacy turkish");
    assert_eq!(cfg_tr.language, Language::Turkish);

    // Test legacy "english"
    let json_en = r#"{"language": "english"}"#;
    let cfg_en: AppConfig = serde_json::from_str(json_en).expect("parse legacy english");
    assert_eq!(cfg_en.language, Language::English);

    // Test ISO-639 codes "tr" and "en"
    let json_tr_code = r#"{"language": "tr"}"#;
    let cfg_tr_code: AppConfig = serde_json::from_str(json_tr_code).expect("parse tr code");
    assert_eq!(cfg_tr_code.language, Language::Turkish);

    // Test auto
    let json_auto = r#"{"language": "auto"}"#;
    let cfg_auto: AppConfig = serde_json::from_str(json_auto).expect("parse auto");
    assert_eq!(cfg_auto.language, Language::Auto);

    // Test arbitrary locale e.g. "de"
    let json_de = r#"{"language": "de"}"#;
    let cfg_de: AppConfig = serde_json::from_str(json_de).expect("parse de");
    assert_eq!(cfg_de.language, Language::Custom("de".to_string()));
}
