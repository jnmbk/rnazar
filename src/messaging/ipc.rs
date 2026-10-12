use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::Sender;

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
        let runtime_dir = dirs::runtime_dir()
            .or_else(|| std::env::var_os("TMPDIR").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        runtime_dir.join("rnazar.sock")
    }

    pub fn start_ipc_server(tx: Sender<Option<String>>) {
        let path = socket_path();
        let _ = fs::remove_file(&path);

        match UnixListener::bind(&path) {
            Ok(listener) => {
                std::thread::spawn(move || {
                    for stream in listener.incoming().flatten() {
                        let tx_clone = tx.clone();
                        std::thread::spawn(move || {
                            let reader = BufReader::new(stream);
                            for line in reader.lines().map_while(Result::ok) {
                                if let Ok(msg) = serde_json::from_str::<IpcCrashMessage>(&line) {
                                    let _ = tx_clone.send(msg.app_name);
                                }
                            }
                        });
                    }
                });
            }
            Err(e) => {
                log::warn!("Failed to bind Unix IPC socket: {}", e);
            }
        }
    }

    pub fn send_ipc_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let path = socket_path();
        let mut stream = UnixStream::connect(path)?;
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
        traits::ListenerExt as _,
        GenericNamespaced, ListenerOptions, Stream, ToNsName,
    };

    const PIPE_NAME: &str = "@rnazar-messaging.sock";

    pub fn start_ipc_server(tx: Sender<Option<String>>) {
        std::thread::spawn(move || {
            let name = match PIPE_NAME.to_ns_name::<GenericNamespaced>() {
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
                let tx_clone = tx.clone();
                std::thread::spawn(move || {
                    let reader = BufReader::new(conn);
                    for line in reader.lines().map_while(Result::ok) {
                        if let Ok(msg) = serde_json::from_str::<IpcCrashMessage>(&line) {
                            let _ = tx_clone.send(msg.app_name);
                        }
                    }
                });
            }
        });
    }

    pub fn send_ipc_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let name = PIPE_NAME.to_ns_name::<GenericNamespaced>()?;
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

pub fn start_ipc_listener(tx: Sender<Option<String>>) {
    #[cfg(unix)]
    unix_ipc::start_ipc_server(tx);

    #[cfg(windows)]
    windows_ipc::start_ipc_server(tx);
}

pub fn trigger_ipc_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    return unix_ipc::send_ipc_crash(app_name);

    #[cfg(windows)]
    return windows_ipc::send_ipc_crash(app_name);

    #[cfg(not(any(unix, windows)))]
    Err("Unsupported platform for IPC".into())
}
