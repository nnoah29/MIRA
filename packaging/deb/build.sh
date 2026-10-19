#!/usr/bin/env bash
# Paquet .deb (Debian, Ubuntu, Mint, Pop!_OS…) à partir du bundle : packaging/out/mira-mail_<version>_amd64.deb
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="$here/../out"
version="$(cat "$out/VERSION")"
tree="$out/deb-root"

rm -rf "$tree"
"$here/../common/install-tree.sh" "$out/bundle" "$tree" /usr
install -d "$tree/DEBIAN"
{
  echo "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
  echo "Upstream-Name: MIRA"
  echo "Source: https://github.com/nnoah29/MIRA"
  echo
  echo "Files: *"
  echo "Copyright: 2026 nnoah29"
  echo "License: MIT"
  sed 's/^$/./; s/^/ /' "$here/../../LICENSE"
} | install -Dm644 /dev/stdin "$tree/usr/share/doc/mira-mail/copyright"
size="$(du -sk "$tree/usr" | cut -f1)"
sed -e "s/@VERSION@/$version/" -e "s/@SIZE@/$size/" "$here/control.in" > "$tree/DEBIAN/control"
install -m755 "$here/postinst" "$here/postrm" "$tree/DEBIAN/"

deb="$out/mira-mail_${version}_amd64.deb"
dpkg-deb --build --root-owner-group -Zxz "$tree" "$deb"
echo "Paquet : $deb"
