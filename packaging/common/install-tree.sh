#!/usr/bin/env bash
# Installe le bundle MIRA dans une arborescence de paquet : install-tree.sh <bundle> <destdir> [prefix]
# Résultat : <prefix>/lib/mira-mail (application), <prefix>/bin/mira-mail, lanceur et icônes.
set -euo pipefail

bundle="$1"
dest="$2"
prefix="${3:-/usr}"
here="$(cd "$(dirname "$0")" && pwd)"
app="$dest$prefix/lib/mira-mail"

mkdir -p "$app" "$dest$prefix/bin"
cp -a "$bundle/." "$app/"
chmod 755 "$app/mira"
[ -f "$app/mira-daemon" ] && chmod 755 "$app/mira-daemon"
find "$app/lib" -name '*.so' -exec chmod 755 {} +
ln -sf ../lib/mira-mail/mira "$dest$prefix/bin/mira-mail"

sed "s|^Exec=.*|Exec=mira-mail|" "$here/com.toffanoah.mira.desktop" \
  | install -Dm644 /dev/stdin "$dest$prefix/share/applications/com.toffanoah.mira.desktop"
install -Dm644 "$here/com.toffanoah.mira.png" "$dest$prefix/share/icons/hicolor/512x512/apps/com.toffanoah.mira.png"
install -Dm644 "$here/com.toffanoah.mira.svg" "$dest$prefix/share/icons/hicolor/scalable/apps/com.toffanoah.mira.svg"
install -Dm644 "$here/../../LICENSE" "$dest$prefix/share/licenses/mira-mail/LICENSE"
install -Dm644 "$here/com.toffanoah.mira.metainfo.xml" "$dest$prefix/share/metainfo/com.toffanoah.mira.metainfo.xml"

# Moteur en arrière-plan : unité systemd --user (activée par l'application, jamais par le paquet).
sed "s|@DAEMON@|$prefix/lib/mira-mail/mira-daemon|" "$here/mira-daemon.service.in" \
  | install -Dm644 /dev/stdin "$dest$prefix/lib/systemd/user/mira-daemon.service"
