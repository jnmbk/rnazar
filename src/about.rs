use crate::i18n::Language;

pub const CREATOR_INFO: &str = "Uğur Çetin <ugur.jnmbk@gmail.com>";
pub const LICENSE_NAME: &str = "MIT License";
pub const LICENSE_TEXT: &str = "MIT License\n\nCopyright (c) 2026 Uğur Çetin\n\nPermission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the \"Software\"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so.\n\nTHE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND.";
pub const WEBSITE_URL: &str = "https://github.com/jnmbk/rnazar";

#[cfg(target_os = "linux")]
pub fn show_about_window(lang: Language) {
    use gdk_pixbuf::Pixbuf;
    use gtk::prelude::*;
    use gtk::{AboutDialog, License};

    // Initialize GTK if not already initialized
    let _ = gtk::init();

    let dialog = AboutDialog::new();
    dialog.set_program_name("rnazar");
    dialog.set_version(Some("0.1.0"));
    dialog.set_comments(Some(lang.about_body()));
    dialog.set_copyright(Some("Copyright © 2026 Uğur Çetin"));
    dialog.set_authors(&[CREATOR_INFO]);
    dialog.set_license_type(License::MitX11);
    dialog.set_license(Some(LICENSE_TEXT));
    dialog.set_website(Some(WEBSITE_URL));
    dialog.set_website_label(Some("rnazar on GitHub"));

    let icon_bytes = include_bytes!("../assets/icons/png/128x128.png");
    if let Ok(pixbuf) = Pixbuf::from_read(std::io::Cursor::new(icon_bytes)) {
        dialog.set_logo(Some(&pixbuf));
    }

    dialog.connect_response(|dialog, _| {
        dialog.close();
    });

    dialog.show_all();
    dialog.present();
}

#[cfg(not(target_os = "linux"))]
pub fn show_about_window(lang: Language) {
    std::thread::spawn(move || {
        let title = lang.about_title();
        let description = format!(
            "{}\n\nCreator: {}\nLicense: {}\nWebsite: {}\n\n{}",
            lang.about_body(),
            CREATOR_INFO,
            LICENSE_NAME,
            WEBSITE_URL,
            LICENSE_TEXT
        );

        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(&description)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    });
}
