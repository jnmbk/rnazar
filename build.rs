use rspolib::SaveAsMOFile;

fn main() {
    println!("cargo:rerun-if-changed=po");
    compile_po_files();

    let version = get_app_version();
    println!("cargo:rustc-env=APP_VERSION={version}");

    #[cfg(windows)]
    compile_windows_resources(&version);
}

fn get_app_version() -> String {
    std::env::var("RNAZAR_VERSION")
        .or_else(|_| std::env::var("GITHUB_REF_NAME"))
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            std::process::Command::new("git")
                .args(["describe", "--tags", "--always"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|v| !v.is_empty())
        })
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
        .trim_start_matches('v')
        .to_string()
}

fn compile_po_files() {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut locales = Vec::new();

    if let Ok(entries) = std::fs::read_dir("po") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("po") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let target_mo = out_dir.join(format!("{stem}.mo"));
                    if let Ok(po) = rspolib::pofile(path.to_str().unwrap_or_default()) {
                        po.save_as_mofile(target_mo.to_str().unwrap_or_default());
                        locales.push(stem.to_string());
                    }
                }
            }
        }
    }

    locales.sort_by(|a, b| {
        let rank = |code: &str| match code {
            "en" => 0,
            "tr" => 1,
            _ => 2,
        };
        (rank(a), a).cmp(&(rank(b), b))
    });

    let entries = locales
        .iter()
        .map(|code| {
            format!("(\"{code}\", include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{code}.mo\")))")
        })
        .collect::<Vec<_>>()
        .join(",\n    ");

    let content =
        format!("pub static COMPILED_LOCALES: &[(&str, &[u8])] = &[\n    {entries}\n];\n");
    let _ = std::fs::write(out_dir.join("locales_registry.rs"), content);
}

#[cfg(windows)]
fn compile_windows_resources(version: &str) {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icons/rnazar.ico");
    res.set("ProductName", "rnazar");
    res.set("ProductVersion", version);
    res.set("FileVersion", version);
    res.set(
        "FileDescription",
        "System tray evil eye ward and crash notifier",
    );
    res.set("LegalCopyright", "Copyright (c) 2026");
    if let Err(e) = res.compile() {
        eprintln!("Warning: Failed to compile Windows resources: {e}");
    }
}
