use crate::i18n::{available_languages, Language};
use muda::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use std::error::Error;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

const ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tray_icon_32x32.rgba"));
const ICON_GRAY_RGBA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/tray_icon_gray_32x32.rgba"));

pub struct LanguageMenuItem {
    pub language: Language,
    pub item: CheckMenuItem,
}

pub struct TrayMenuController {
    pub menu: Menu,
    pub item_protection: CheckMenuItem,
    pub lang_submenu: Submenu,
    pub item_lang_auto: CheckMenuItem,
    pub lang_items: Vec<LanguageMenuItem>,
    pub item_about: MenuItem,
    pub item_exit: MenuItem,
}

impl TrayMenuController {
    pub fn new(current_lang: Language, protection_enabled: bool) -> Result<Self, Box<dyn Error>> {
        let menu = Menu::new();

        let item_protection = CheckMenuItem::new(
            current_lang.menu_protection(),
            true,
            protection_enabled,
            None,
        );

        let item_lang_auto = CheckMenuItem::new(
            current_lang.menu_lang_auto(),
            true,
            current_lang == Language::Auto,
            None,
        );

        let lang_submenu = Submenu::new(current_lang.menu_language(), true);
        lang_submenu.append(&item_lang_auto)?;

        let mut lang_items = Vec::new();
        for avail in available_languages() {
            let is_checked = current_lang == avail.language;
            let item = CheckMenuItem::new(avail.native_name, true, is_checked, None);
            lang_submenu.append(&item)?;
            lang_items.push(LanguageMenuItem {
                language: avail.language,
                item,
            });
        }

        let item_about = MenuItem::new(current_lang.menu_about(), true, None);
        let item_sep1 = PredefinedMenuItem::separator();
        let item_sep2 = PredefinedMenuItem::separator();
        let item_exit = MenuItem::new(current_lang.menu_exit(), true, None);

        menu.append(&item_protection)?;
        menu.append(&item_sep1)?;
        menu.append(&lang_submenu)?;
        menu.append(&item_about)?;
        menu.append(&item_sep2)?;
        menu.append(&item_exit)?;

        Ok(Self {
            menu,
            item_protection,
            lang_submenu,
            item_lang_auto,
            lang_items,
            item_about,
            item_exit,
        })
    }

    pub fn update_language(&self, lang: &Language) {
        self.item_protection.set_text(lang.menu_protection());
        self.lang_submenu.set_text(lang.menu_language());
        self.item_lang_auto.set_text(lang.menu_lang_auto());
        self.item_lang_auto.set_checked(lang == &Language::Auto);

        for lang_item in &self.lang_items {
            lang_item.item.set_checked(lang == &lang_item.language);
        }

        self.item_about.set_text(lang.menu_about());
        self.item_exit.set_text(lang.menu_exit());
    }

    pub fn update_protection(&self, enabled: bool) {
        self.item_protection.set_checked(enabled);
    }
}

pub fn load_tray_icon(protection_enabled: bool) -> Result<Icon, Box<dyn Error>> {
    let raw = if protection_enabled {
        ICON_RGBA
    } else {
        ICON_GRAY_RGBA
    };
    let icon = Icon::from_rgba(raw.to_vec(), 32, 32)?;
    Ok(icon)
}

pub fn build_tray_icon(menu: Menu, protection_enabled: bool) -> Result<TrayIcon, Box<dyn Error>> {
    let icon = load_tray_icon(protection_enabled)?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("rnazar")
        .with_icon(icon)
        .build()?;
    Ok(tray)
}
