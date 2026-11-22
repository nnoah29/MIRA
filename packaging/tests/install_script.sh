#!/usr/bin/env bash
# Teste install.sh en vase clos : HOME isolé, archives fabriquées, faux gestionnaire de paquets, aucun réseau.
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
script="$here/../install.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fails=0
ok() { echo "  ok  : $*"; }
ko() { echo "  ÉCHEC : $*" >&2; fails=$((fails + 1)); }
has() { grep -q -- "$1" <<<"$out"; }
check() { if eval "$2"; then ok "$1"; else ko "$1"; fi; }

make_release() {
  local version="$1" dir="$2" mode="${3:-good}" src="$tmp/src-$1-${3:-good}"
  rm -rf "$src"
  mkdir -p "$src/mira-mail-$version/app" "$src/mira-mail-$version/share" "$dir"
  printf '#!/bin/sh\necho %s\n' "$version" > "$src/mira-mail-$version/app/mira"
  printf '#!/bin/sh\n' > "$src/mira-mail-$version/app/mira-daemon"
  chmod 755 "$src/mira-mail-$version/app/mira" "$src/mira-mail-$version/app/mira-daemon"
  local share="$src/mira-mail-$version/share"
  printf '[Desktop Entry]\nName=MIRA\nExec=mira-mail\n' > "$share/com.toffanoah.mira.desktop"
  : > "$share/com.toffanoah.mira.png"; : > "$share/com.toffanoah.mira.svg"; : > "$share/com.toffanoah.mira.metainfo.xml"
  printf '[Service]\nExecStart=@DAEMON@\n' > "$share/mira-daemon.service.in"
  [ "$mode" != "nobinary" ] || rm "$src/mira-mail-$version/app/mira"
  [ "$mode" != "nometa" ] || rm "$share/com.toffanoah.mira.metainfo.xml"
  [ "$mode" != "traversal" ] || printf 'x' > "$src/evil"
  local archive="mira-mail-$version-x86_64.tar.gz"
  if [ "$mode" = "traversal" ]; then
    tar -czf "$dir/$archive" -C "$src" --transform 's|^evil|mira-mail-'"$version"'/../evil|' "mira-mail-$version" evil 2>/dev/null
  else
    tar -czf "$dir/$archive" -C "$src" "mira-mail-$version"
  fi
  (cd "$dir" && sha256sum "$archive" > SHA256SUMS)
  [ "$mode" != "badsum" ] || printf '%064d  %s\n' 0 "$archive" > "$dir/SHA256SUMS"
}

new_env() {
  rm -rf "$tmp/home" "$tmp/bin" "$tmp/sysroot"
  mkdir -p "$tmp/home" "$tmp/bin" "$tmp/sysroot"
  export HOME="$tmp/home"
  unset XDG_DATA_HOME XDG_CONFIG_HOME
  export PATH="$tmp/bin:$HOME/.local/bin:/usr/bin:/bin"
  export MIRA_SYSTEM_ROOT="$tmp/sysroot"
  local tool
  for tool in pacman dpkg-query rpm; do printf '#!/bin/sh\nexit 1\n' > "$tmp/bin/$tool"; chmod +x "$tmp/bin/$tool"; done
}

run() { MIRA_INSTALL_FROM="$1" bash "$script" "${@:2}" 2>&1; }
root() { echo "$HOME/.local/share/mira-mail"; }

make_release 1.0.0 "$tmp/r100"
make_release 1.1.0 "$tmp/r110"
make_release 1.2.0 "$tmp/bad-sum" badsum
make_release 1.3.0 "$tmp/no-bin" nobinary
make_release 1.4.0 "$tmp/no-meta" nometa
make_release 1.5.0 "$tmp/trav" traversal

echo "# installation neuve"
new_env
out="$(run "$tmp/r100" 1.0.0)"; code=$?
check "code 0" "[ $code = 0 ]"
check "version enregistrée" "[ \"\$(cat $(root)/current/VERSION)\" = 1.0.0 ]"
check "lanceur pointe sur current" "[ \"\$(readlink $HOME/.local/bin/mira-mail)\" = \"$(root)/current/app/mira\" ]"
check "Exec du .desktop vise le lanceur" "grep -qx 'Exec=$HOME/.local/bin/mira-mail' $HOME/.local/share/applications/com.toffanoah.mira.desktop"
check "unité systemd vise current" "grep -qx 'ExecStart=$(root)/current/app/mira-daemon' $HOME/.config/systemd/user/mira-daemon.service"
check "aucun résidu .stage" "! ls -A $(root) | grep -q '^.stage'"

echo "# relance : déjà à jour"
out="$(run "$tmp/r100" 1.0.0)"
check "message déjà installé" "has 'déjà installé'"

echo "# mise à jour 1.0.0 -> 1.1.0 : une seule version reste sur la machine"
out="$(run "$tmp/r110" 1.1.0)"
check "current en 1.1.0" "[ \"\$(cat $(root)/current/VERSION)\" = 1.1.0 ]"
check "une seule version conservée" "[ \$(ls -d $(root)/[0-9]*/ | wc -l) = 1 ]"
check "l'ancienne est supprimée" "! ls $(root) | grep -q '^1.0.0'"

echo "# --force réinstalle la même version"
out="$(run "$tmp/r110" 1.1.0 --force)"
check "réinstallée" "has 'installé'"
check "toujours une seule version" "[ \$(ls -d $(root)/[0-9]*/ | wc -l) = 1 ]"

echo "# somme SHA-256 fausse : rien ne bouge"
before="$(readlink "$(root)/current")"
out="$(run "$tmp/bad-sum" 1.2.0)"; code=$?
check "refusée (code non nul)" "[ $code != 0 ]"
check "message somme" "has 'SHA-256'"
check "current inchangé" "[ \"\$(readlink $(root)/current)\" = \"$before\" ]"

echo "# archive sans binaire / sans métadonnées : rien ne bouge"
out="$(run "$tmp/no-bin" 1.3.0)"; code=$?
check "sans binaire refusée" "[ $code != 0 ] && has 'app/mira'"
out="$(run "$tmp/no-meta" 1.4.0)"; code=$?
check "sans métadonnées refusée" "[ $code != 0 ] && has 'incomplète'"
check "current toujours en 1.1.0" "[ \"\$(cat $(root)/current/VERSION)\" = 1.1.0 ]"

echo "# archive avec chemin .. : rejetée"
out="$(run "$tmp/trav" 1.5.0)"; code=$?
check "chemin dangereux refusé" "[ $code != 0 ] && has 'chemins'"
check "rien écrit hors de l'arbre" "[ ! -e $(root)/evil ] && [ ! -e $(root)/../evil ]"

echo "# arguments invalides"
out="$(run "$tmp/r100" ../../etc)"; code=$?
check "version avec .. refusée" "[ $code != 0 ] && has 'invalide'"
out="$(run "$tmp/r100" --nimporte)"; code=$?
check "option inconnue refusée" "[ $code != 0 ] && has 'option inconnue'"
out="$(run "$tmp/r100" 1.0.0 1.1.0)"; code=$?
check "deux versions refusées" "[ $code != 0 ]"

echo "# installation système détectée (pacman)"
new_env
printf '#!/bin/sh\n[ "$1" = "-Q" ] && [ "$2" = "mira-mail" ] && echo "mira-mail-dev 2.3.1" && exit 0\nexit 1\n' > "$tmp/bin/pacman"
chmod +x "$tmp/bin/pacman"
out="$(run "$tmp/r100" 1.0.0)"; code=$?
check "code 2" "[ $code = 2 ]"
check "propose pacman -R" "has 'sudo pacman -R mira-mail-dev'"
check "rien installé" "[ ! -e $(root) ] && [ ! -e $HOME/.local/bin/mira-mail ]"

echo "# installation système détectée (fichiers)"
new_env
mkdir -p "$tmp/sysroot/usr/lib/mira-mail"; : > "$tmp/sysroot/usr/lib/mira-mail/mira"
out="$(run "$tmp/r100" 1.0.0)"; code=$?
check "refusée par les fichiers" "[ $code = 2 ] && has 'gestionnaire de paquets'"

echo "# lanceur étranger dans ~/.local/bin"
new_env
mkdir -p "$HOME/.local/bin"; echo "mien" > "$HOME/.local/bin/mira-mail"
out="$(run "$tmp/r100" 1.0.0)"; code=$?
check "refusé sans --force" "[ $code != 0 ] && has \"n'appartient pas\""
check "fichier de l'utilisateur intact" "[ \"\$(cat $HOME/.local/bin/mira-mail)\" = mien ]"
out="$(run "$tmp/r100" 1.0.0 --force)"; code=$?
check "remplacé avec --force" "[ $code = 0 ] && [ -L $HOME/.local/bin/mira-mail ]"

echo "# autre mira-mail devant dans le PATH : avertissement"
new_env
printf '#!/bin/sh\n' > "$tmp/bin/mira-mail"; chmod +x "$tmp/bin/mira-mail"
out="$(run "$tmp/r100" 1.0.0)"
check "avertit du doublon" "has 'lance .* et non cette installation'"

echo "# root refusé"
if [ "$(id -u)" = 0 ]; then
  out="$(run "$tmp/r100" 1.0.0)"; check "root refusé" "has 'root'"
else
  ok "ignoré (le test ne tourne pas en root)"
fi

echo "# désinstallation"
new_env
run "$tmp/r100" 1.0.0 >/dev/null
out="$(run "$tmp/r100" --uninstall)"; code=$?
check "code 0" "[ $code = 0 ]"
check "tout retiré" "[ ! -e $(root) ] && [ ! -e $HOME/.local/bin/mira-mail ] && [ ! -e $HOME/.config/systemd/user/mira-daemon.service ]"

if [ "$fails" = 0 ]; then echo "OK : install.sh"; else echo "$fails échec(s)" >&2; exit 1; fi
