pub const MENU_LANGUAGE: &str = "Dil";
pub const MENU_LANG_AUTO: &str = "Otomatik";
pub const MENU_LANG_EN: &str = "İngilizce";
pub const MENU_LANG_TR: &str = "Türkçe";
pub const MENU_ABOUT: &str = "rnazar Hakkında";
pub const MENU_EXIT: &str = "Çıkış";

pub const ABOUT_TITLE: &str = "rnazar Hakkında";
pub const ABOUT_BODY: &str =
    "rnazar v0.1.0\nElemtere fiş, kem gözlere şiş.\nÇöken uygulamalarınızın icabına bakar.";

pub const CRASH_TITLE: &str = "Nazar Değdi!";
pub const CRASH_BODY: &str = "Nazar değdi ve icabına bakıldı.";

pub fn crash_body_with_app(app: &str) -> String {
    format!("{} uygulamasına nazar değdi ve icabına bakıldı.", app)
}
