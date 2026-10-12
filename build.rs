fn main() {
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
