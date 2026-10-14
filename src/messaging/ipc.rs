use crate::messaging::CrashCallback;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpcCrashMessage {
    pub app_name: Option<String>,
}

#[cfg(unix)]
mod unix_ipc {
    use super::*;
    use std::fs;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;

    fn socket_path() -> PathBuf {
        if let Some(custom) = std::env::var_os("RNAZAR_SOCKET_PATH") {
            return PathBuf::from(custom);
        }
        if let Some(runtime_dir) = dirs::runtime_dir() {
            return runtime_dir.join("rnazar.sock");
        }
        let uid = unsafe { libc::getuid() };
        let tmp = std::env::var_os("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        tmp.join(format!("rnazar-{uid}.sock"))
    }

    pub fn start_ipc_server(cb: CrashCallback) {
        let path = socket_path();
        let _ = fs::remove_file(&path);

        match UnixListener::bind(&path) {
            Ok(listener) => {
                let _ = std::thread::Builder::new()
                    .name("rnazar-ipc".into())
                    .spawn(move || {
                        for stream in listener.incoming().flatten() {
                            let cb = cb.clone();
                            std::thread::spawn(move || {
                                let reader = BufReader::new(stream);
                                for line in reader.lines().map_while(Result::ok) {
                                    if let Ok(msg) = serde_json::from_str::<IpcCrashMessage>(&line)
                                    {
                                        cb(msg.app_name);
                                    }
                                }
                            });
                        }
                    });
            }
            Err(e) => {
                log::warn!("Failed to bind Unix IPC socket at {:?}: {}", path, e);
            }
        }
    }

    pub fn send_ipc_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let path = socket_path();
        let mut stream = UnixStream::connect(&path).or_else(|orig_err| {
            let uid = unsafe { libc::getuid() };
            let fallback = PathBuf::from(format!("/tmp/rnazar-{uid}.sock"));
            if fallback != path {
                UnixStream::connect(fallback)
            } else {
                Err(orig_err)
            }
        })?;
        let msg = IpcCrashMessage {
            app_name: app_name.map(|s| s.to_string()),
        };
        let mut json = serde_json::to_string(&msg)?;
        json.push('\n');
        stream.write_all(json.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

#[cfg(windows)]
mod windows_ipc {
    use super::*;
    use interprocess::local_socket::{
        traits::{ListenerExt as _, Stream as _},
        GenericNamespaced, ListenerOptions, Stream, ToNsName,
    };

    fn pipe_name() -> String {
        if let Some(custom) = std::env::var_os("RNAZAR_SOCKET_PATH") {
            let path = std::path::PathBuf::from(custom);
            if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                return fname.to_string();
            }
        }
        "rnazar-messaging.sock".to_string()
    }

    pub fn start_ipc_server(cb: CrashCallback) {
        let _ = std::thread::Builder::new()
            .name("rnazar-ipc".into())
            .spawn(move || {
                let name = match pipe_name().to_ns_name::<GenericNamespaced>() {
                    Ok(n) => n,
                    Err(e) => {
                        log::warn!("Invalid Windows pipe name: {}", e);
                        return;
                    }
                };

                let listener = match ListenerOptions::new().name(name).create_sync() {
                    Ok(l) => l,
                    Err(e) => {
                        log::warn!("Failed to bind Windows IPC pipe: {}", e);
                        return;
                    }
                };

                for conn in listener.incoming().flatten() {
                    let cb = cb.clone();
                    std::thread::spawn(move || {
                        let reader = BufReader::new(conn);
                        for line in reader.lines().map_while(Result::ok) {
                            if let Ok(msg) = serde_json::from_str::<IpcCrashMessage>(&line) {
                                cb(msg.app_name);
                            }
                        }
                    });
                }
            });
    }

    pub fn send_ipc_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let name = pipe_name().to_ns_name::<GenericNamespaced>()?;
        let mut conn = Stream::connect(name)?;
        let msg = IpcCrashMessage {
            app_name: app_name.map(|s| s.to_string()),
        };
        let mut json = serde_json::to_string(&msg)?;
        json.push('\n');
        conn.write_all(json.as_bytes())?;
        conn.flush()?;
        Ok(())
    }
}

pub fn start_ipc_listener(cb: CrashCallback) {
    #[cfg(unix)]
    unix_ipc::start_ipc_server(cb);

    #[cfg(windows)]
    windows_ipc::start_ipc_server(cb);
}

pub fn trigger_ipc_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    return unix_ipc::send_ipc_crash(app_name);

    #[cfg(windows)]
    return windows_ipc::send_ipc_crash(app_name);

    #[cfg(not(any(unix, windows)))]
    Err("Unsupported platform for IPC".into())
}
