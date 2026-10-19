# Changelog

## [0.9.0](https://github.com/jnmbk/rnazar/compare/v0.8.3...v0.9.0) (2026-10-19)


### Features

* **about:** clean author name to prevent broken mailto link logs, add knazar Pardus 1.0 inspiration ([4f5dcd3](https://github.com/jnmbk/rnazar/commit/4f5dcd313dc5a43cb13a7d8e78d8c428450f570c))
* **about:** display Pardus knazar credit under dedicated Inspiration section instead of Artwork ([cb3c4ca](https://github.com/jnmbk/rnazar/commit/cb3c4ca1e251e0a5b94202f2d97a8187bae3ac9a))
* add packaging support for Debian, Fedora, and Arch Linux ([278d46d](https://github.com/jnmbk/rnazar/commit/278d46d4b70cb8701d48713e8032c221a3f56160))
* added protection status as a tooltip to tray icon ([d4cb650](https://github.com/jnmbk/rnazar/commit/d4cb6501f834a0f8f044059359e59b76c62898f8))
* crash monitoring for windows and mac os ([238c6ba](https://github.com/jnmbk/rnazar/commit/238c6ba0eea240340c1542f94d31f517817eafb5))
* disk and ram usage optimizations ([89d150e](https://github.com/jnmbk/rnazar/commit/89d150e439f46b513c1159a67d3ff671c8572b3b))
* exit confirmation ([dc66387](https://github.com/jnmbk/rnazar/commit/dc663878d9c9d08e8310abc0602e52b44d373678))
* **i18n:** migrate translations to GNU gettext PO/MO catalogs ([69264d1](https://github.com/jnmbk/rnazar/commit/69264d178b55d6f8a9fc045c19d3b23cda69edc3))
* **i18n:** update crash title translations to "Evil eye!" in English and Turkish ([8695a01](https://github.com/jnmbk/rnazar/commit/8695a01e40f9cda80016530828968e5800651f67))
* initial commit for rnazar system tray & crash notifier ([ff9dbb9](https://github.com/jnmbk/rnazar/commit/ff9dbb9bb4670d4c41ee555507062877522e3f33))
* **logging:** use log::debug! and log::warn! with env_logger defaulting to warn ([bb7b7bd](https://github.com/jnmbk/rnazar/commit/bb7b7bdf9ba7e1813d1f44ced4ae5a55c8500324))
* **mac:** better dmg creation ([fffc4fc](https://github.com/jnmbk/rnazar/commit/fffc4fc1ed60b7bfb4ee5972e93e303372f7b5eb))
* monitor systemd coredump for crashes ([f75e5c6](https://github.com/jnmbk/rnazar/commit/f75e5c641c1f5435ba107465c873364e84a0fea4))
* optimized package sizes ([65bcb9f](https://github.com/jnmbk/rnazar/commit/65bcb9f8b5ad7ef7f6b7b7b5f1878c4d1d1d61bb))
* protection on/off control ([07c72e0](https://github.com/jnmbk/rnazar/commit/07c72e0243a3610128c0d9b9ccb17741c46e3975))
* **release:** dynamically inject Git release tag version into About window and installers ([885e4f2](https://github.com/jnmbk/rnazar/commit/885e4f2d27d9902d85489bf8a46b7a02ed003f74))


### Bug Fixes

* correctly forward muda MenuEvent with payload and display About dialog ([8fc7c88](https://github.com/jnmbk/rnazar/commit/8fc7c881c629b97efc118a2c8bcbd1a33472351b))
* crash detection test ([388eca2](https://github.com/jnmbk/rnazar/commit/388eca23e93ec330ede507f4a93bc0dd9345af32))
* **debian:** fix control file formatting and trailing newline issue for dpkg-deb ([78db9ec](https://github.com/jnmbk/rnazar/commit/78db9ec4dd3db46e2a25b81cc78276b907718f8a))
* enhance Windows event XML parsing for EventID 1001 and add related tests ([a6711dc](https://github.com/jnmbk/rnazar/commit/a6711dc41758e71e982dbdf643ece5be3dc88b64))
* **fedora:** remove cargo BuildRequires and add --nodeps to rpmbuild ([1688d06](https://github.com/jnmbk/rnazar/commit/1688d06b9545c66ab7f84e1f34ed915cf9443a6e))
* **i18n:** about window translations ([692f8aa](https://github.com/jnmbk/rnazar/commit/692f8aaafa379462cefe10cd6b7b1c38afab691b))
* **i18n:** about window translations ([d73ed74](https://github.com/jnmbk/rnazar/commit/d73ed74246735ffa22ae458a2b91e47830d88d3f))
* **i18n:** better about text ([8017667](https://github.com/jnmbk/rnazar/commit/801766756182c2d095352409743886538e9546ae))
* **i18n:** translateable Created by in about window ([b93c54d](https://github.com/jnmbk/rnazar/commit/b93c54d2056c01b9328855f16f17649a3aaf4f4f))
* **ipc:** harden socket permissions, bound message reads, and remove unsafe getuid ([9661e87](https://github.com/jnmbk/rnazar/commit/9661e87b1b4afe822df49d20657de54fc51d407e))
* mac os installer ([9fe4632](https://github.com/jnmbk/rnazar/commit/9fe4632cf365eefe150d0dcd4af81324f2e79ec4))
* nofitication hendling ([ad3346b](https://github.com/jnmbk/rnazar/commit/ad3346bbae4374aeff5e8832995da952c96bcce2))
* resolve crash notifications and detection on macos ([98bea93](https://github.com/jnmbk/rnazar/commit/98bea9378581c856cefeef9139f95cd4d5ca198d))
* resolve notify-rust handle compilation on windows and macos ([9ed4655](https://github.com/jnmbk/rnazar/commit/9ed46559e204b34becc167b95527027dd3676720))
* test failures ([0562ab2](https://github.com/jnmbk/rnazar/commit/0562ab20b55c6277658f4160564d0e7cd5224515))
* test warning on mac os ([2d4f516](https://github.com/jnmbk/rnazar/commit/2d4f516fcfb1943f005d169d6f914dac46ca485a))
* test warnings ([c393080](https://github.com/jnmbk/rnazar/commit/c393080dc3497953a3978f8e2a73876efecf2aee))
* update action reference for Release Please workflow ([f82e6f4](https://github.com/jnmbk/rnazar/commit/f82e6f44acff15a4552df541dce49bae8e5ad110))
* update compression method for AppImage creation to zstd ([fa85eed](https://github.com/jnmbk/rnazar/commit/fa85eed79a8ae2e37ddbdf61e9d3855f8101307f))
* update crash notification message in README and main.rs ([b9ababb](https://github.com/jnmbk/rnazar/commit/b9ababbb7adfa835751df1e0a77218da2ed2de55))
* **windows:** bring ListenerExt trait into scope for interprocess listener.incoming() ([9606953](https://github.com/jnmbk/rnazar/commit/9606953db5f6b7d9eced8e79ca84918af28ee1fe))
* **windows:** bring Stream trait into scope for Stream::connect ([acf16cc](https://github.com/jnmbk/rnazar/commit/acf16ccef58cf77e3c87cd4e766fbf265b888429))
* **windows:** remove unused trait imports in windows_ipc ([37187e9](https://github.com/jnmbk/rnazar/commit/37187e9e99af61b2147bc53b9b0abe5166388142))
* **windows:** suppress console window in release builds with windows_subsystem attribute ([8a1f4d5](https://github.com/jnmbk/rnazar/commit/8a1f4d53f8fd1e32a1e98586a3697a375ee8efcf))

## [0.8.3](https://github.com/jnmbk/rnazar/compare/v0.8.2...v0.8.3) (2026-10-19)


### Bug Fixes

* **i18n:** translateable Created by in about window ([b93c54d](https://github.com/jnmbk/rnazar/commit/b93c54d2056c01b9328855f16f17649a3aaf4f4f))

## [0.8.2](https://github.com/jnmbk/rnazar/compare/v0.8.1...v0.8.2) (2026-10-19)


### Bug Fixes

* **i18n:** about window translations ([692f8aa](https://github.com/jnmbk/rnazar/commit/692f8aaafa379462cefe10cd6b7b1c38afab691b))
* update action reference for Release Please workflow ([f82e6f4](https://github.com/jnmbk/rnazar/commit/f82e6f44acff15a4552df541dce49bae8e5ad110))
