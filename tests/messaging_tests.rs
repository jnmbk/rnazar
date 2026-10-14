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
        protection: true,
    };
    let json = serde_json::to_string(&config).expect("serialization failed");
    let deserialized: AppConfig = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.language, Language::Turkish);
    assert!(deserialized.protection);

    // Backward compatibility: default protection to true when omitted
    let json_legacy = r#"{"language": "turkish"}"#;
    let legacy_cfg: AppConfig = serde_json::from_str(json_legacy).expect("parse legacy config");
    assert_eq!(legacy_cfg.language, Language::Turkish);
    assert!(legacy_cfg.protection);

    // Explicitly disabled protection
    let json_disabled = r#"{"language": "english", "protection": false}"#;
    let disabled_cfg: AppConfig =
        serde_json::from_str(json_disabled).expect("parse disabled protection config");
    assert_eq!(disabled_cfg.language, Language::English);
    assert!(!disabled_cfg.protection);
}

#[test]
fn test_crash_monitor_controller_start_stop() {
    use rnazar::messaging::monitor::CrashMonitorController;
    use std::sync::Arc;

    let controller = CrashMonitorController::new(Arc::new(|_| {}));
    assert!(!controller.is_running());

    controller.start();
    assert!(controller.is_running());

    // Idempotent start
    controller.start();
    assert!(controller.is_running());

    controller.stop();
    assert!(!controller.is_running());

    // Idempotent stop
    controller.stop();
    assert!(!controller.is_running());

    // Can be started again after stop
    controller.start();
    assert!(controller.is_running());

    controller.stop();
    assert!(!controller.is_running());
}

#[test]
fn test_tray_icons_colored_and_grayscale() {
    use rnazar::tray::load_tray_icon;

    let icon_colored = load_tray_icon(true);
    assert!(icon_colored.is_ok(), "failed to load colored tray icon");

    let icon_gray = load_tray_icon(false);
    assert!(icon_gray.is_ok(), "failed to load grayscale tray icon");
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

    // macOS DiagnosticReports (.ips and .crash), including hyphenated app names
    assert_eq!(
        parse_crash_filename("Safari-2026-10-12-210000.ips"),
        Some("Safari".to_string())
    );
    assert_eq!(
        parse_crash_filename("obs-studio-2026-10-12-210000.ips"),
        Some("obs-studio".to_string())
    );
    assert_eq!(
        parse_crash_filename("iTerm2_2026-10-12-210000_MacBook.crash"),
        Some("iTerm2".to_string())
    );
    // Ignore hidden in-progress dotfiles and non-crash macOS logs
    assert_eq!(parse_crash_filename(".Safari-2026-10-12-210000.ips"), None);
    assert_eq!(
        parse_crash_filename("JetsamEvent-2026-10-12-210000.ips"),
        None
    );

    // Windows LocalDumps (.dmp / .mdmp) and WER ReportArchive folders (AppCrash_...)
    assert_eq!(
        parse_crash_filename("notepad.exe.4812.dmp"),
        Some("notepad".to_string())
    );
    assert_eq!(
        parse_crash_filename("my.cool.app.exe.9001.mdmp"),
        Some("my.cool.app".to_string())
    );
    assert_eq!(
        parse_crash_filename("AppCrash_calculator.exe_9f8e7d6c_00000000_12345678"),
        Some("calculator".to_string())
    );
    assert_eq!(
        parse_crash_filename("AppCrash_my_game.exe_11223344_00000000_87654321"),
        Some("my_game".to_string())
    );
    assert_eq!(
        parse_crash_filename("AppHang_explorer.exe_11223344_00000000"),
        None
    );

    // Ubuntu/Debian Apport
    assert_eq!(
        parse_crash_filename("_usr_bin_vlc.1000.crash"),
        Some("vlc".to_string())
    );

    // Ignore rnazar itself and crash daemons
    assert_eq!(parse_crash_filename("rnazar-2026-10-12-210000.ips"), None);
    assert_eq!(
        parse_crash_filename("AppCrash_rnazar.exe_12345678_00000000"),
        None
    );
    assert_eq!(parse_crash_filename("WerFault.exe.1234.dmp"), None);
}

#[test]
fn test_parse_macos_crash_log_and_ips_header() {
    use rnazar::messaging::monitor::{parse_macos_crash_log, parse_macos_ips_header};

    // macOS unified log NDJSON from ReportCrash
    let ndjson_corpse = r#"{"eventMessage":"Formulating fatal 309 report for corpse[1959] UnCrackable Level 1","processImagePath":"/System/Library/CoreServices/ReportCrash"}"#;
    assert_eq!(
        parse_macos_crash_log(ndjson_corpse),
        Some("UnCrackable Level 1".to_string())
    );

    let ndjson_saved = r#"{"eventMessage":"client log create type 309 result success: /Users/test/Library/Logs/DiagnosticReports/Firefox-2026-10-12-210000.ips"}"#;
    assert_eq!(
        parse_macos_crash_log(ndjson_saved),
        Some("Firefox".to_string())
    );

    // Path with spaces in application name
    let ndjson_spaces = r#"{"eventMessage":"client log create type 309 result success: /Users/test/Library/Logs/DiagnosticReports/Brave Browser-2026-10-12-210000.ips"}"#;
    assert_eq!(
        parse_macos_crash_log(ndjson_spaces),
        Some("Brave Browser".to_string())
    );

    // Formulating crash report for process ...
    let proc_log = r#"{"eventMessage":"Formulating crash report for process Safari [1234]"}"#;
    assert_eq!(parse_macos_crash_log(proc_log), Some("Safari".to_string()));

    let legacy_saved =
        "Saved crash report for Pages[4120] version 14.0 to /Users/test/Library/Logs/DiagnosticReports/Pages_2026-10-12.crash";
    assert_eq!(
        parse_macos_crash_log(legacy_saved),
        Some("Pages".to_string())
    );

    let self_log = r#"{"eventMessage":"Formulating fatal 309 report for corpse[999] rnazar"}"#;
    assert_eq!(parse_macos_crash_log(self_log), None);

    // macOS .ips JSON header (bug_type 309 = crash, bug_type 202 = spin/non-crash)
    let crash_ips = "{\"app_name\":\"Blender\",\"bug_type\":\"309\",\"name\":\"Blender\"}\n{\"procName\":\"Blender\"}";
    assert_eq!(
        parse_macos_ips_header(crash_ips),
        Some("Blender".to_string())
    );

    // Support numeric bug_type (e.g. integer 309)
    let crash_ips_numeric = "{\"app_name\":\"Blender\",\"bug_type\":309,\"name\":\"Blender\"}\n{\"procName\":\"Blender\"}";
    assert_eq!(
        parse_macos_ips_header(crash_ips_numeric),
        Some("Blender".to_string())
    );

    let spin_ips = "{\"app_name\":\"Blender\",\"bug_type\":\"202\",\"name\":\"Blender\"}\n{}";
    assert_eq!(parse_macos_ips_header(spin_ips), None);

    let spin_ips_numeric = "{\"app_name\":\"Blender\",\"bug_type\":202,\"name\":\"Blender\"}\n{}";
    assert_eq!(parse_macos_ips_header(spin_ips_numeric), None);
}

#[test]
fn test_parse_windows_event_xml_and_wer_report() {
    use rnazar::messaging::monitor::{
        decode_wer_bytes, parse_windows_event_xml, parse_windows_wer_report,
    };

    // Windows 11 Event ID 1000 XML with named Data attributes
    let win11_xml = r#"
    <Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>
      <System>
        <Provider Name='Application Error'/>
        <EventID>1000</EventID>
      </System>
      <EventData>
        <Data Name='AppName'>notepad.exe</Data>
        <Data Name='AppVersion'>10.0.22621.1</Data>
      </EventData>
    </Event>"#;
    assert_eq!(
        parse_windows_event_xml(win11_xml),
        Some("notepad".to_string())
    );

    // Windows 10 Event ID 1000 XML with positional Data elements
    let win10_xml = r#"
    <Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
      <System>
        <Provider Name="Application Error" />
        <EventID Qualifiers="0">1000</EventID>
      </System>
      <EventData>
        <Data>C:\Program Files\VideoLAN\VLC\vlc.exe</Data>
        <Data>3.0.20.0</Data>
      </EventData>
    </Event>"#;
    assert_eq!(parse_windows_event_xml(win10_xml), Some("vlc".to_string()));

    // Windows 11 Event ID 1000 XML with AppPath fallback
    let win11_apppath_xml = r#"
    <Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>
      <System>
        <Provider Name='Application Error'/>
        <EventID>1000</EventID>
      </System>
      <EventData>
        <Data Name='AppPath'>C:\Program Files\WindowsApps\GIMP\bin\pythonw.exe</Data>
      </EventData>
    </Event>"#;
    assert_eq!(
        parse_windows_event_xml(win11_apppath_xml),
        Some("pythonw".to_string())
    );

    // Windows Event ID 1001 (Windows Error Reporting) APPCRASH
    let win_wer_1001_xml = r#"
    <Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>
      <System>
        <Provider Name='Windows Error Reporting'/>
        <EventID>1001</EventID>
      </System>
      <EventData>
        <Data Name='EventName'>APPCRASH</Data>
        <Data Name='P1'>calculator.exe</Data>
      </EventData>
    </Event>"#;
    assert_eq!(
        parse_windows_event_xml(win_wer_1001_xml),
        Some("calculator".to_string())
    );

    // Windows Event ID 1001 non-crash should be ignored
    let win_wer_noncrash_xml = r#"
    <Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>
      <System>
        <Provider Name='Windows Error Reporting'/>
        <EventID>1001</EventID>
      </System>
      <EventData>
        <Data Name='EventName'>WindowsUpdateFailure</Data>
        <Data Name='P1'>update.exe</Data>
      </EventData>
    </Event>"#;
    assert_eq!(parse_windows_event_xml(win_wer_noncrash_xml), None);

    // Non-1000/1001 event ID should be ignored
    let other_event_xml = r#"<Event><System><EventID>1002</EventID></System><EventData><Data>notepad.exe</Data></EventData></Event>"#;
    assert_eq!(parse_windows_event_xml(other_event_xml), None);

    // Windows Report.wer content (UTF-16LE with BOM)
    let wer_text =
        "Version=1\r\nEventType=APPCRASH\r\nNsAppName=winword.exe\r\nFriendlyAppName=Word\r\n";
    let mut utf16le_bytes = vec![0xFF, 0xFE];
    for u in wer_text.encode_utf16() {
        utf16le_bytes.extend_from_slice(&u.to_le_bytes());
    }
    let decoded = decode_wer_bytes(&utf16le_bytes);
    assert_eq!(
        parse_windows_wer_report(&decoded),
        Some("winword".to_string())
    );

    // Non-crash WER report (e.g. AppHangB1) should be ignored
    let hang_wer = "Version=1\r\nEventType=AppHangB1\r\nNsAppName=winword.exe\r\n";
    assert_eq!(parse_windows_wer_report(hang_wer), None);
}

#[test]
fn test_crash_dir_scanner() {
    use rnazar::messaging::monitor::CrashDirScanner;
    use std::fs;

    let temp_dir =
        std::env::temp_dir().join(format!("rnazar-crash-scan-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("failed to create temp test dir");

    // Pre-existing crash report before scanner starts should not fire
    fs::write(
        temp_dir.join("OldApp-2026-10-11-120000.ips"),
        "{\"app_name\":\"OldApp\",\"bug_type\":\"309\"}\n{}",
    )
    .unwrap();

    let mut scanner = CrashDirScanner::new(vec![temp_dir.clone()]);
    assert!(scanner.poll_new_crashes().is_empty());

    // Hidden dotfile created while OSAnalytics is writing should be ignored until renamed
    let dot_ips = temp_dir.join(".Safari-2026-10-12-210000.ips");
    let final_ips = temp_dir.join("Safari-2026-10-12-210000.ips");
    fs::write(
        &dot_ips,
        "{\"app_name\":\"Safari\",\"bug_type\":\"309\"}\n{}",
    )
    .unwrap();
    assert!(scanner.poll_new_crashes().is_empty());

    fs::rename(&dot_ips, &final_ips).unwrap();
    assert_eq!(scanner.poll_new_crashes(), vec!["Safari".to_string()]);
    // Subsequent poll should not re-emit already seen file
    assert!(scanner.poll_new_crashes().is_empty());

    // Windows WER folder with Report.wer inside
    let wer_dir = temp_dir.join("AppCrash_mspaint.exe_123456");
    fs::create_dir_all(&wer_dir).unwrap();
    fs::write(
        wer_dir.join("Report.wer"),
        "Version=1\r\nEventType=BEX64\r\nNsAppName=mspaint.exe\r\n",
    )
    .unwrap();
    assert_eq!(scanner.poll_new_crashes(), vec!["mspaint".to_string()]);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
#[cfg(target_os = "windows")]
#[ignore = "requires live Windows Error Reporting service which is disabled or slow in CI environments"]
fn test_windows_live_crash_detection() {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping live crash detection in CI environment");
        return;
    }

    use rnazar::messaging::monitor::CrashMonitorController;
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::time::Duration;

    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let crasher_path = std::env::temp_dir().join(format!(
        "rnazar_crasher_{}_{}.exe",
        std::process::id(),
        unique_id
    ));
    if !crasher_path.exists() {
        let status = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    r#"Add-Type -TypeDefinition "using System; public class Crasher {{ public static void Main() {{ throw new Exception(); }} }}" -OutputAssembly "{}" -OutputType ConsoleApplication"#,
                    crasher_path.display()
                ),
            ])
            .status();
        if !status.is_ok_and(|s| s.success()) {
            return;
        }
    }

    let (tx, rx) = channel();
    let controller = CrashMonitorController::new(Arc::new(move |app| {
        let _ = tx.send(app);
    }));

    controller.start();

    let _ = std::process::Command::new(&crasher_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    let expected_name = format!("rnazar_crasher_{}_{}", std::process::id(), unique_id);
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let mut detected = false;

    while std::time::Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if let Ok(app) = rx.recv_timeout(remaining) {
            if app.as_deref() == Some(expected_name.as_str()) {
                detected = true;
                break;
            }
        } else {
            break;
        }
    }

    let _ = std::fs::remove_file(&crasher_path);
    controller.stop();

    assert!(
        detected,
        "Expected crash notification for {} within deadline",
        expected_name
    );
}

#[test]
fn test_normalize_app_name_edge_cases() {
    use rnazar::messaging::monitor::normalize_app_name;

    // Empty and whitespace
    assert_eq!(normalize_app_name(""), None);
    assert_eq!(normalize_app_name("   "), None);
    assert_eq!(normalize_app_name("\t\n"), None);

    // Quoted strings and paths
    assert_eq!(
        normalize_app_name(r#""calc.exe""#),
        Some("calc".to_string())
    );
    assert_eq!(normalize_app_name("'spotify'"), Some("spotify".to_string()));
    assert_eq!(
        normalize_app_name(r#"C:\Program Files\VLC\vlc.exe"#),
        Some("vlc".to_string())
    );
    assert_eq!(
        normalize_app_name("/usr/bin/gnome-terminal"),
        Some("gnome-terminal".to_string())
    );

    // Dots and invalid names
    assert_eq!(normalize_app_name(".hidden"), None);
    assert_eq!(normalize_app_name(".exe"), None);
    assert_eq!(normalize_app_name("/path/to/.dotapp"), None);

    // Ignored crash reporters and rnazar itself
    assert_eq!(normalize_app_name("rnazar"), None);
    assert_eq!(normalize_app_name("rnazar.exe"), None);
    assert_eq!(normalize_app_name("WerFault.exe"), None);
    assert_eq!(normalize_app_name("WERFAULT"), None);
    assert_eq!(normalize_app_name("reportcrash"), None);
    assert_eq!(normalize_app_name("wermgr.exe"), None);
}

#[test]
fn test_extract_crash_app_from_path() {
    use rnazar::messaging::monitor::extract_crash_app_from_path;
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("rnazar-extract-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    // Non-crash extension returns None
    let non_crash_file = temp_dir.join("readme.txt");
    assert_eq!(extract_crash_app_from_path(&non_crash_file), None);

    // .ips with JSON header
    let ips_path = temp_dir.join("App-2026-10-12-000000.ips");
    fs::write(
        &ips_path,
        "{\"app_name\":\"CustomApp\",\"bug_type\":\"309\"}\n{}",
    )
    .unwrap();
    assert_eq!(
        extract_crash_app_from_path(&ips_path),
        Some("CustomApp".to_string())
    );

    // .dmp filename fallback
    let dmp_path = temp_dir.join("testapp.exe.1234.dmp");
    fs::write(&dmp_path, b"dummy dump data").unwrap();
    assert_eq!(
        extract_crash_app_from_path(&dmp_path),
        Some("testapp".to_string())
    );

    // Report.wer file
    let wer_path = temp_dir.join("Report.wer");
    fs::write(
        &wer_path,
        "Version=1\r\nEventType=APPCRASH\r\nNsAppName=word.exe\r\n",
    )
    .unwrap();
    assert_eq!(
        extract_crash_app_from_path(&wer_path),
        Some("word".to_string())
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
