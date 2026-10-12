use crate::messaging::CrashCallback;
use std::path::Path;
use std::time::{Duration, Instant};

/// Parses a single JSON line emitted by `journalctl --output=json` for `systemd-coredump`
/// (`MESSAGE_ID=fc2e22bc6ee647b6b90729ab34a250b1`) and returns the crashed application's name.
pub fn parse_systemd_coredump_json(line: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(line.trim()).ok()?;

    // Prefer COREDUMP_EXE basename since COREDUMP_COMM is truncated to 15 chars by the kernel
    let from_exe = val
        .get("COREDUMP_EXE")
        .and_then(|v| v.as_str())
        .and_then(|exe| Path::new(exe).file_name())
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let from_comm = val
        .get("COREDUMP_COMM")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let app_name = from_exe.or(from_comm)?;

    if app_name.eq_ignore_ascii_case("rnazar") {
        return None;
    }

    Some(app_name.to_string())
}

/// Extracts an application name from a crash report filename (e.g. `Firefox-2026-10-12.ips`,
/// `notepad.exe.1234.dmp`, or `_usr_bin_vlc.1000.crash`).
pub fn parse_crash_filename(filename: &str) -> Option<String> {
    let name = filename.trim();
    if name.is_empty() {
        return None;
    }

    // Apport format on Ubuntu/Debian: _usr_bin_appname.1000.crash
    if let Some(stem) = name.strip_suffix(".crash") {
        if stem.starts_with('_') {
            let without_uid = stem
                .rsplit_once('.')
                .map(|(prefix, _)| prefix)
                .unwrap_or(stem);
            if let Some(bin) = without_uid.rsplit('_').next() {
                if !bin.is_empty() && !bin.eq_ignore_ascii_case("rnazar") {
                    return Some(bin.to_string());
                }
            }
        }
    }

    // Windows WER format: app.exe.1234.dmp
    if let Some(stem) = name.strip_suffix(".dmp") {
        let app = stem.split('.').next().unwrap_or(stem).trim();
        if !app.is_empty() && !app.eq_ignore_ascii_case("rnazar") {
            return Some(app.to_string());
        }
    }

    // macOS DiagnosticReports format: AppName-YYYY-MM-DD-HHMMSS.ips or AppName_YYYY-...
    if let Some(stem) = name.strip_suffix(".ips").or_else(|| name.strip_suffix(".crash")) {
        let app = stem
            .split(['-', '_'])
            .next()
            .unwrap_or(stem)
            .trim();
        if !app.is_empty() && !app.eq_ignore_ascii_case("rnazar") {
            return Some(app.to_string());
        }
    }

    None
}

pub fn start_crash_monitor(cb: CrashCallback) {
    #[cfg(target_os = "linux")]
    start_linux_crash_monitor(cb);

    #[cfg(not(target_os = "linux"))]
    start_dir_crash_monitor(cb);
}

#[cfg(target_os = "linux")]
fn start_linux_crash_monitor(cb: CrashCallback) {
    use std::io::{BufRead, BufReader};
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};

    let _ = std::thread::Builder::new()
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

            match cmd.spawn() {
                Ok(mut child) => {
                    log::debug!("Started systemd-coredump crash monitor via journalctl.");
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        let mut last_crash: Option<(String, Instant)> = None;

                        for line in reader.lines().map_while(Result::ok) {
                            if let Some(app_name) = parse_systemd_coredump_json(&line) {
                                let now = Instant::now();
                                let is_duplicate = last_crash.as_ref().is_some_and(|(prev, ts)| {
                                    prev == &app_name && now.duration_since(*ts) < Duration::from_secs(2)
                                });

                                if !is_duplicate {
                                    log::debug!("Detected OS application crash: {}", app_name);
                                    last_crash = Some((app_name.clone(), now));
                                    cb(Some(app_name));
                                }
                            }
                        }
                    }
                    let _ = child.wait();
                }
                Err(e) => {
                    log::warn!("Failed to spawn journalctl for crash monitoring: {}", e);
                }
            }
        });
}

#[cfg(not(target_os = "linux"))]
fn start_dir_crash_monitor(cb: CrashCallback) {
    use std::collections::HashSet;
    use std::path::PathBuf;

    let watch_dir: Option<PathBuf> = {
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir().map(|h| h.join("Library/Logs/DiagnosticReports"))
        }
        #[cfg(target_os = "windows")]
        {
            dirs::data_local_dir().map(|d| d.join("CrashDumps"))
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            None
        }
    };

    let Some(dir) = watch_dir else {
        return;
    };

    let _ = std::thread::Builder::new()
        .name("rnazar-crash-mon".into())
        .spawn(move || {
            let mut seen: HashSet<String> = std::fs::read_dir(&dir)
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();

            loop {
                std::thread::sleep(Duration::from_secs(2));
                let Ok(entries) = std::fs::read_dir(&dir) else {
                    continue;
                };
                for entry in entries.flatten() {
                    let Ok(fname) = entry.file_name().into_string() else {
                        continue;
                    };
                    if seen.insert(fname.clone()) {
                        if let Some(app_name) = parse_crash_filename(&fname) {
                            cb(Some(app_name));
                        }
                    }
                }
            }
        });
}

