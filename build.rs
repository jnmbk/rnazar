fn main() {
    println!("cargo:rerun-if-changed=po/tr.po");
    println!("cargo:rerun-if-changed=po/en.po");

    // Attempt to re-compile .po to .mo if msgfmt is installed
    compile_po_files();

    let version = get_app_version();
    println!("cargo:rustc-env=APP_VERSION={}", version);

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icons/rnazar.ico");
        res.set("ProductName", "rnazar");
        res.set("ProductVersion", &version);
        res.set("FileVersion", &version);
        res.set(
            "FileDescription",
            "System tray evil eye ward and crash notifier",
        );
        res.set("LegalCopyright", "Copyright (c) 2026");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}

fn get_app_version() -> String {
    // 1. Explicit RNAZAR_VERSION env var
    if let Ok(val) = std::env::var("RNAZAR_VERSION") {
        let clean = val.trim().trim_start_matches('v');
        if !clean.is_empty() {
            return clean.to_string();
        }
    }

    // 2. GitHub Actions tag name (e.g. GITHUB_REF_NAME=v1.0.0)
    if let Ok(val) = std::env::var("GITHUB_REF_NAME") {
        let clean = val.trim().trim_start_matches('v');
        if !clean.is_empty() {
            return clean.to_string();
        }
    }

    // 3. Git tag if in a git repository
    if let Ok(output) = std::process::Command::new("git")
        .args(["describe", "--tags", "--always"])
        .output()
    {
        if output.status.success() {
            let desc = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let clean = desc.trim_start_matches('v');
            if !clean.is_empty() {
                return clean.to_string();
            }
        }
    }

    // 4. Default to Cargo.toml package version
    env!("CARGO_PKG_VERSION").to_string()
}

fn compile_po_files() {
    let po_dir = std::path::Path::new("po");
    if let Ok(entries) = std::fs::read_dir(po_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("po") {
                let mo_path = path.with_extension("mo");
                let _ = std::process::Command::new("msgfmt")
                    .arg("-o")
                    .arg(&mo_path)
                    .arg(&path)
                    .status();
            }
        }
    }
}
