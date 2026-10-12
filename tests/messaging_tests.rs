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

#[test]
fn test_ipc_roundtrip() {
    use rnazar::messaging::ipc::{start_ipc_listener, trigger_ipc_crash};
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::time::Duration;

    let test_socket = std::env::temp_dir().join(format!("rnazar-test-{}.sock", std::process::id()));
    std::env::set_var("RNAZAR_SOCKET_PATH", &test_socket);

    let (tx, rx) = channel();
    start_ipc_listener(Arc::new(move |app| {
        let _ = tx.send(app);
    }));

    std::thread::sleep(Duration::from_millis(100));

    let res = trigger_ipc_crash(Some("ipc-unit-test-app"));
    assert!(res.is_ok(), "trigger_ipc_crash failed: {:?}", res.err());

    let received = rx.recv_timeout(Duration::from_secs(3));
    assert_eq!(received.unwrap(), Some("ipc-unit-test-app".to_string()));

    std::env::remove_var("RNAZAR_SOCKET_PATH");
    let _ = std::fs::remove_file(test_socket);
}

#[test]
fn test_parse_systemd_coredump_json() {
    use rnazar::messaging::monitor::parse_systemd_coredump_json;

    // Prefers full binary name from COREDUMP_EXE over truncated 15-char COREDUMP_COMM
    let line = r#"{"COREDUMP_COMM":"gnome-calculato","COREDUMP_EXE":"/usr/bin/gnome-calculator","COREDUMP_UID":"1000"}"#;
    assert_eq!(
        parse_systemd_coredump_json(line),
        Some("gnome-calculator".to_string())
    );

    // Falls back to COREDUMP_COMM when COREDUMP_EXE is missing
    let comm_only = r#"{"COREDUMP_COMM":"firefox","COREDUMP_UID":"1000"}"#;
    assert_eq!(
        parse_systemd_coredump_json(comm_only),
        Some("firefox".to_string())
    );

    // Ignores rnazar itself
    let self_crash = r#"{"COREDUMP_COMM":"rnazar","COREDUMP_EXE":"/usr/bin/rnazar"}"#;
    assert_eq!(parse_systemd_coredump_json(self_crash), None);
}

#[test]
fn test_parse_crash_filename() {
    use rnazar::messaging::monitor::parse_crash_filename;

    assert_eq!(
        parse_crash_filename("Safari-2026-10-12-210000.ips"),
        Some("Safari".to_string())
    );
    assert_eq!(
        parse_crash_filename("notepad.exe.4812.dmp"),
        Some("notepad".to_string())
    );
    assert_eq!(
        parse_crash_filename("_usr_bin_vlc.1000.crash"),
        Some("vlc".to_string())
    );
    assert_eq!(parse_crash_filename("rnazar-2026-10-12.ips"), None);
}
