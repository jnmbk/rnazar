use crate::i18n::Language;
use muda::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use std::error::Error;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

const ICON_BYTES: &[u8] = include_bytes!("../assets/icons/png/32x32.png");

pub struct TrayMenuController {
    pub menu: Menu,
    pub lang_submenu: Submenu,
    pub item_lang_auto: CheckMenuItem,
    pub item_lang_en: CheckMenuItem,
    pub item_lang_tr: CheckMenuItem,
    pub item_about: MenuItem,
    pub item_exit: MenuItem,
}

impl TrayMenuController {
    pub fn new(current_lang: Language) -> Result<Self, Box<dyn Error>> {
        let menu = Menu::new();

        let item_lang_auto = CheckMenuItem::new(
            current_lang.menu_lang_auto(),
            true,
            current_lang == Language::Auto,
            None,
        );
        let item_lang_en = CheckMenuItem::new(
            current_lang.menu_lang_en(),
            true,
            current_lang == Language::English,
            None,
        );
        let item_lang_tr = CheckMenuItem::new(
            current_lang.menu_lang_tr(),
            true,
            current_lang == Language::Turkish,
            None,
        );

        let lang_submenu = Submenu::new(current_lang.menu_language(), true);
        lang_submenu.append(&item_lang_auto)?;
        lang_submenu.append(&item_lang_en)?;
        lang_submenu.append(&item_lang_tr)?;

        let item_about = MenuItem::new(current_lang.menu_about(), true, None);
        let item_sep = PredefinedMenuItem::separator();
        let item_exit = MenuItem::new(current_lang.menu_exit(), true, None);

        menu.append(&lang_submenu)?;
        menu.append(&item_about)?;
        menu.append(&item_sep)?;
        menu.append(&item_exit)?;

        Ok(Self {
            menu,
            lang_submenu,
            item_lang_auto,
            item_lang_en,
            item_lang_tr,
            item_about,
            item_exit,
        })
    }

    pub fn update_language(&self, lang: Language) {
        self.lang_submenu.set_text(lang.menu_language());
        self.item_lang_auto.set_text(lang.menu_lang_auto());
        self.item_lang_auto.set_checked(lang == Language::Auto);

        self.item_lang_en.set_text(lang.menu_lang_en());
        self.item_lang_en.set_checked(lang == Language::English);

        self.item_lang_tr.set_text(lang.menu_lang_tr());
        self.item_lang_tr.set_checked(lang == Language::Turkish);

        self.item_about.set_text(lang.menu_about());
        self.item_exit.set_text(lang.menu_exit());
    }
}

pub fn load_tray_icon() -> Result<Icon, Box<dyn Error>> {
    let img = image::load_from_memory(ICON_BYTES)?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = Icon::from_rgba(rgba.into_raw(), width, height)?;
    Ok(icon)
}

pub fn build_tray_icon(menu: Menu) -> Result<TrayIcon, Box<dyn Error>> {
    let icon = load_tray_icon()?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("rnazar")
        .with_icon(icon)
        .build()?;
    Ok(tray)
}
