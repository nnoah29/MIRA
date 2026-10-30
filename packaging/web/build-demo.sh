#!/usr/bin/env bash
# Démo en ligne (PWA) : compile l'app pour le web et prépare dans packaging/out/web-demo/ ce qu'il faut copier
# à la racine de la branche `web` (GitHub Pages : https://nnoah29.github.io/MIRA/).
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
out="$root/packaging/out/web-demo"

(cd "$root/app" && flutter build web --release --base-href /MIRA/ --no-web-resources-cdn)

rm -rf "$out"
mkdir -p "$out"
cp -r "$root/app/build/web/." "$out/"
rm -f "$out"/canvaskit/skwasm* "$out"/canvaskit/*.symbols "$out"/canvaskit/chromium/*.symbols

build="$(git -C "$root" rev-parse --short HEAD)-$(date +%Y%m%d%H%M)"
sed -i "s/__BUILD__/$build/" "$out/sw.js"
if grep -q "__BUILD__" "$out/sw.js"; then
  echo "Jeton de build non remplacé dans sw.js" >&2
  exit 1
fi
echo "Démo prête : $out (build $build)"
echo "À copier à la racine de la branche web, en gardant 404.html et .nojekyll."
