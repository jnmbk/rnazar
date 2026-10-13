use crate::messaging::CrashCallback;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const IGNORED_APPS: &[&str] = &[
    "rnazar",
    "reportcrash",
    "osanalyticshelper",
    "werfault",
    "werfaultsecure",
    "wermgr",
];

const NON_CRASH_MACOS_PREFIXES: &[&str] = &[
    "jetsamevent",
    "stacks",
    "microstackshots",
    "excuserfault",
    "sirisearchfeedback",
    "differentialprivacy",
    "systemmemoryreset",
];

fn strip_ascii_suffix_ignore_case<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
    let split_idx = s.len().checked_sub(suffix.len())?;
    let head = s.get(..split_idx)?;
    let tail = s.get(split_idx..)?;
    if !head.is_empty() && tail.eq_ignore_ascii_case(suffix) {
        Some(head)
    } else {
        None
    }
}

fn strip_ascii_prefix_ignore_case<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let head = s.get(..prefix.len())?;
    let tail = s.get(prefix.len()..)?;
    if head.eq_ignore_ascii_case(prefix) {
        Some(tail)
    } else {
        None
    }
}

fn find_ascii_ignore_case(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let bytes = haystack.as_bytes();
    let nbytes = needle.as_bytes();
    bytes
        .windows(nbytes.len())
        .position(|w| w.eq_ignore_ascii_case(nbytes))
}

/// Normalizes a raw application name or executable path into a clean display name,
/// filtering out `rnazar` itself and OS crash-reporting helper daemons.
pub fn normalize_app_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches(['"', '\'']);
    if trimmed.is_empty() {
        return None;
    }

    let base = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed).trim();

    if base.is_empty() || base.starts_with('.') {
        return None;
    }

    let stem = strip_ascii_suffix_ignore_case(base, ".exe")
        .unwrap_or(base)
        .trim();

    if stem.is_empty() || stem.starts_with('.') {
        return None;
    }

    if IGNORED_APPS.iter().any(|ig| stem.eq_ignore_ascii_case(ig)) {
        return None;
    }

    Some(stem.to_string())
}

/// Parses a single JSON line emitted by `journalctl --output=json` for `systemd-coredump`
/// (`MESSAGE_ID=fc2e22bc6ee647b6b90729ab34a250b1`) and returns the crashed application's name.
pub fn parse_systemd_coredump_json(line: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(line.trim()).ok()?;

    // Prefer COREDUMP_EXE basename since COREDUMP_COMM is truncated to 15 chars by the kernel
    let from_exe = val
        .get("COREDUMP_EXE")
        .and_then(|v| v.as_str())
        .and_then(normalize_app_name);

    let from_comm = val
        .get("COREDUMP_COMM")
        .and_then(|v| v.as_str())
        .and_then(normalize_app_name);

    from_exe.or(from_comm)
}

/// Strips a macOS crash report timestamp suffix such as `-2026-10-12-210000` or
/// `_2026-10-12-210000_MacBook` while preserving hyphens/underscores in the app name itself.
fn strip_macos_timestamp_suffix(stem: &str) -> &str {
    let bytes = stem.as_bytes();
    // Look for `[-_]YYYY-MM-DD` (11 bytes minimum: sep + 4 digits + '-' + 2 digits + '-' + 2 digits)
    if bytes.len() >= 12 {
        for i in 1..=(bytes.len() - 11) {
            let sep = bytes[i];
            if (sep == b'-' || sep == b'_')
                && bytes[i + 1..i + 5].iter().all(u8::is_ascii_digit)
                && bytes[i + 5] == b'-'
                && bytes[i + 6..i + 8].iter().all(u8::is_ascii_digit)
                && bytes[i + 8] == b'-'
                && bytes[i + 9..i + 11].iter().all(u8::is_ascii_digit)
            {
                if let Some(prefix) = stem.get(..i) {
                    return prefix;
                }
            }
        }
    }
    stem.split(['-', '_']).next().unwrap_or(stem)
}

/// Extracts an application name from a crash report filename or WER directory name
/// (e.g. `Firefox-2026-10-12.ips`, `notepad.exe.1234.dmp`, `AppCrash_notepad.exe_1234`,
/// or `_usr_bin_vlc.1000.crash`).
pub fn parse_crash_filename(filename: &str) -> Option<String> {
    let name = filename.trim();
    if name.is_empty() || name.starts_with('.') {
        return None;
    }

    // Apport format on Ubuntu/Debian: _usr_bin_appname.1000.crash
    if let Some(stem) = strip_ascii_suffix_ignore_case(name, ".crash") {
        if stem.starts_with('_') {
            let without_uid = stem
                .rsplit_once('.')
                .map(|(prefix, _)| prefix)
                .unwrap_or(stem);
            if let Some(bin) = without_uid.rsplit('_').next() {
                return normalize_app_name(bin);
            }
        }
    }

    // Windows WER ReportArchive / ReportQueue folder format: AppCrash_<exe_name>_<hash>_...
    if let Some(rest) = strip_ascii_prefix_ignore_case(name, "AppCrash_") {
        let raw_app = find_ascii_ignore_case(rest, ".exe_")
            .and_then(|idx| rest.get(..idx))
            .unwrap_or_else(|| rest.split('_').next().unwrap_or(rest));
        return normalize_app_name(raw_app);
    }

    // Windows LocalDumps format: app.exe.1234.dmp (or .hdmp / .mdmp)
    if let Some(stem) = strip_ascii_suffix_ignore_case(name, ".dmp")
        .or_else(|| strip_ascii_suffix_ignore_case(name, ".hdmp"))
        .or_else(|| strip_ascii_suffix_ignore_case(name, ".mdmp"))
    {
        let raw_app = find_ascii_ignore_case(stem, ".exe.")
            .and_then(|idx| stem.get(..idx))
            .or_else(|| strip_ascii_suffix_ignore_case(stem, ".exe"))
            .unwrap_or_else(|| stem.split('.').next().unwrap_or(stem));
        return normalize_app_name(raw_app);
    }

    // macOS DiagnosticReports format: AppName-YYYY-MM-DD-HHMMSS.ips or AppName_YYYY-...crash
    if let Some(stem) = strip_ascii_suffix_ignore_case(name, ".ips")
        .or_else(|| strip_ascii_suffix_ignore_case(name, ".crash"))
    {
        let app = strip_macos_timestamp_suffix(stem).trim();
        if NON_CRASH_MACOS_PREFIXES
            .iter()
            .any(|p| app.eq_ignore_ascii_case(p))
        {
            return None;
        }
        return normalize_app_name(app);
    }

    None
}

fn parse_macos_ips_header_inner(content: &str) -> Option<Option<String>> {
    let first_line = content.lines().map(str::trim).find(|l| !l.is_empty())?;
    let val: serde_json::Value = serde_json::from_str(first_line).ok()?;
    let obj = val.as_object()?;

    if let Some(bug_type) = obj.get("bug_type").and_then(|v| v.as_str()) {
        // In Apple OSAnalytics, bug_type 309 (macOS 12+) and 109 (legacy) denote app crashes
        if bug_type != "309" && bug_type != "109" {
            return Some(None);
        }
    }

    let app = obj
        .get("app_name")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("name").and_then(|v| v.as_str()))
        .or_else(|| obj.get("procName").and_then(|v| v.as_str()))
        .and_then(normalize_app_name);

    Some(app)
}

/// Parses the JSON metadata header (line 1) of a macOS `.ips` file and returns
/// the crashed application's name if it is a crash report (`bug_type` `"309"` or `"109"`).
pub fn parse_macos_ips_header(content: &str) -> Option<String> {
    parse_macos_ips_header_inner(content).flatten()
}

/// Parses a line emitted by macOS `log stream --style ndjson --predicate 'process == "ReportCrash"'`
/// (or plain text `ReportCrash` log output) and returns the crashed application's name.
pub fn parse_macos_crash_log(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let msg_owned: String;
    let msg: &str = if trimmed.starts_with('{') {
        let val: serde_json::Value = serde_json::from_str(trimmed).ok()?;
        msg_owned = val.get("eventMessage")?.as_str()?.to_string();
        &msg_owned
    } else {
        trimmed
    };

    // Pattern 1: "Formulating fatal 309 report for corpse[1959] UnCrackable Level 1"
    // or "Formulating fatal report for corpse[1234] Safari"
    if msg.contains("Formulating") {
        if let Some(corpse_idx) = msg.find("corpse[") {
            let after_corpse = &msg[corpse_idx + "corpse[".len()..];
            if let Some(close_bracket) = after_corpse.find(']') {
                let app_part = after_corpse[close_bracket + 1..].trim();
                if let Some(app) = normalize_app_name(app_part) {
                    return Some(app);
                }
            }
        }
    }

    // Pattern 2: "Saved crash report for AppName[1234] version ..."
    if let Some(idx) = msg.find("Saved crash report for ") {
        let rest = &msg[idx + "Saved crash report for ".len()..];
        let app_part = rest.split('[').next().unwrap_or(rest).trim();
        if let Some(app) = normalize_app_name(app_part) {
            return Some(app);
        }
    }

    // Pattern 3: OSAnalytics / ReportCrash saving a type 309 .ips or .crash report path
    if msg.contains("309") || msg.contains("crash report") {
        for token in msg.split_whitespace().rev() {
            let clean = token.trim_matches(['"', '\'', ',', ';', ')']);
            if clean.ends_with(".ips") || clean.ends_with(".crash") {
                let fname = clean.rsplit('/').next().unwrap_or(clean);
                if let Some(app) = parse_crash_filename(fname) {
                    return Some(app);
                }
            }
        }
    }

    None
}

fn unescape_xml_entities(input: &str) -> String {
    if !input.contains('&') {
        return input.to_string();
    }
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Parses a Windows Event Log XML fragment for Application Error (`EventID = 1000`)
/// and extracts the faulting application name.
pub fn parse_windows_event_xml(xml: &str) -> Option<String> {
    let trimmed = xml.trim();
    if trimmed.is_empty() {
        return None;
    }

    // If <EventID> tag is present, ensure it is EventID 1000 (Application Error)
    if let Some(eid_start) = find_ascii_ignore_case(trimmed, "<EventID") {
        let after_tag = &trimmed[eid_start..];
        let gt = after_tag.find('>')?;
        let rest = &after_tag[gt + 1..];
        let end_eid = find_ascii_ignore_case(rest, "</EventID>")?;
        let eid_text = rest[..end_eid].trim();
        if eid_text != "1000" {
            return None;
        }
    }

    let ed_start = find_ascii_ignore_case(trimmed, "<EventData")?;
    let after_ed = &trimmed[ed_start..];
    let ed_gt = after_ed.find('>')?;
    let ed_body_and_rest = &after_ed[ed_gt + 1..];
    let ed_end = find_ascii_ignore_case(ed_body_and_rest, "</EventData>")?;
    let ed_body = &ed_body_and_rest[..ed_end];

    // Prefer named <Data Name="AppName"> / <Data Name='AppName'> (Windows 11 / Server 2022+)
    for pattern in [
        "Name=\"AppName\"",
        "Name='AppName'",
        "Name=\"Application\"",
        "Name='Application'",
    ] {
        if let Some(attr_idx) = find_ascii_ignore_case(ed_body, pattern) {
            let after_attr = &ed_body[attr_idx + pattern.len()..];
            if let Some(gt_idx) = after_attr.find('>') {
                let val_rest = &after_attr[gt_idx + 1..];
                if let Some(close_idx) = find_ascii_ignore_case(val_rest, "</Data>") {
                    let raw = unescape_xml_entities(&val_rest[..close_idx]);
                    if let Some(app) = normalize_app_name(&raw) {
                        return Some(app);
                    }
                }
            }
        }
    }

    // Fallback to the first positional <Data>...</Data> element (Windows 10 and earlier)
    let first_data = find_ascii_ignore_case(ed_body, "<Data")?;
    let after_data = &ed_body[first_data + "<Data".len()..];
    let gt_idx = after_data.find('>')?;
    let val_rest = &after_data[gt_idx + 1..];
    let close_idx = find_ascii_ignore_case(val_rest, "</Data>")?;
    let raw = unescape_xml_entities(&val_rest[..close_idx]);
    normalize_app_name(&raw)
}

/// Decodes raw bytes from a Windows `Report.wer` file, supporting UTF-16LE (with or without BOM),
/// UTF-16BE, and UTF-8.
pub fn decode_wer_bytes(bytes: &[u8]) -> String {
    if let Some(stripped) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let u16s: Vec<u16> = stripped
            .as_chunks::<2>()
            .0
            .iter()
            .copied()
            .map(u16::from_le_bytes)
            .collect();
        return String::from_utf16_lossy(&u16s);
    }
    if let Some(stripped) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let u16s: Vec<u16> = stripped
            .as_chunks::<2>()
            .0
            .iter()
            .copied()
            .map(u16::from_be_bytes)
            .collect();
        return String::from_utf16_lossy(&u16s);
    }
    if bytes.len() >= 4 && bytes[1] == 0 && bytes[3] == 0 {
        let u16s: Vec<u16> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .copied()
            .map(u16::from_le_bytes)
            .collect();
        return String::from_utf16_lossy(&u16s);
    }
    String::from_utf8_lossy(bytes).into_owned()
}

/// Parses the decoded text of a Windows Error Reporting `Report.wer` file and extracts
/// the crashed application's name if `EventType` is a crash event.
pub fn parse_windows_wer_report(content: &str) -> Option<String> {
    let mut event_type: Option<&str> = None;
    let mut ns_app_name: Option<&str> = None;
    let mut app_path: Option<&str> = None;
    let mut app_name: Option<&str> = None;
    let mut friendly_app_name: Option<&str> = None;

    for line in content.lines() {
        let line = line.trim();
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let val = val.trim();
        if val.is_empty() {
            continue;
        }

        if key.eq_ignore_ascii_case("EventType") {
            event_type = Some(val);
        } else if key.eq_ignore_ascii_case("NsAppName") {
            ns_app_name = Some(val);
        } else if key.eq_ignore_ascii_case("AppPath") {
            app_path = Some(val);
        } else if key.eq_ignore_ascii_case("AppName") {
            app_name = Some(val);
        } else if key.eq_ignore_ascii_case("FriendlyAppName") {
            friendly_app_name = Some(val);
        }
    }

    if let Some(et) = event_type {
        const CRASH_EVENT_PREFIXES: &[&str] = &["appcrash", "bex", "moappcrash", "clr20r3"];
        let is_crash = CRASH_EVENT_PREFIXES
            .iter()
            .any(|p| strip_ascii_prefix_ignore_case(et, p).is_some());
        if !is_crash {
            return None;
        }
    }

    ns_app_name
        .and_then(normalize_app_name)
        .or_else(|| app_path.and_then(normalize_app_name))
        .or_else(|| app_name.and_then(normalize_app_name))
        .or_else(|| friendly_app_name.and_then(normalize_app_name))
}

/// Inspects a newly discovered crash report path (file or WER report subdirectory)
/// and extracts the crashed application's name.
pub fn extract_crash_app_from_path(path: &Path) -> Option<String> {
    let fname = path.file_name()?.to_str()?;
    if fname.starts_with('.') {
        return None;
    }

    // If it's a directory (e.g. Windows WER `ReportArchive/AppCrash_...`), check `Report.wer` inside
    if path.is_dir() {
        let wer_file = path.join("Report.wer");
        if let Ok(bytes) = std::fs::read(&wer_file) {
            let text = decode_wer_bytes(&bytes);
            if let Some(app) = parse_windows_wer_report(&text) {
                return Some(app);
            }
            // If Report.wer explicitly had a non-crash EventType (e.g. AppHangB1), do not fall back
            if text.lines().any(|l| {
                l.trim()
                    .split_once('=')
                    .is_some_and(|(k, _)| k.trim().eq_ignore_ascii_case("EventType"))
            }) {
                return None;
            }
        }
        return parse_crash_filename(fname);
    }

    // Direct `.wer` file
    if strip_ascii_suffix_ignore_case(fname, ".wer").is_some() {
        if let Ok(bytes) = std::fs::read(path) {
            let text = decode_wer_bytes(&bytes);
            return parse_windows_wer_report(&text);
        }
    }

    // macOS `.ips` file: read JSON header on line 1 if possible to verify `bug_type == 309`
    if strip_ascii_suffix_ignore_case(fname, ".ips").is_some() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(parsed) = parse_macos_ips_header_inner(&content) {
                return parsed;
            }
        }
    }

    parse_crash_filename(fname)
}

/// Stateful directory scanner that watches one or more crash report directories
/// and detects newly created crash reports without alerting on pre-existing files.
pub struct CrashDirScanner {
    dirs: Vec<PathBuf>,
    seen: HashSet<PathBuf>,
}

impl CrashDirScanner {
    pub fn new(dirs: Vec<PathBuf>) -> Self {
        let mut seen = HashSet::new();
        for dir in &dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|s| !s.starts_with('.'))
                    {
                        seen.insert(path);
                    }
                }
            }
        }
        Self { dirs, seen }
    }

    pub fn poll_new_crashes(&mut self) -> Vec<String> {
        let mut detected = Vec::new();
        for dir in &self.dirs {
            let Ok(entries) = std::fs::read_dir(dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(fname) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                // Skip temporary hidden dotfiles (e.g. `.AppName-YYYY-MM-DD.ips` while being written)
                if fname.starts_with('.') {
                    continue;
                }
                if self.seen.insert(path.clone()) {
                    if let Some(app_name) = extract_crash_app_from_path(&path) {
                        detected.push(app_name);
                    }
                }
            }
        }
        detected
    }
}

fn make_dedup_callback(cb: CrashCallback) -> CrashCallback {
    let last_crash: Mutex<Option<(String, Instant)>> = Mutex::new(None);
    Arc::new(move |app_opt: Option<String>| {
        if let Some(ref app_name) = app_opt {
            let now = Instant::now();
            if let Ok(mut guard) = last_crash.lock() {
                let is_duplicate = guard.as_ref().is_some_and(|(prev, ts)| {
                    prev.eq_ignore_ascii_case(app_name)
                        && now.duration_since(*ts) < Duration::from_secs(5)
                });
                if is_duplicate {
                    return;
                }
                *guard = Some((app_name.clone(), now));
            }
        }
        cb(app_opt);
    })
}

#[derive(Clone)]
pub struct CrashMonitorController {
    cb: CrashCallback,
    active: Arc<Mutex<Option<ActiveMonitors>>>,
}

impl CrashMonitorController {
    pub fn new(cb: CrashCallback) -> Self {
        Self {
            cb,
            active: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self) {
        let mut guard = match self.active.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if guard.is_some() {
            return;
        }
        let dedup_cb = make_dedup_callback(self.cb.clone());
        *guard = Some(ActiveMonitors::start(dedup_cb));
        log::debug!("Crash monitor started.");
    }

    pub fn stop(&self) {
        let mut guard = match self.active.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if let Some(active) = guard.take() {
            active.stop();
            log::debug!("Crash monitor stopped.");
        }
    }

    pub fn is_running(&self) -> bool {
        match self.active.lock() {
            Ok(g) => g.is_some(),
            Err(p) => p.into_inner().is_some(),
        }
    }
}

pub fn start_crash_monitor(cb: CrashCallback) -> CrashMonitorController {
    let controller = CrashMonitorController::new(cb);
    controller.start();
    controller
}

#[cfg(target_os = "linux")]
struct ActiveMonitors {
    running: Arc<std::sync::atomic::AtomicBool>,
    child_pid: Arc<Mutex<Option<u32>>>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

#[cfg(target_os = "linux")]
impl ActiveMonitors {
    fn start(cb: CrashCallback) -> Self {
        use std::io::{BufRead, BufReader};
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};
        use std::sync::atomic::{AtomicBool, Ordering};

        let running = Arc::new(AtomicBool::new(true));
        let child_pid = Arc::new(Mutex::new(None));

        let thread_running = Arc::clone(&running);
        let thread_child_pid = Arc::clone(&child_pid);

        let thread_handle = std::thread::Builder::new()
            .name("rnazar-crash-mon".into())
            .spawn(move || {
                let mut cmd = Command::new("journalctl");
                cmd.args([
                    "--follow",
                    "--lines=0",
                    "--output=json",
                    "--output-fields=COREDUMP_EXE,COREDUMP_COMM,COREDUMP_UID",
                    "MESSAGE_ID=fc2e22bc6ee647b6b90729ab34a250b1",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

                unsafe {
                    cmd.pre_exec(|| {
                        // Ensure journalctl child terminates automatically if rnazar exits
                        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
                        Ok(())
                    });
                }

                if !thread_running.load(Ordering::SeqCst) {
                    return;
                }

                match cmd.spawn() {
                    Ok(mut child) => {
                        let pid = child.id();
                        if let Ok(mut guard) = thread_child_pid.lock() {
                            *guard = Some(pid);
                        }
                        if !thread_running.load(Ordering::SeqCst) {
                            let _ = child.kill();
                            let _ = child.wait();
                            return;
                        }

                        log::debug!("Started systemd-coredump crash monitor via journalctl.");
                        if let Some(stdout) = child.stdout.take() {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines().map_while(Result::ok) {
                                if !thread_running.load(Ordering::SeqCst) {
                                    break;
                                }
                                if let Some(app_name) = parse_systemd_coredump_json(&line) {
                                    log::debug!("Detected Linux application crash: {}", app_name);
                                    cb(Some(app_name));
                                }
                            }
                        }
                        let _ = child.kill();
                        let _ = child.wait();
                        if let Ok(mut guard) = thread_child_pid.lock() {
                            *guard = None;
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to spawn journalctl for crash monitoring: {}", e);
                    }
                }
            })
            .ok();

        Self {
            running,
            child_pid,
            thread_handle,
        }
    }

    fn stop(mut self) {
        use std::sync::atomic::Ordering;

        self.running.store(false, Ordering::SeqCst);
        if let Ok(guard) = self.child_pid.lock() {
            if let Some(pid) = *guard {
                unsafe {
                    libc::kill(pid as libc::pid_t, libc::SIGTERM);
                }
            }
        }
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(target_os = "macos")]
struct ActiveMonitors {
    running: Arc<std::sync::atomic::AtomicBool>,
    log_child_pid: Arc<Mutex<Option<u32>>>,
    log_thread: Option<std::thread::JoinHandle<()>>,
    dir_stop_tx: Option<std::sync::mpsc::SyncSender<()>>,
    dir_thread: Option<std::thread::JoinHandle<()>>,
}

#[cfg(target_os = "macos")]
impl ActiveMonitors {
    fn start(cb: CrashCallback) -> Self {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        use std::sync::atomic::{AtomicBool, Ordering};

        let running = Arc::new(AtomicBool::new(true));
        let log_child_pid = Arc::new(Mutex::new(None));

        let thread_running = Arc::clone(&running);
        let thread_log_child_pid = Arc::clone(&log_child_pid);
        let log_cb = cb.clone();

        let log_thread = std::thread::Builder::new()
            .name("rnazar-macos-log".into())
            .spawn(move || {
                let mut cmd = Command::new("/usr/bin/log");
                cmd.args([
                    "stream",
                    "--style",
                    "ndjson",
                    "--predicate",
                    r#"process == "ReportCrash""#,
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

                let spawn_res = cmd.spawn().or_else(|_| {
                    Command::new("log")
                        .args([
                            "stream",
                            "--style",
                            "ndjson",
                            "--predicate",
                            r#"process == "ReportCrash""#,
                        ])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                });

                if !thread_running.load(Ordering::SeqCst) {
                    return;
                }

                match spawn_res {
                    Ok(mut child) => {
                        let pid = child.id();
                        if let Ok(mut guard) = thread_log_child_pid.lock() {
                            *guard = Some(pid);
                        }
                        if !thread_running.load(Ordering::SeqCst) {
                            let _ = child.kill();
                            let _ = child.wait();
                            return;
                        }

                        log::debug!("Started macOS ReportCrash unified log stream monitor.");
                        if let Some(stdout) = child.stdout.take() {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines().map_while(Result::ok) {
                                if !thread_running.load(Ordering::SeqCst) {
                                    break;
                                }
                                if let Some(app_name) = parse_macos_crash_log(&line) {
                                    log::debug!(
                                        "Detected macOS application crash via unified log: {}",
                                        app_name
                                    );
                                    log_cb(Some(app_name));
                                }
                            }
                        }
                        let _ = child.kill();
                        let _ = child.wait();
                        if let Ok(mut guard) = thread_log_child_pid.lock() {
                            *guard = None;
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to spawn macOS `log stream` for crash monitoring: {}",
                            e
                        );
                    }
                }
            })
            .ok();

        let watch_dirs = default_crash_watch_dirs();
        let (dir_stop_tx, dir_thread) = if watch_dirs.is_empty() {
            (None, None)
        } else {
            let (tx, rx) = std::sync::mpsc::sync_channel::<()>(0);
            let dir_running = Arc::clone(&running);
            let dir_cb = cb;
            let handle = std::thread::Builder::new()
                .name("rnazar-crash-dir".into())
                .spawn(move || {
                    let mut scanner = CrashDirScanner::new(watch_dirs);
                    loop {
                        match rx.recv_timeout(Duration::from_secs(2)) {
                            Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                                if !dir_running.load(Ordering::SeqCst) {
                                    break;
                                }
                                for app_name in scanner.poll_new_crashes() {
                                    if !dir_running.load(Ordering::SeqCst) {
                                        break;
                                    }
                                    log::debug!(
                                        "Detected application crash from diagnostic directory: {}",
                                        app_name
                                    );
                                    dir_cb(Some(app_name));
                                }
                            }
                        }
                    }
                })
                .ok();
            (Some(tx), handle)
        };

        Self {
            running,
            log_child_pid,
            log_thread,
            dir_stop_tx,
            dir_thread,
        }
    }

    fn stop(mut self) {
        use std::sync::atomic::Ordering;

        self.running.store(false, Ordering::SeqCst);
        if let Ok(guard) = self.log_child_pid.lock() {
            if let Some(pid) = *guard {
                unsafe {
                    libc::kill(pid as libc::pid_t, libc::SIGTERM);
                }
            }
        }
        if let Some(tx) = self.dir_stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.log_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.dir_thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(target_os = "windows")]
struct ActiveMonitors {
    running: Arc<std::sync::atomic::AtomicBool>,
    evt_stop_tx: Option<std::sync::mpsc::SyncSender<()>>,
    evt_thread: Option<std::thread::JoinHandle<()>>,
    dir_stop_tx: Option<std::sync::mpsc::SyncSender<()>>,
    dir_thread: Option<std::thread::JoinHandle<()>>,
}

#[cfg(target_os = "windows")]
impl ActiveMonitors {
    fn start(cb: CrashCallback) -> Self {
        use std::ffi::c_void;
        use std::ptr::null_mut;
        use std::sync::atomic::{AtomicBool, Ordering};

        type EvtHandle = *mut c_void;
        type EvtSubscribeCallback = unsafe extern "system" fn(
            action: u32,
            user_context: *mut c_void,
            event: EvtHandle,
        ) -> u32;

        const EVT_SUBSCRIBE_TO_FUTURE_EVENTS: u32 = 1;
        const EVT_SUBSCRIBE_ACTION_DELIVER: u32 = 1;
        const EVT_RENDER_EVENT_XML: u32 = 1;

        #[link(name = "wevtapi")]
        extern "system" {
            fn EvtSubscribe(
                session: EvtHandle,
                signal_event: *mut c_void,
                channel_path: *const u16,
                query: *const u16,
                bookmark: EvtHandle,
                context: *mut c_void,
                callback: EvtSubscribeCallback,
                flags: u32,
            ) -> EvtHandle;

            fn EvtRender(
                context: EvtHandle,
                fragment: EvtHandle,
                flags: u32,
                buffer_size: u32,
                buffer: *mut c_void,
                buffer_used: *mut u32,
                property_count: *mut u32,
            ) -> i32;

            fn EvtClose(handle: EvtHandle) -> i32;
        }

        struct WinEvtContext {
            cb: CrashCallback,
            running: Arc<AtomicBool>,
        }

        unsafe extern "system" fn event_callback(
            action: u32,
            user_context: *mut c_void,
            event: EvtHandle,
        ) -> u32 {
            if action != EVT_SUBSCRIBE_ACTION_DELIVER || user_context.is_null() || event.is_null() {
                return 0;
            }

            let ctx = &*user_context.cast::<WinEvtContext>();
            if !ctx.running.load(Ordering::SeqCst) {
                return 0;
            }

            let mut buffer_used: u32 = 0;
            let mut property_count: u32 = 0;
            let mut buf: Vec<u16> = vec![0u16; 4096];

            let ok = EvtRender(
                null_mut(),
                event,
                EVT_RENDER_EVENT_XML,
                (buf.len() * std::mem::size_of::<u16>()) as u32,
                buf.as_mut_ptr().cast::<c_void>(),
                &mut buffer_used,
                &mut property_count,
            );

            if ok == 0 && buffer_used > 0 && buffer_used <= 65536 {
                let needed_u16 = (buffer_used as usize).div_ceil(2);
                buf.resize(needed_u16, 0);
                let retry_ok = EvtRender(
                    null_mut(),
                    event,
                    EVT_RENDER_EVENT_XML,
                    (buf.len() * std::mem::size_of::<u16>()) as u32,
                    buf.as_mut_ptr().cast::<c_void>(),
                    &mut buffer_used,
                    &mut property_count,
                );
                if retry_ok == 0 {
                    return 0;
                }
            } else if ok == 0 {
                return 0;
            }

            let u16_len = (buffer_used as usize / 2).min(buf.len());
            let slice = &buf[..u16_len];
            let trimmed_slice = slice.strip_suffix(&[0]).unwrap_or(slice);
            let xml = String::from_utf16_lossy(trimmed_slice);

            if let Some(app_name) = parse_windows_event_xml(&xml) {
                if ctx.running.load(Ordering::SeqCst) {
                    log::debug!(
                        "Detected Windows application crash via Event Log (EventID 1000): {}",
                        app_name
                    );
                    (ctx.cb)(Some(app_name));
                }
            }

            0
        }

        let running = Arc::new(AtomicBool::new(true));

        let (evt_stop_tx, evt_stop_rx) = std::sync::mpsc::sync_channel::<()>(0);
        let evt_running = Arc::clone(&running);
        let evt_cb = cb.clone();

        let evt_thread = std::thread::Builder::new()
            .name("rnazar-win-evt".into())
            .spawn(move || {
                let channel: Vec<u16> = "Application\0".encode_utf16().collect();
                let query: Vec<u16> =
                    "*[System[Provider[@Name='Application Error'] and (EventID=1000)]]\0"
                        .encode_utf16()
                        .collect();

                let ctx_ptr = Box::into_raw(Box::new(WinEvtContext {
                    cb: evt_cb,
                    running: evt_running,
                }))
                .cast::<c_void>();

                let handle = unsafe {
                    EvtSubscribe(
                        null_mut(),
                        null_mut(),
                        channel.as_ptr(),
                        query.as_ptr(),
                        null_mut(),
                        ctx_ptr,
                        event_callback,
                        EVT_SUBSCRIBE_TO_FUTURE_EVENTS,
                    )
                };

                if handle.is_null() {
                    log::warn!(
                        "Failed to subscribe to Windows Application Event Log (EventID 1000)."
                    );
                    unsafe {
                        drop(Box::from_raw(ctx_ptr.cast::<WinEvtContext>()));
                    }
                    return;
                }

                log::debug!("Subscribed to Windows Application Event Log (EventID 1000).");
                let _ = evt_stop_rx.recv();

                unsafe {
                    EvtClose(handle);
                    drop(Box::from_raw(ctx_ptr.cast::<WinEvtContext>()));
                }
            })
            .ok();

        let watch_dirs = default_crash_watch_dirs();
        let (dir_stop_tx, dir_thread) = if watch_dirs.is_empty() {
            (None, None)
        } else {
            let (tx, rx) = std::sync::mpsc::sync_channel::<()>(0);
            let dir_running = Arc::clone(&running);
            let dir_cb = cb;
            let handle = std::thread::Builder::new()
                .name("rnazar-crash-dir".into())
                .spawn(move || {
                    let mut scanner = CrashDirScanner::new(watch_dirs);
                    loop {
                        match rx.recv_timeout(Duration::from_secs(2)) {
                            Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                                if !dir_running.load(Ordering::SeqCst) {
                                    break;
                                }
                                for app_name in scanner.poll_new_crashes() {
                                    if !dir_running.load(Ordering::SeqCst) {
                                        break;
                                    }
                                    log::debug!(
                                        "Detected application crash from diagnostic directory: {}",
                                        app_name
                                    );
                                    dir_cb(Some(app_name));
                                }
                            }
                        }
                    }
                })
                .ok();
            (Some(tx), handle)
        };

        Self {
            running,
            evt_stop_tx: Some(evt_stop_tx),
            evt_thread,
            dir_stop_tx,
            dir_thread,
        }
    }

    fn stop(mut self) {
        use std::sync::atomic::Ordering;

        self.running.store(false, Ordering::SeqCst);
        if let Some(tx) = self.evt_stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(tx) = self.dir_stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.evt_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.dir_thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "macos"),
    not(target_os = "windows")
))]
struct ActiveMonitors;

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "macos"),
    not(target_os = "windows")
))]
impl ActiveMonitors {
    fn start(_cb: CrashCallback) -> Self {
        ActiveMonitors
    }
    fn stop(self) {}
}

#[cfg(not(target_os = "linux"))]
fn default_crash_watch_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Library/Logs/DiagnosticReports"));
        }
        dirs.push(PathBuf::from("/Library/Logs/DiagnosticReports"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local_data) = dirs::data_local_dir() {
            dirs.push(local_data.join(r"Microsoft\Windows\WER\ReportArchive"));
            dirs.push(local_data.join(r"Microsoft\Windows\WER\ReportQueue"));
            dirs.push(local_data.join("CrashDumps"));
        }
        let program_data = std::env::var_os("ProgramData")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
        dirs.push(program_data.join(r"Microsoft\Windows\WER\ReportArchive"));
        dirs.push(program_data.join(r"Microsoft\Windows\WER\ReportQueue"));
    }

    dirs
}
