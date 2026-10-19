#!/usr/bin/env bash
# AppImage (un seul fichier exécutable) à partir du bundle : out/MIRA-<version>-x86_64.AppImage
# Il faut appimagetool : dans le PATH, ou APPIMAGETOOL=/chemin/vers/appimagetool.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="$(cd "$here/../out" && pwd)"
version="$(cat "$out/VERSION")"
appdir="$out/MIRA.AppDir"
tool="${APPIMAGETOOL:-$(command -v appimagetool || true)}"
if [ -z "$tool" ]; then
  echo "appimagetool introuvable : https://github.com/AppImage/appimagetool/releases" >&2
  exit 1
fi

rm -rf "$appdir"
"$here/../common/install-tree.sh" "$out/bundle" "$appdir" /usr
install -m755 "$here/AppRun" "$appdir/AppRun"
cp "$appdir/usr/share/applications/io.github.nnoah29.Mira.desktop" "$appdir/"
cp "$here/../common/io.github.nnoah29.Mira.png" "$appdir/io.github.nnoah29.Mira.png"
ln -sf io.github.nnoah29.Mira.png "$appdir/.DirIcon"

ARCH=x86_64 "$tool" "$appdir" "$out/MIRA-$version-x86_64.AppImage"
echo "AppImage : $out/MIRA-$version-x86_64.AppImage"
