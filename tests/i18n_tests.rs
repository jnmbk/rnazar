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
        assert!(!lang.menu_protection().is_empty());
        assert!(!lang.tray_tooltip(true).is_empty());
        assert!(!lang.tray_tooltip(false).is_empty());
        assert!(!lang.menu_language().is_empty());
        assert!(!lang.menu_about().is_empty());
        assert!(!lang.menu_exit().is_empty());
        assert!(!lang.about_title().is_empty());
        assert!(!lang.about_body().is_empty());
        assert!(!lang.crash_title().is_empty());
        assert!(!lang.menu_lang_auto().is_empty());
        assert!(!lang.exit_confirm_title().is_empty());
        assert!(!lang.exit_confirm_message().is_empty());
        assert!(!lang.website_label().is_empty());
    }
}

#[test]
fn test_about_window_translations() {
    assert_eq!(Language::English.about_title(), "About rnazar");
    assert_eq!(Language::Turkish.about_title(), "rnazar Hakkında");
    assert_eq!(Language::English.website_label(), "rnazar on GitHub");
    assert_eq!(Language::Turkish.website_label(), "GitHub'da rnazar");
    assert_eq!(Language::English.inspiration_section(), "Inspiration");
    assert_eq!(Language::Turkish.inspiration_section(), "Esinlenme");
    assert_eq!(
        Language::English.inspiration_info(),
        "Inspired by knazar from Pardus 1.0"
    );
    assert_eq!(
        Language::Turkish.inspiration_info(),
        "Pardus 1.0 knazar uygulamasından esinlenilmiştir."
    );
}

#[test]
fn test_dialog_button_translations() {
    assert_eq!(Language::English.dialog_about(), "About");
    assert_eq!(Language::Turkish.dialog_about(), "Hakkında");
    assert_eq!(Language::English.dialog_credits(), "Credits");
    assert_eq!(Language::Turkish.dialog_credits(), "Hazırlayanlar");
    assert_eq!(Language::English.dialog_license(), "License");
    assert_eq!(Language::Turkish.dialog_license(), "Lisans");
    assert_eq!(Language::English.dialog_close(), "Close");
    assert_eq!(Language::Turkish.dialog_close(), "Kapat");
    assert_eq!(Language::English.dialog_yes(), "Yes");
    assert_eq!(Language::Turkish.dialog_yes(), "Evet");
    assert_eq!(Language::English.dialog_no(), "No");
    assert_eq!(Language::Turkish.dialog_no(), "Hayır");
}

#[cfg(target_os = "linux")]
#[test]
fn test_about_dialog_gtk_widgets() {
    use gtk::prelude::*;
    use rnazar::about::localize_about_dialog;

    if gtk::init().is_err() {
        return;
    }

    for (lang, expected_title, expected_tabs) in [
        (
            Language::Turkish,
            "rnazar Hakkında",
            ["Hakkında", "Hazırlayanlar", "Lisans"],
        ),
        (
            Language::English,
            "About rnazar",
            ["About", "Credits", "License"],
        ),
    ] {
        let dialog = gtk::AboutDialog::new();
        dialog.set_program_name("rnazar");
        dialog.set_title(lang.about_title());
        dialog.set_website_label(Some(lang.website_label()));
        dialog.set_authors(&["Test Author"]);
        dialog.set_license(Some("Test License"));
        localize_about_dialog(&dialog, &lang);

        assert_eq!(dialog.title().as_deref(), Some(expected_title));

        // Verify stack child titles
        fn verify_stack_titles(widget: &gtk::Widget, expected: &[&str]) {
            if let Ok(stack) = widget.clone().downcast::<gtk::Stack>() {
                for child in stack.children() {
                    match stack.child_name(&child).as_deref() {
                        Some("main") => {
                            assert_eq!(stack.child_title(&child).as_deref(), Some(expected[0]))
                        }
                        Some("credits") => {
                            assert_eq!(stack.child_title(&child).as_deref(), Some(expected[1]))
                        }
                        Some("license") => {
                            assert_eq!(stack.child_title(&child).as_deref(), Some(expected[2]))
                        }
                        _ => {}
                    }
                }
            }
            if let Ok(container) = widget.clone().downcast::<gtk::Container>() {
                for child in container.children() {
                    verify_stack_titles(&child, expected);
                }
            }
        }
        verify_stack_titles(dialog.upcast_ref(), &expected_tabs);

        // Verify buttons in stack switcher or action area
        let mut labels = Vec::new();
        fn collect_labels(widget: &gtk::Widget, labels: &mut Vec<String>) {
            if let Ok(btn) = widget.clone().downcast::<gtk::Button>() {
                if let Some(lbl) = btn.label() {
                    labels.push(lbl.to_string());
                }
            }
            if let Ok(container) = widget.clone().downcast::<gtk::Container>() {
                for child in container.children() {
                    collect_labels(&child, labels);
                }
            }
        }
        collect_labels(dialog.upcast_ref(), &mut labels);
        for expected_tab in expected_tabs {
            assert!(
                labels
                    .iter()
                    .any(|l| l.replace('_', "").contains(expected_tab)),
                "Expected tab '{}' in dialog labels: {:?}",
                expected_tab,
                labels
            );
        }
    }
}

#[test]
fn test_menu_protection_translations() {
    assert_eq!(Language::English.menu_protection(), "Protection");
    assert_eq!(Language::Turkish.menu_protection(), "Koruma");
}

#[test]
fn test_tray_tooltip_translations() {
    assert_eq!(
        Language::English.tray_tooltip(true),
        "rnazar (Protection: Enabled)"
    );
    assert_eq!(
        Language::English.tray_tooltip(false),
        "rnazar (Protection: Disabled)"
    );
    assert_eq!(
        Language::Turkish.tray_tooltip(true),
        "rnazar (Koruma: Etkin)"
    );
    assert_eq!(
        Language::Turkish.tray_tooltip(false),
        "rnazar (Koruma: Devre Dışı)"
    );
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
