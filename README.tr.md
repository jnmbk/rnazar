# rnazar 🧿

[English](README.md) | [Türkçe](README.tr.md)

> Sistem çekmeceniz için çökme bildirici ve nazar boncuğu.

**Linux**, **Windows** ve **macOS** için Rust ile yazılmış, çapraz platform bir sistem tepsisi uygulamasıdır. Sistem tepsinizde yer alır, sağ tık menüsü ile dil seçimi ve hakkında bilgilerini sunar ve bir uygulama çöktüğünde *"nazar değdi ve icabına bakıldı"* uyarısıyla size bildirir.

Bu proje, 2006 yılında Pardus projesi kapsamında KDE3 için geliştirilen knazar projesinden esinlenmiştir.

---

## Özellikler

- **Sistem Tepsisi Simgesi**: Bildirim alanında nazar boncuğu simgesini görüntüler.
- **Sağ Tık Bağlam Menüsü**:
  - **Dil**: **Otomatik** (sistem dilini algılar), **Türkçe** ve diğer desteklenen diller arasında geçiş yapın.
  - **Hakkında**: Uygulama ve nazar boncuğu hakkında bilgi görüntüler.
  - **Çıkış**: Uygulamayı kapatır.
- **Yerelleştirilmiş Çökme Bildirimleri**:
  - Bir uygulama çöktüğünde bildirim gönderir (örn. *"Nazar değdi ve icabına bakıldı."*).
- **Otomatik İşletim Sistemi Çökme Tespiti**:
  - **Linux**: `journalctl` aracılığıyla gerçek zamanlı olarak `systemd-coredump` olaylarını izler.
  - **macOS**: Unified Logging (`log stream`) üzerinden `ReportCrash` olaylarını ve `DiagnosticReports` (`.ips` / `.crash`) dizinini takip eder.
  - **Windows**: Windows Olay Günlüğü'ne (`EventID 1000` / `Application Error`) gerçek zamanlı abone olur ve Windows Hata Bildirimi'ni (`WER\ReportArchive`, `WER\ReportQueue` ve `CrashDumps`) izler.
- **Masaüstü Mesajlaşma Entegrasyonu**:
  - **Linux (D-Bus)**: `org.rnazar.Notifier` arayüzünü `NotifyCrash(app_name: String)` ile oturum veriyolunda (session bus) sunar.
  - **Çapraz Platform CLI**: `rnazar notify --app <ad>` veya `rnazar --crash <ad>` ile bildirimleri kolayca tetikleyin.
  - **IPC**: Üçüncü taraf çökme yakalayıcıları (crash handlers) için yerel named pipe / Unix domain socket dinleyicisi.
- **Simgeli Masaüstü Kısayolları**:
  - Tüm kısayollar (Windows `.lnk`, Linux `.desktop`, macOS `.app`) nazar boncuğu simgesiyle oluşturulur.

---

## Çökme Bildirimlerini Tetikleme

### 1. CLI ile
```bash
# Bir uygulama çökmesini bildirme
rnazar notify --app "hesap-makinesi"

# Hızlı bayrak ile
rnazar --crash "hesap-makinesi"
```

### 2. Linux D-Bus Mesajlaşma Altyapısı ile
```bash
# busctl kullanarak:
busctl --user call org.rnazar.Notifier /org/rnazar/Notifier org.rnazar.Notifier NotifyCrash s "gnome-calculator"

# gdbus kullanarak:
gdbus call --session --dest org.rnazar.Notifier --object-path /org/rnazar/Notifier --method org.rnazar.Notifier.NotifyCrash "firefox"
```

---

## Kurulum ve Paketleme

### Linux (AppImage)
GitHub Sürümleri (Releases) sayfasından `rnazar-x86_64.AppImage` dosyasını indirin, çalıştırılabilir yapın ve çalıştırın:
```bash
chmod +x rnazar-x86_64.AppImage
./rnazar-x86_64.AppImage
```

Yerel olarak derlemek için:
```bash
./packaging/appimage/build-appimage.sh
```

### Linux (Debian / Ubuntu / Linux Mint / Pop!_OS)
GitHub Releases sayfasından `rnazar_<version>_amd64.deb` paketini indirin ve kurun:
```bash
sudo dpkg -i rnazar_<version>_amd64.deb
sudo apt-get install -f # gerekirse eksik bağımlılıkları çözer
```

Yerel olarak derlemek için:
```bash
./packaging/debian/build-deb.sh
```

### Linux (Fedora / RHEL)
GitHub Releases sayfasından `rnazar-<version>-1.x86_64.rpm` paketini indirin ve kurun:
```bash
sudo dnf install ./rnazar-<version>-1.x86_64.rpm
```

Yerel olarak derlemek için:
```bash
./packaging/fedora/build-rpm.sh
```

### Linux (Arch Linux / EndeavourOS / Manjaro / CachyOS / Garuda)
GitHub Releases sayfasından `rnazar-<version>-1-x86_64.pkg.tar.zst` paketini indirin ve kurun:
```bash
sudo pacman -U rnazar-<version>-1-x86_64.pkg.tar.zst
```

Veya `makepkg` ya da paketleme betiği ile derleyin:
```bash
cd packaging/arch && makepkg -si
# veya
./packaging/arch/build-arch.sh
```

### Windows (Inno Setup)
Releases sayfasından `rnazar-vX.Y.Z-windows-x64-setup.exe` dosyasını indirip çalıştırın. Uygulamayı `Program Files` dizinine kuracak, nazar simgeli Başlat Menüsü ve Masaüstü kısayollarını oluşturacak ve isteğe bağlı olarak başlangıçta çalıştırmayı etkinleştirecektir.

### macOS (DMG)
`rnazar-vX.Y.Z-macos-universal.dmg` dosyasını indirin, açın ve `rnazar.app` uygulamasını Uygulamalar (Applications) klasörünüze sürükleyin.

> [!IMPORTANT]
> **macOS Sistem Ayarları ve İzinleri:**
> 1. **Uygulamayı Açma İzni (Gatekeeper):** Apple Geliştirici sertifikasıyla imzalanmamış açık kaynaklı uygulamalarda macOS, ilk açılışta uygulamanın geliştiricisi doğrulanamadığı için açılamadığını belirten bir uyarı gösterebilir.
>    - **Sistem Ayarları** (System Settings) > **Gizlilik ve Güvenlik** (Privacy & Security) bölümüne gidin.
>    - Aşağı kaydırıp Güvenlik bölümünde yer alan **"Yine de Aç"** (Open Anyway) butonuna tıklayın ve onaylayın.
>    - *Alternatif:* Uygulamalar klasöründeki `rnazar.app` simgesine sağ tıklayıp (veya `Control` tuşuna basılı tutarak tıklayıp) **Aç**'ı seçin. Dilerseniz Terminal'den `xattr -cr /Applications/rnazar.app` komutunu vererek karantina özniteliğini de kaldırabilirsiniz.
> 2. **Bildirim İzinleri:** Çökme uyarılarının ekranda gösterilmesi için **Sistem Ayarları** > **Bildirimler** (Notifications) altında `rnazar` için bildirimlere izin verildiğinden emin olun.
> 3. **Tam Disk Erişimi (İsteğe Bağlı / Çökme Tespiti):** `rnazar`, çökmeleri yakalamak için `log stream` ve `DiagnosticReports` dizinini takip eder. Sistem çapındaki kilitlenme raporlarının sorunsuz okunabilmesi için gerekirse **Sistem Ayarları** > **Gizlilik ve Güvenlik** > **Tam Disk Erişimi** (Full Disk Access) listesine `rnazar.app` uygulamasını ekleyin.

---

## Kaynak Koddan Derleme

Sisteminizde Rust'ın (1.80+) kurulu olduğundan emin olun:
```bash
# Linux'ta GTK 3 geliştirme paketlerinin kurulu olduğundan emin olun:
# Ubuntu/Debian: sudo apt-get install libgtk-3-dev
# Arch Linux: sudo pacman -S gtk3

cargo build --release
```

---

## Testleri Çalıştırma

```bash
cargo test
```

---

## Yerelleştirme (i18n)

Çeviriler [`cargo-i18n`](https://github.com/kellpossible/cargo-i18n) kullanılarak standart GNU Gettext formatında ([`po/`](po/) altındaki `.po` / `.pot` dosyaları) yönetilir.

Derleme sırasında, [`build.rs`](build.rs) otomatik olarak `.po` kataloglarını ikili `.mo` kaynaklarına derler ve harici bir çalışma zamanı bağımlılığı olmadan çalıştırılabilir dosyaya gömer.

Güncel dizgeleri (strings) çıkarmak ve çeviri kataloglarını senkronize etmek için:
```bash
# cargo-i18n aracını kurun (bir defaya mahsus)
cargo install cargo-i18n

# Dizgeleri çıkarın ve .po dosyalarını senkronize edin
cargo i18n
```

---

## Lisans

Bu proje [MIT License](LICENSE) şartları altında lisanslanmıştır.

