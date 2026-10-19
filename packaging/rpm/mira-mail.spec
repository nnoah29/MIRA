# Paquet RPM de MIRA (Fedora, RHEL/Rocky/Alma 9+, openSUSE) construit à partir du bundle déjà compilé.
%global debug_package %{nil}
%global __strip /bin/true
# Bibliothèques privées livrées avec l'application : ni fournies au système, ni réclamées.
%global __provides_exclude_from ^%{_prefix}/lib/mira-mail/.*$
# libc GLIBC_2.39 : références faibles de la bibliothèque standard Rust, utilisées seulement si présentes.
%global __requires_exclude ^(lib(app|flutter_linux_gtk|pdfium|mira_bridge|url_launcher_linux_plugin)\\.so.*|libc\\.so\\.6\\(GLIBC_2\\.(3[5-9]|4[0-9])\\).*)$

Name:           mira-mail
Version:        %{mira_version}
Release:        1%{?dist}
Summary:        Desktop mail client that works offline
License:        MIT
URL:            https://github.com/nnoah29/MIRA
ExclusiveArch:  x86_64
Requires:       gtk3
Recommends:     (zenity or kdialog)
Recommends:     xdg-utils

%description
MIRA keeps a full copy of your mail on your own computer, so everything
opens instantly, works without a connection, and stays yours.

Connect Gmail, Outlook, Yahoo or any IMAP account and read, sort, search
and write from a single, calm interface.

%install
%{mira_packaging}/common/install-tree.sh %{mira_bundle} %{buildroot} %{_prefix}

%files
%license %{_datadir}/licenses/mira-mail/LICENSE
%{_bindir}/mira-mail
%{_prefix}/lib/mira-mail/
%{_datadir}/applications/io.github.nnoah29.Mira.desktop
%{_datadir}/icons/hicolor/512x512/apps/io.github.nnoah29.Mira.png
%{_datadir}/icons/hicolor/scalable/apps/io.github.nnoah29.Mira.svg
%{_datadir}/metainfo/io.github.nnoah29.Mira.metainfo.xml
