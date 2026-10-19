use crate::i18n::Language;

pub const APP_VERSION: &str = env!("APP_VERSION");
pub const CREATOR_INFO: &str = "Uğur Çetin";
pub const INSPIRATION_INFO: &str = "Inspired by knazar from Pardus 1.0";
pub const LICENSE_NAME: &str = "MIT License";
pub const LICENSE_TEXT: &str = "MIT License\n\nCopyright (c) 2026 Uğur Çetin\n\nPermission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the \"Software\"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so.\n\nTHE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND.";
pub const WEBSITE_URL: &str = "https://github.com/jnmbk/rnazar";

#[cfg(target_os = "linux")]
pub fn localize_about_dialog(dialog: &gtk::AboutDialog, lang: &Language) {
    use gtk::prelude::*;

    fn walk(widget: &gtk::Widget, lang: &Language, action_toggle_idx: &mut usize) {
        if widget.downcast_ref::<gtk::LinkButton>().is_some() {
            return;
        }

        if let Ok(stack) = widget.clone().downcast::<gtk::Stack>() {
            for child in stack.children() {
                match stack.child_name(&child).as_deref() {
                    Some("main") => stack.set_child_title(&child, Some(lang.dialog_about())),
                    Some("credits") => stack.set_child_title(&child, Some(lang.dialog_credits())),
                    Some("license") => stack.set_child_title(&child, Some(lang.dialog_license())),
                    _ => {}
                }
            }
        }

        if let Ok(switcher) = widget.clone().downcast::<gtk::StackSwitcher>() {
            let titles = [
                lang.dialog_about(),
                lang.dialog_credits(),
                lang.dialog_license(),
            ];
            for (idx, child) in switcher.children().iter().enumerate() {
                if let Some(&title) = titles.get(idx) {
                    if let Ok(btn) = child.clone().downcast::<gtk::Button>() {
                        btn.set_label(title);
                    }
                }
            }
            return;
        }

        if let Ok(toggle) = widget.clone().downcast::<gtk::ToggleButton>() {
            match *action_toggle_idx {
                0 => toggle.set_label(lang.dialog_credits()),
                1 => toggle.set_label(lang.dialog_license()),
                _ => {}
            }
            *action_toggle_idx += 1;
            return;
        }

        if let Ok(btn) = widget.clone().downcast::<gtk::Button>() {
            btn.set_label(lang.dialog_close());
            return;
        }

        if let Ok(container) = widget.clone().downcast::<gtk::Container>() {
            for child in container.children() {
                walk(&child, lang, action_toggle_idx);
            }
        }
    }

    let mut action_toggle_idx = 0;
    walk(dialog.upcast_ref(), lang, &mut action_toggle_idx);
}

#[cfg(target_os = "linux")]
pub fn show_about_window(lang: Language) {
    use gdk_pixbuf::Pixbuf;
    use gtk::prelude::*;
    use gtk::{AboutDialog, License};

    // Initialize GTK if not already initialized
    let _ = gtk::init();

    let dialog = AboutDialog::new();
    dialog.set_program_name("rnazar");
    dialog.set_title(lang.about_title());
    dialog.set_version(Some(APP_VERSION));
    let comments = format!("{}\n\n{}", lang.about_body(), lang.inspiration_info());
    dialog.set_comments(Some(&comments));
    dialog.set_copyright(Some("Copyright © 2026 Uğur Çetin"));
    dialog.set_authors(&[CREATOR_INFO]);

    dialog.add_credit_section(lang.inspiration_section(), &[lang.inspiration_info()]);
    dialog.set_license_type(License::MitX11);
    dialog.set_license(Some(LICENSE_TEXT));
    dialog.set_website(Some(WEBSITE_URL));
    dialog.set_website_label(Some(lang.website_label()));
    localize_about_dialog(&dialog, &lang);

    dialog.connect_activate_link(|_dialog, uri| {
        let _ = gtk::show_uri_on_window(gtk::Window::NONE, uri, 0);
        gtk::glib::Propagation::Stop
    });

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
            "{}\n\n{}\n\nCreator: {}\nLicense: {}\nWebsite: {}\n\n{}",
            lang.about_body(),
            lang.inspiration_info(),
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
