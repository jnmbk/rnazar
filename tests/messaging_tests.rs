use rnazar::config::AppConfig;
use rnazar::i18n::Language;
use rnazar::messaging::ipc::IpcCrashMessage;

#[test]
fn test_ipc_crash_message_serialization() {
    let msg = IpcCrashMessage {
        app_name: Some("test-app".to_string()),
    };
    let json = serde_json::to_string(&msg).expect("serialization failed");
    let deserialized: IpcCrashMessage =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.app_name.as_deref(), Some("test-app"));

    let empty_msg = IpcCrashMessage { app_name: None };
    let json_empty = serde_json::to_string(&empty_msg).expect("serialization failed");
    let deserialized_empty: IpcCrashMessage =
        serde_json::from_str(&json_empty).expect("deserialization failed");
    assert_eq!(deserialized_empty.app_name, None);
}

#[test]
fn test_app_config_serialization() {
    let config = AppConfig {
        language: Language::Turkish,
    };
    let json = serde_json::to_string(&config).expect("serialization failed");
    let deserialized: AppConfig = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.language, Language::Turkish);
}
