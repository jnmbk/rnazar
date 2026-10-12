%define _binary_payload w19.zstdio

Name:           rnazar
Version:        %{version}
Release:        1%{?dist}
Summary:        System tray evil eye ward and crash notifier
License:        MIT or ASL 2.0
URL:            https://github.com/jnmbk/rnazar

Requires:       gtk3

%description
A cross-platform system tray application that lives in your notification
area and alerts you when applications crash with localized notifications.

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/scalable/apps
mkdir -p %{buildroot}%{_datadir}/dbus-1/services
mkdir -p %{buildroot}%{_datadir}/licenses/%{name}

install -s -m 755 %{_sourcedir}/rnazar %{buildroot}%{_bindir}/rnazar
install -m 644 %{_sourcedir}/rnazar.desktop %{buildroot}%{_datadir}/applications/rnazar.desktop
install -m 644 %{_sourcedir}/org.rnazar.Notifier.service %{buildroot}%{_datadir}/dbus-1/services/org.rnazar.Notifier.service
install -m 644 %{_sourcedir}/LICENSE %{buildroot}%{_datadir}/licenses/%{name}/LICENSE
install -m 644 %{_sourcedir}/nazar.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/rnazar.svg

for size in 16x16 32x32 48x48 64x64 128x128 256x256 512x512; do
    if [ -f "%{_sourcedir}/png/${size}.png" ]; then
        mkdir -p %{buildroot}%{_datadir}/icons/hicolor/${size}/apps
        install -m 644 %{_sourcedir}/png/${size}.png %{buildroot}%{_datadir}/icons/hicolor/${size}/apps/rnazar.png
    fi
done

%post
if [ $1 -eq 1 ] ; then
    touch --no-create %{_datadir}/icons/hicolor &>/dev/null || :
fi
update-desktop-database &> /dev/null || :

%postun
if [ $1 -eq 0 ] ; then
    touch --no-create %{_datadir}/icons/hicolor &>/dev/null || :
    gtk-update-icon-cache %{_datadir}/icons/hicolor &>/dev/null || :
fi
update-desktop-database &> /dev/null || :

%posttrans
gtk-update-icon-cache %{_datadir}/icons/hicolor &>/dev/null || :

%files
%{_bindir}/rnazar
%{_datadir}/applications/rnazar.desktop
%{_datadir}/dbus-1/services/org.rnazar.Notifier.service
%{_datadir}/icons/hicolor/*/apps/rnazar.*
%{_datadir}/licenses/%{name}/LICENSE

%changelog
* Mon Oct 12 2026 jnmbk <noreply@github.com> - %{version}-1
- Release version %{version}

