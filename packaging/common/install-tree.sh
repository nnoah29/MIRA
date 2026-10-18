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
find "$app/lib" -name '*.so' -exec chmod 755 {} +
ln -sf ../lib/mira-mail/mira "$dest$prefix/bin/mira-mail"

sed "s|^Exec=.*|Exec=mira-mail|" "$here/io.github.nnoah29.Mira.desktop" \
  | install -Dm644 /dev/stdin "$dest$prefix/share/applications/io.github.nnoah29.Mira.desktop"
install -Dm644 "$here/io.github.nnoah29.Mira.png" "$dest$prefix/share/icons/hicolor/512x512/apps/io.github.nnoah29.Mira.png"
install -Dm644 "$here/io.github.nnoah29.Mira.svg" "$dest$prefix/share/icons/hicolor/scalable/apps/io.github.nnoah29.Mira.svg"
install -Dm644 "$here/../../LICENSE" "$dest$prefix/share/licenses/mira-mail/LICENSE"
install -Dm644 "$here/io.github.nnoah29.Mira.metainfo.xml" "$dest$prefix/share/metainfo/io.github.nnoah29.Mira.metainfo.xml"
