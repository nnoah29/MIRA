#!/usr/bin/env bash
# Installe MIRA pour l'utilisateur courant (sans sudo) depuis la dernière release GitHub.
# Relancé, il met à jour si une version plus récente existe. `--uninstall` retire l'installation.
# Usage : install.sh [--force] [--uninstall] [version]   (ex. install.sh 1.0.0)
set -euo pipefail

repo="nnoah29/MIRA"
data="${XDG_DATA_HOME:-$HOME/.local/share}"
root="$data/mira-mail"
bin="$HOME/.local/bin/mira-mail"
desktop="$data/applications/io.github.nnoah29.Mira.desktop"
icons="$data/icons/hicolor"
force=0
wanted=""

for arg in "$@"; do
  case "$arg" in
    --force) force=1 ;;
    --uninstall)
      rm -rf "$root" "$bin" "$desktop" \
        "$icons/512x512/apps/io.github.nnoah29.Mira.png" "$icons/scalable/apps/io.github.nnoah29.Mira.svg" \
        "$data/metainfo/io.github.nnoah29.Mira.metainfo.xml"
      echo "MIRA désinstallé. Vos comptes et mails restent dans ~/.local/share et ~/.config."
      exit 0 ;;
    -h|--help) sed -n '2,4p' "$0"; exit 0 ;;
    *) wanted="${arg#v}" ;;
  esac
done

[ "$(uname -m)" = "x86_64" ] || { echo "Seul x86_64 est publié pour Linux." >&2; exit 1; }

use_gh=0
if command -v gh >/dev/null && gh auth status >/dev/null 2>&1; then use_gh=1; fi

latest_version() {
  if [ "$use_gh" = 1 ]; then
    gh release view --repo "$repo" --json tagName -q .tagName
  else
    curl -fsSL "https://api.github.com/repos/$repo/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p'
  fi
}

download() {
  local tag="$1" file="$2" dest="$3"
  if [ "$use_gh" = 1 ]; then
    gh release download "$tag" --repo "$repo" --pattern "$file" --dir "$dest" --clobber
  else
    curl -fL --progress-bar -o "$dest/$file" "https://github.com/$repo/releases/download/$tag/$file"
  fi
}

version="${wanted:-$(latest_version)}"
version="${version#v}"
[ -n "$version" ] || { echo "Impossible de trouver la dernière version (dépôt privé : connectez gh avec « gh auth login »)." >&2; exit 1; }

installed=""
[ -f "$root/current/VERSION" ] && installed="$(cat "$root/current/VERSION")"
if [ "$installed" = "$version" ] && [ "$force" = 0 ]; then
  echo "MIRA $version est déjà installé et à jour."
  exit 0
fi
if [ -n "$installed" ] && [ -z "$wanted" ] && [ "$force" = 0 ] \
   && [ "$(printf '%s\n%s\n' "$installed" "$version" | sort -V | tail -1)" = "$installed" ]; then
  echo "MIRA $installed est installé, plus récent que la dernière release ($version)."
  exit 0
fi

if [ -n "$installed" ]; then echo "Mise à jour de MIRA $installed → $version"; else echo "Installation de MIRA $version"; fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
archive="mira-mail-$version-x86_64.tar.gz"
download "v$version" "$archive" "$tmp"
download "v$version" SHA256SUMS "$tmp"
(cd "$tmp" && grep " $archive\$" SHA256SUMS | sha256sum -c --quiet -) || { echo "Somme SHA-256 incorrecte : archive rejetée." >&2; exit 1; }

mkdir -p "$root"
rm -rf "$root/$version.new"
mkdir "$root/$version.new"
tar -xzf "$tmp/$archive" -C "$root/$version.new" --strip-components=1
echo "$version" > "$root/$version.new/VERSION"
rm -rf "$root/$version"
mv "$root/$version.new" "$root/$version"
ln -sfn "$version" "$root/current"

mkdir -p "$(dirname "$bin")" "$data/applications" "$icons/512x512/apps" "$icons/scalable/apps" "$data/metainfo"
ln -sfn "$root/current/app/mira" "$bin"
sed "s|^Exec=.*|Exec=$bin|" "$root/current/share/io.github.nnoah29.Mira.desktop" > "$desktop"
install -m644 "$root/current/share/io.github.nnoah29.Mira.png" "$icons/512x512/apps/"
install -m644 "$root/current/share/io.github.nnoah29.Mira.svg" "$icons/scalable/apps/"
install -m644 "$root/current/share/io.github.nnoah29.Mira.metainfo.xml" "$data/metainfo/"
command -v update-desktop-database >/dev/null && update-desktop-database -q "$data/applications" || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$icons" || true

for old in "$root"/*/; do
  old="$(basename "$old")"
  case "$old" in current|"$version"|"${installed:-}") ;; *) rm -rf "${root:?}/$old" ;; esac
done

echo "MIRA $version installé : lancez « mira-mail » ou cherchez MIRA dans le menu des applications."
case ":$PATH:" in *":$HOME/.local/bin:"*) ;; *) echo "Ajoutez ~/.local/bin à votre PATH pour lancer « mira-mail » depuis un terminal." ;; esac
