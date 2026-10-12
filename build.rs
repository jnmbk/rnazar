fn main() {
    println!("cargo:rerun-if-changed=po/tr.po");
    println!("cargo:rerun-if-changed=po/en.po");

    // Attempt to re-compile .po to .mo if msgfmt is installed
    compile_po_files();

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icons/rnazar.ico");
        res.set("ProductName", "rnazar");
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
