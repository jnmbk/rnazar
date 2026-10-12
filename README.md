# rnazar 🧿

> The evil eye ward for your system tray & crash notifier.

A cross-platform system tray application written in Rust for **Linux**, **Windows**, and **macOS**. It lives in your system tray, provides language selection and about controls via a right-click menu, and alerts you with *"nazar değdi ve icabına bakıldı"* whenever an application crashes.

This project is inspired by knazar project that was made for KDE3 back in 2006 by Pardus project.

---

## Features

- **System Tray Icon**: Displays the Nazar amulet icon in the notification area.
- **Right-Click Context Menu**:
  - **Language**: Toggle between **Auto** (detects system language), **English**, and **Türkçe**.
  - **About**: Displays information about the app and the Nazar amulet.
  - **Exit**: Closes the application.
- **Localized Crash Notifications**:
  - Turkish: *"Nazar değdi ve icabına bakıldı."*
  - English: *"An evil eye struck and it was taken care of."*
- **Desktop Messaging Integration**:
  - **Linux (D-Bus)**: Exposes `org.rnazar.Notifier` on the session bus with `NotifyCrash(app_name: String)`.
  - **Cross-Platform CLI**: Trigger notifications easily with `rnazar notify --app <name>` or `rnazar --crash <name>`.
  - **IPC**: Local named pipe / Unix domain socket listener for third-party crash handlers.
- **Desktop Shortcuts with Icon**:
  - All shortcuts (Windows `.lnk`, Linux `.desktop`, macOS `.app`) display the Nazar icon.

---

## Triggering Crash Notifications

### 1. Via CLI
```bash
# Report an application crash
rnazar notify --app "calculator"

# Quick flag
rnazar --crash "calculator"
```

### 2. Via Linux D-Bus Messaging Infrastructure
```bash
# Using busctl:
busctl --user call org.rnazar.Notifier /org/rnazar/Notifier org.rnazar.Notifier NotifyCrash s "gnome-calculator"

# Using gdbus:
gdbus call --session --dest org.rnazar.Notifier --object-path /org/rnazar/Notifier --method org.rnazar.Notifier.NotifyCrash "firefox"
```

---

## Installation & Packaging

### Linux (AppImage)
Download `rnazar-x86_64.AppImage` from GitHub Releases, make it executable, and run:
```bash
chmod +x rnazar-x86_64.AppImage
./rnazar-x86_64.AppImage
```

To build locally:
```bash
./packaging/appimage/build-appimage.sh
```

### Windows (Inno Setup)
Download and run `rnazar-vX.Y.Z-windows-x64-setup.exe` from Releases. It will install the application to `Program Files`, create Start Menu and Desktop shortcuts with the Nazar icon, and optionally enable startup on boot.

### macOS (DMG)
Download `rnazar-vX.Y.Z-macos-universal.dmg`, open it, and drag `rnazar.app` to your Applications folder.

---

## Building from Source

Ensure you have Rust installed (1.80+):
```bash
# On Linux, ensure GTK 3 development headers are installed:
# Ubuntu/Debian: sudo apt-get install libgtk-3-dev
# Arch Linux: sudo pacman -S gtk3

cargo build --release
```

---

## Running Tests

```bash
cargo test
```

---

## Localization (i18n)

Translations are managed via [`cargo-i18n`](https://github.com/kellpossible/cargo-i18n) using the standard GNU Gettext format (`.po` / `.pot` files under [`po/`](po/)).

At build time, [`build.rs`](build.rs) automatically compiles `.po` catalogs to binary `.mo` resources and embeds them into the executable with zero runtime or external dependencies.

To extract updated strings and synchronize translation catalogs:
```bash
# Install cargo-i18n (once)
cargo install cargo-i18n

# Extract strings and synchronize .po files
cargo i18n
```

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE) or [MIT License](LICENSE).

