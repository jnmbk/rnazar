use crate::messaging::CrashCallback;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

const MAX_IPC_MESSAGE_BYTES: u64 = 4096;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpcCrashMessage {
    pub app_name: Option<String>,
}

#[cfg(unix)]
mod unix_ipc {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    fn current_uid() -> u32 {
        fs::metadata("/proc/self")
            .or_else(|_| {
                dirs::home_dir()
                    .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
                    .and_then(fs::metadata)
            })
            .map(|m| m.uid())
            .unwrap_or(0)
    }

    fn fallback_socket_dir(uid: u32) -> PathBuf {
        let tmp = std::env::var_os("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        tmp.join(format!("rnazar-{uid}"))
    }

    pub fn default_socket_path() -> PathBuf {
        if let Some(custom) = std::env::var_os("RNAZAR_SOCKET_PATH") {
            return PathBuf::from(custom);
        }
        if let Some(runtime_dir) = dirs::runtime_dir() {
            return runtime_dir.join("rnazar.sock");
        }
        let uid = current_uid();
        fallback_socket_dir(uid).join("rnazar.sock")
    }

    fn ensure_secure_socket_parent(path: &Path) -> std::io::Result<()> {
        let Some(parent) = path.parent() else {
            return Ok(());
        };
        let uid = current_uid();
        if parent == fallback_socket_dir(uid) {
            if !parent.exists() {
                let mut builder = fs::DirBuilder::new();
                builder.recursive(true).mode(0o700);
                builder.create(parent)?;
            }
            let meta = fs::symlink_metadata(parent)?;
            if !meta.file_type().is_dir() || meta.file_type().is_symlink() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "IPC socket directory is not a secure directory",
                ));
            }
            if uid != 0 && meta.uid() != uid {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "IPC socket directory is not owned by the current user",
                ));
            }
            let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
        }
        Ok(())
    }

    pub fn remove_stale_socket(path: &Path) {
        if let Ok(meta) = fs::symlink_metadata(path) {
            if meta.file_type().is_socket() {
                let _ = fs::remove_file(path);
            }
        }
    }

    pub fn start_ipc_server_at(path: PathBuf, cb: CrashCallback) {
        if let Err(e) = ensure_secure_socket_parent(&path) {
            log::warn!(
                "Refusing to prepare Unix IPC socket parent {:?}: {}",
                path,
                e
            );
            return;
        }
        remove_stale_socket(&path);

        match UnixListener::bind(&path) {
            Ok(listener) => {
                let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
                let _ = std::thread::Builder::new()
                    .name("rnazar-ipc".into())
                    .spawn(move || {
                        for stream in listener.incoming().flatten() {
                            let cb = cb.clone();
                            std::thread::spawn(move || {
                                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                                let reader = BufReader::new(stream.take(MAX_IPC_MESSAGE_BYTES));
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

    pub fn start_ipc_server(cb: CrashCallback) {
        start_ipc_server_at(default_socket_path(), cb);
    }

    pub fn send_ipc_crash_at(
        path: &Path,
        app_name: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = UnixStream::connect(path).or_else(|orig_err| {
            let uid = current_uid();
            let fallback = fallback_socket_dir(uid).join("rnazar.sock");
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

    pub fn send_ipc_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let path = default_socket_path();
        send_ipc_crash_at(&path, app_name)
    }

    pub fn cleanup_default_socket() {
        remove_stale_socket(&default_socket_path());
    }
}

#[cfg(windows)]
mod windows_ipc {
    use super::*;
    use interprocess::local_socket::{
        traits::{ListenerExt as _, Stream as _},
        GenericNamespaced, ListenerOptions, Stream, ToNsName,
    };

    fn pipe_name_from_path(custom: Option<&Path>) -> String {
        if let Some(p) = custom {
            if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                return fname.to_string();
            }
        }
        if let Some(custom_env) = std::env::var_os("RNAZAR_SOCKET_PATH") {
            let path = std::path::PathBuf::from(custom_env);
            if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                return fname.to_string();
            }
        }
        "rnazar-messaging.sock".to_string()
    }

    pub fn start_ipc_server_with_name(pipe: String, cb: CrashCallback) {
        let _ = std::thread::Builder::new()
            .name("rnazar-ipc".into())
            .spawn(move || {
                let name = match pipe.to_ns_name::<GenericNamespaced>() {
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
                        let reader = BufReader::new(conn.take(MAX_IPC_MESSAGE_BYTES));
                        for line in reader.lines().map_while(Result::ok) {
                            if let Ok(msg) = serde_json::from_str::<IpcCrashMessage>(&line) {
                                cb(msg.app_name);
                            }
                        }
                    });
                }
            });
    }

    pub fn start_ipc_server(cb: CrashCallback) {
        start_ipc_server_with_name(pipe_name_from_path(None), cb);
    }

    pub fn start_ipc_server_at(path: &Path, cb: CrashCallback) {
        start_ipc_server_with_name(pipe_name_from_path(Some(path)), cb);
    }

    pub fn send_ipc_crash_with_name(
        pipe: &str,
        app_name: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = pipe.to_ns_name::<GenericNamespaced>()?;
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

    pub fn send_ipc_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        send_ipc_crash_with_name(&pipe_name_from_path(None), app_name)
    }

    pub fn send_ipc_crash_at(
        path: &Path,
        app_name: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        send_ipc_crash_with_name(&pipe_name_from_path(Some(path)), app_name)
    }
}

pub fn start_ipc_listener(cb: CrashCallback) {
    #[cfg(unix)]
    unix_ipc::start_ipc_server(cb);

    #[cfg(windows)]
    windows_ipc::start_ipc_server(cb);
}

pub fn start_ipc_listener_with_path(path: &Path, cb: CrashCallback) {
    #[cfg(unix)]
    unix_ipc::start_ipc_server_at(path.to_path_buf(), cb);

    #[cfg(windows)]
    windows_ipc::start_ipc_server_at(path, cb);
}

pub fn trigger_ipc_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    return unix_ipc::send_ipc_crash(app_name);

    #[cfg(windows)]
    return windows_ipc::send_ipc_crash(app_name);

    #[cfg(not(any(unix, windows)))]
    Err("Unsupported platform for IPC".into())
}

pub fn trigger_ipc_crash_with_path(
    path: &Path,
    app_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    return unix_ipc::send_ipc_crash_at(path, app_name);

    #[cfg(windows)]
    return windows_ipc::send_ipc_crash_at(path, app_name);

    #[cfg(not(any(unix, windows)))]
    Err("Unsupported platform for IPC".into())
}

pub fn cleanup_ipc_socket() {
    #[cfg(unix)]
    unix_ipc::cleanup_default_socket();
}
