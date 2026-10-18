#!/usr/bin/env bash
# Construit le bundle Linux de MIRA (app Flutter + moteur Rust) dans packaging/out/bundle,
# point de départ commun de tous les formats (.deb, .rpm, PKGBUILD, AppImage, Flatpak, Snap).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/packaging/out"
bundle="$out/bundle"

echo "== Moteur (bibliothèque du pont, release)"
(cd "$root/bridge" && cargo build --release)

echo "== Application Flutter (release)"
(cd "$root/app" && flutter build linux --release)

echo "== Assemblage"
rm -rf "$bundle"
mkdir -p "$out"
cp -a "$root/app/build/linux/x64/release/bundle" "$bundle"
install -m755 "$root/bridge/target/release/libmira_bridge.so" "$bundle/lib/"
# Pont Java d'un plugin Android : jamais chargé sous Linux, il ferait dépendre les paquets de libjvm.
rm -f "$bundle/lib/libdartjni.so"
strip --strip-unneeded "$bundle/lib/libmira_bridge.so" || true

version="$(sed -n 's/^version: *\([0-9.]*\).*/\1/p' "$root/app/pubspec.yaml")"
echo "$version" > "$out/VERSION"
echo "Bundle prêt : $bundle (version $version)"
