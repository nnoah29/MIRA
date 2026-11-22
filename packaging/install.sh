#!/usr/bin/env bash
# Installe MIRA pour l'utilisateur courant (sans sudo) depuis une release GitHub : un seul emplacement, ~/.local/share/mira-mail.
# Refuse de s'installer si MIRA est déjà installé par le système (pacman, apt, dnf) ; relancé, il met à jour. `--uninstall` retire l'installation.
# Usage : install.sh [--force] [--uninstall] [--help] [version]   (ex. install.sh 1.0.0)
# Hors ligne : MIRA_INSTALL_FROM=<dossier contenant l'archive et SHA256SUMS> install.sh <version>
set -euo pipefail
umask 022

repo="nnoah29/MIRA"
home_dir="${HOME:-}"
data="${XDG_DATA_HOME:-$home_dir/.local/share}"
root="$data/mira-mail"
bin="$home_dir/.local/bin/mira-mail"
desktop="$data/applications/com.toffanoah.mira.desktop"
unit_dir="${XDG_CONFIG_HOME:-$home_dir/.config}/systemd/user"
unit="$unit_dir/mira-daemon.service"
icons="$data/icons/hicolor"
sysroot="${MIRA_SYSTEM_ROOT:-}"
force=0
uninstall=0
wanted=""
stage=""
rollback_to=""

die() { echo "Erreur : $*" >&2; exit 1; }

cleanup() {
  local code=$?
  [ -z "$stage" ] || rm -rf "$stage"
  [ "$code" -ne 0 ] && [ -n "$rollback_to" ] || return 0
  if [ "$rollback_to" = "-" ]; then
    rm -f "$root/current"
  elif [ -d "$root/$rollback_to" ]; then
    ln -sfn "$rollback_to" "$root/.current.undo" && mv -Tf "$root/.current.undo" "$root/current" || true
  fi
  echo "L'installation a échoué : l'état précédent est rétabli." >&2
}
trap cleanup EXIT

parse_args() {
  local arg
  for arg in "$@"; do
    case "$arg" in
      --force) force=1 ;;
      --uninstall) uninstall=1 ;;
      -h|--help) sed -n '2,5p' "$0"; exit 0 ;;
      -*) die "option inconnue « $arg » (voir --help)." ;;
      *) [ -z "$wanted" ] || die "une seule version peut être demandée."; wanted="${arg#v}" ;;
    esac
  done
}

check_environment() {
  [ "$(id -u)" != 0 ] || die "ne lancez pas ce script avec sudo ou en root : il installe pour l'utilisateur courant, sans privilèges."
  [ -n "$home_dir" ] && [ -d "$home_dir" ] || die "le dossier personnel (HOME) est introuvable."
  [ "$(uname -s)" = "Linux" ] && [ "$(uname -m)" = "x86_64" ] || die "seul Linux x86_64 est publié."
  local tool
  for tool in tar sha256sum mktemp sort awk grep sed mv ln; do
    command -v "$tool" >/dev/null 2>&1 || die "l'outil « $tool » est requis et introuvable."
  done
  if [ -z "${MIRA_INSTALL_FROM:-}" ] && ! command -v gh >/dev/null 2>&1 && ! command -v curl >/dev/null 2>&1; then
    die "« gh » ou « curl » est requis pour télécharger MIRA."
  fi
}

valid_version() {
  case "$1" in
    ''|*[!0-9.]*|.*|*.|*..*) return 1 ;;
    *.*) return 0 ;;
    *) return 1 ;;
  esac
}

system_package() {
  local name owner
  for name in mira-mail mira-mail-bin mira-mail-dev mira-mail-git; do
    if command -v pacman >/dev/null 2>&1 && owner="$(pacman -Q "$name" 2>/dev/null)" && [ -n "$owner" ]; then
      echo "pacman|${owner%% *}"; return 0
    fi
  done
  if command -v dpkg-query >/dev/null 2>&1 && dpkg-query -W -f='${db:Status-Abbrev}' mira-mail 2>/dev/null | grep -q '^ii'; then
    echo "apt|mira-mail"; return 0
  fi
  if command -v rpm >/dev/null 2>&1 && rpm -q mira-mail >/dev/null 2>&1; then
    echo "dnf|mira-mail"; return 0
  fi
  if [ -e "$sysroot/usr/lib/mira-mail/mira" ] || [ -e "$sysroot/usr/bin/mira-mail" ] || [ -e "$sysroot/opt/mira-mail" ]; then
    echo "fichiers|mira-mail"; return 0
  fi
  return 1
}

removal_hint() {
  case "$1" in
    pacman) echo "sudo pacman -R $2" ;;
    apt) echo "sudo apt remove $2" ;;
    dnf) echo "sudo dnf remove $2" ;;
    *) echo "retirez-le avec votre gestionnaire de paquets (fichiers sous /usr/lib/mira-mail ou /opt/mira-mail)" ;;
  esac
}

refuse_if_system_install() {
  local found manager name
  found="$(system_package)" || return 0
  manager="${found%%|*}"
  name="${found#*|}"
  {
    echo "MIRA est déjà installé par le système (« $name »)."
    echo "Deux installations en parallèle se disputeraient le lanceur et le démon : MIRA n'a qu'un seul emplacement."
    echo "Pour l'utiliser : mettez-le à jour avec votre gestionnaire de paquets."
    echo "Pour passer à l'installation utilisateur : $(removal_hint "$manager" "$name"), puis relancez ce script."
  } >&2
  exit 2
}

latest_version() {
  if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    gh release view --repo "$repo" --json tagName -q .tagName
  else
    curl -fsSL --retry 3 --connect-timeout 15 "https://api.github.com/repos/$repo/releases/latest" \
      | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p'
  fi
}

fetch() {
  local tag="$1" file="$2" dest="$3" progress="--silent"
  if [ -n "${MIRA_INSTALL_FROM:-}" ]; then
    [ -f "$MIRA_INSTALL_FROM/$file" ] || die "« $file » est absent de $MIRA_INSTALL_FROM."
    cp "$MIRA_INSTALL_FROM/$file" "$dest/$file"
  elif command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    gh release download "$tag" --repo "$repo" --pattern "$file" --dir "$dest" --clobber \
      || die "téléchargement de « $file » impossible (version $tag inexistante ?)."
  else
    if [ -t 2 ]; then progress="--progress-bar"; fi
    curl -fL "$progress" --show-error --retry 3 --retry-delay 2 --connect-timeout 15 --max-time 900 \
      -o "$dest/$file" "https://github.com/$repo/releases/download/$tag/$file" \
      || die "téléchargement de « $file » impossible (version $tag inexistante, ou dépôt privé : connectez gh avec « gh auth login »)."
  fi
  [ -s "$dest/$file" ] || die "« $file » téléchargé est vide."
}

verify_checksum() {
  local dir="$1" archive="$2" line
  line="$(awk -v f="$archive" '$2 == f || $2 == "*" f' "$dir/SHA256SUMS")"
  [ -n "$line" ] && [ "$(printf '%s\n' "$line" | wc -l)" = 1 ] \
    || die "« $archive » ne figure pas (ou figure plusieurs fois) dans SHA256SUMS : archive rejetée."
  (cd "$dir" && printf '%s\n' "$line" | sha256sum -c --quiet -) || die "somme SHA-256 incorrecte : archive rejetée."
}

extract_archive() {
  local archive="$1" dest="$2"
  tar -tzf "$archive" > "$stage/entries" 2>/dev/null || die "l'archive est illisible ou corrompue."
  if grep -qE '(^/|(^|/)\.\.(/|$))' "$stage/entries"; then
    die "l'archive contient des chemins absolus ou « .. » : rejetée."
  fi
  mkdir -p "$dest"
  tar -xzf "$archive" -C "$dest" --strip-components=1 --no-same-owner || die "extraction de l'archive impossible (disque plein ?)."
}

check_payload() {
  local tree="$1" file
  [ -x "$tree/app/mira" ] || die "l'archive ne contient pas « app/mira » exécutable : installation annulée, rien n'a été modifié."
  for file in com.toffanoah.mira.desktop com.toffanoah.mira.png com.toffanoah.mira.svg com.toffanoah.mira.metainfo.xml; do
    [ -f "$tree/share/$file" ] || die "l'archive est incomplète (« share/$file » manque) : installation annulée, rien n'a été modifié."
  done
}

refuse_foreign_launcher() {
  [ -e "$bin" ] || [ -L "$bin" ] || return 0
  [ "$force" = 0 ] || return 0
  case "$(readlink "$bin" 2>/dev/null || true)" in
    "$root"/*) return 0 ;;
  esac
  die "$bin existe déjà et n'appartient pas à MIRA. Déplacez-le, ou relancez avec --force pour le remplacer."
}

switch_current() {
  local target="$1"
  ln -sfn "$target" "$root/.current.new"
  mv -Tf "$root/.current.new" "$root/current"
}

link_launcher() {
  mkdir -p "$(dirname "$bin")"
  ln -sfn "$root/current/app/mira" "$bin.new"
  mv -Tf "$bin.new" "$bin"
}

install_desktop_files() {
  local share="$root/current/share"
  mkdir -p "$data/applications" "$icons/512x512/apps" "$icons/scalable/apps" "$data/metainfo"
  sed "s|^Exec=.*|Exec=$bin|" "$share/com.toffanoah.mira.desktop" > "$desktop"
  install -m644 "$share/com.toffanoah.mira.png" "$icons/512x512/apps/"
  install -m644 "$share/com.toffanoah.mira.svg" "$icons/scalable/apps/"
  install -m644 "$share/com.toffanoah.mira.metainfo.xml" "$data/metainfo/"
  command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database -q "$data/applications" || true
  command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t "$icons" || true
}

install_daemon_unit() {
  [ -f "$root/current/share/mira-daemon.service.in" ] || return 0
  mkdir -p "$unit_dir"
  sed "s|@DAEMON@|$root/current/app/mira-daemon|" "$root/current/share/mira-daemon.service.in" > "$unit"
  command -v systemctl >/dev/null 2>&1 && systemctl --user daemon-reload >/dev/null 2>&1 || true
}

prune_old_versions() {
  local keep="$1" dir name
  for dir in "$root"/*/; do
    name="$(basename "$dir")"
    case "$name" in current|"$keep") ;; *) rm -rf "${root:?}/$name" ;; esac
  done
}

report_shadowing() {
  local resolved
  case ":$PATH:" in
    *":$home_dir/.local/bin:"*) ;;
    *) echo "Ajoutez ~/.local/bin à votre PATH pour lancer « mira-mail » depuis un terminal."; return 0 ;;
  esac
  resolved="$(command -v mira-mail 2>/dev/null || true)"
  if [ -n "$resolved" ] && [ "$resolved" != "$bin" ]; then
    echo "Attention : « mira-mail » lance $resolved et non cette installation. Retirez l'autre copie pour n'en garder qu'une." >&2
  fi
}

report_running_daemon() {
  if command -v pgrep >/dev/null 2>&1 && pgrep -u "$(id -u)" -x mira-daemon >/dev/null 2>&1; then
    echo "Un démon MIRA tourne encore avec l'ancienne version : fermez MIRA, ou « systemctl --user restart mira-daemon.service »."
  fi
}

uninstall_all() {
  local found
  command -v systemctl >/dev/null 2>&1 && systemctl --user disable --now mira-daemon.service >/dev/null 2>&1 || true
  rm -f "$unit"
  rm -rf "${root:?}" "$bin" "$bin.new" "$desktop" \
    "$icons/512x512/apps/com.toffanoah.mira.png" "$icons/scalable/apps/com.toffanoah.mira.svg" \
    "$data/metainfo/com.toffanoah.mira.metainfo.xml"
  command -v systemctl >/dev/null 2>&1 && systemctl --user daemon-reload >/dev/null 2>&1 || true
  echo "MIRA désinstallé. Vos comptes et mails restent dans ~/.local/share et ~/.config."
  if found="$(system_package)"; then echo "Une installation système subsiste (« ${found#*|} »)." >&2; fi
}

resolve_version() {
  local version
  version="${wanted:-$(latest_version)}"
  version="${version#v}"
  [ -n "$version" ] || die "impossible de trouver la dernière version (dépôt privé : connectez gh avec « gh auth login »)."
  valid_version "$version" || die "numéro de version invalide « $version » (attendu : 1.2.3)."
  echo "$version"
}

skip_if_up_to_date() {
  local version="$1" installed="$2"
  if [ "$installed" = "$version" ] && [ "$force" = 0 ]; then
    echo "MIRA $version est déjà installé et à jour."
    exit 0
  fi
  if [ -n "$installed" ] && [ -z "$wanted" ] && [ "$force" = 0 ] \
     && [ "$(printf '%s\n%s\n' "$installed" "$version" | sort -V | tail -1)" = "$installed" ]; then
    echo "MIRA $installed est installé, plus récent que la dernière release ($version)."
    exit 0
  fi
}

lock_install_dir() {
  mkdir -p "$root"
  if command -v flock >/dev/null 2>&1; then
    exec 9>"$root/.lock"
    flock -n 9 || die "une autre installation de MIRA est en cours."
  fi
  rm -rf "$root"/.stage.* "$root/.current.new" "$root/.current.undo"
}

install_release() {
  local version="$1" installed="$2" archive previous new_dir
  archive="mira-mail-$version-x86_64.tar.gz"
  stage="$(mktemp -d "$root/.stage.XXXXXX")"
  fetch "v$version" "$archive" "$stage"
  fetch "v$version" SHA256SUMS "$stage"
  verify_checksum "$stage" "$archive"
  extract_archive "$stage/$archive" "$stage/tree"
  check_payload "$stage/tree"
  echo "$version" > "$stage/tree/VERSION"
  previous="$(readlink "$root/current" 2>/dev/null || true)"
  new_dir="$version.$(date +%s)"
  mv "$stage/tree" "$root/$new_dir"
  rollback_to="${previous:--}"
  switch_current "$new_dir"
  link_launcher
  install_desktop_files
  install_daemon_unit
  rollback_to=""
  prune_old_versions "$new_dir"
  echo "MIRA $version installé : lancez « mira-mail » ou cherchez MIRA dans le menu des applications."
  report_shadowing
  if [ -n "$installed" ]; then report_running_daemon; fi
}

main() {
  parse_args "$@"
  check_environment
  if [ "$uninstall" = 1 ]; then uninstall_all; exit 0; fi
  refuse_if_system_install
  local version installed=""
  version="$(resolve_version)"
  if [ -f "$root/current/VERSION" ]; then installed="$(cat "$root/current/VERSION")"; fi
  skip_if_up_to_date "$version" "$installed"
  refuse_foreign_launcher
  lock_install_dir
  if [ -n "$installed" ]; then echo "Mise à jour de MIRA $installed → $version"; else echo "Installation de MIRA $version"; fi
  install_release "$version" "$installed"
}

main "$@"
