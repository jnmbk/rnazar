use clap::Parser;
use rnazar::cli::{Cli, Commands};
use rnazar::config::AppConfig;
use rnazar::i18n::Language;
use rnazar::tray::{load_tray_icon, TrayMenuController};

#[test]
fn test_cli_parsing_default() {
    let cli = Cli::try_parse_from(["rnazar"]).expect("parse default args");
    assert_eq!(cli.command, None);
    assert_eq!(cli.crash, None);
}

#[test]
fn test_cli_parsing_crash_flag() {
    let cli = Cli::try_parse_from(["rnazar", "--crash", "firefox"]).expect("parse crash flag");
    assert_eq!(cli.crash, Some("firefox".to_string()));
    assert_eq!(cli.command, None);
}

#[test]
fn test_cli_parsing_notify_subcommand_without_app() {
    let cli = Cli::try_parse_from(["rnazar", "notify"]).expect("parse notify subcommand");
    assert_eq!(cli.command, Some(Commands::Notify { app: None }));
    assert_eq!(cli.crash, None);
}

#[test]
fn test_cli_parsing_notify_subcommand_with_app() {
    let cli = Cli::try_parse_from(["rnazar", "notify", "--app", "gimp"])
        .expect("parse notify with --app");
    assert_eq!(
        cli.command,
        Some(Commands::Notify {
            app: Some("gimp".to_string())
        })
    );

    let cli_short =
        Cli::try_parse_from(["rnazar", "notify", "-a", "vlc"]).expect("parse notify with -a");
    assert_eq!(
        cli_short.command,
        Some(Commands::Notify {
            app: Some("vlc".to_string())
        })
    );
}

#[test]
fn test_cli_parsing_invalid_argument() {
    let err = Cli::try_parse_from(["rnazar", "--unknown-flag"]);
    assert!(err.is_err());
}

#[test]
fn test_config_load_and_save_roundtrip() {
    let temp_dir = std::env::temp_dir().join(format!("rnazar-cfg-test-{}", std::process::id()));
    let cfg_path = temp_dir.join("subdir").join("config.json");

    // File doesn't exist yet -> load returns default
    let default_cfg = AppConfig::load_from_path(&cfg_path);
    assert_eq!(default_cfg.protection, true);

    // Save customized config
    let custom_cfg = AppConfig {
        language: Language::Turkish,
        protection: false,
    };
    custom_cfg.save_to_path(&cfg_path).expect("save config");
    assert!(cfg_path.exists());

    // Load back
    let loaded = AppConfig::load_from_path(&cfg_path);
    assert_eq!(loaded.language, Language::Turkish);
    assert_eq!(loaded.protection, false);

    // Corrupted file -> fallback to default
    std::fs::write(&cfg_path, "{ broken json content").expect("write broken json");
    let fallback = AppConfig::load_from_path(&cfg_path);
    assert_eq!(fallback.protection, true);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_config_path_format() {
    if let Some(path) = AppConfig::config_path() {
        assert!(path.ends_with(std::path::Path::new("rnazar").join("config.json")));
    }
}

#[test]
fn test_tray_menu_controller_lifecycle() {
    let controller = TrayMenuController::new(Language::Turkish, true);
    assert!(controller.is_ok(), "TrayMenuController creation succeeded");

    let ctrl = controller.unwrap();
    assert_eq!(ctrl.item_protection.is_checked(), true);

    // Switch protection to false
    ctrl.update_protection(false);
    assert_eq!(ctrl.item_protection.is_checked(), false);

    // Switch language to English
    ctrl.update_language(&Language::English);
    assert_eq!(
        ctrl.item_protection.text(),
        Language::English.menu_protection()
    );
    assert_eq!(ctrl.item_about.text(), Language::English.menu_about());
    assert_eq!(ctrl.item_exit.text(), Language::English.menu_exit());

    // Switch language to Auto
    ctrl.update_language(&Language::Auto);
    assert_eq!(ctrl.item_lang_auto.is_checked(), true);
}

#[test]
fn test_tray_icon_rendering() {
    let icon_on = load_tray_icon(true);
    assert!(icon_on.is_ok());

    let icon_off = load_tray_icon(false);
    assert!(icon_off.is_ok());
}
