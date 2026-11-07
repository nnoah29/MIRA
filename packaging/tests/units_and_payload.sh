#!/usr/bin/env bash
# Vérifie que l'arborescence de paquet contient le démon et son unité systemd --user, sans motif restant.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fail() { echo "ÉCHEC : $*" >&2; exit 1; }

bundle="$tmp/bundle"
mkdir -p "$bundle/lib" "$bundle/data"
printf '#!/bin/sh\n' > "$bundle/mira"
printf '#!/bin/sh\n' > "$bundle/mira-daemon"
chmod 644 "$bundle/mira" "$bundle/mira-daemon"

"$here/../common/install-tree.sh" "$bundle" "$tmp/root" /usr

daemon="$tmp/root/usr/lib/mira-mail/mira-daemon"
unit="$tmp/root/usr/lib/systemd/user/mira-daemon.service"
[ -x "$daemon" ] || fail "mira-daemon absent ou non exécutable"
[ -f "$unit" ] || fail "unité systemd --user absente"
grep -qx 'ExecStart=/usr/lib/mira-mail/mira-daemon' "$unit" || fail "ExecStart ne vise pas le démon installé"
grep -qx 'Restart=on-failure' "$unit" || fail "Restart=on-failure manquant"
grep -qx 'WantedBy=default.target' "$unit" || fail "WantedBy=default.target manquant"
if grep -q '@' "$unit"; then fail "motif @…@ non remplacé dans l'unité"; fi

other="$tmp/other"
"$here/../common/install-tree.sh" "$bundle" "$other" /opt/mira
grep -qx 'ExecStart=/opt/mira/lib/mira-mail/mira-daemon' "$other/opt/mira/lib/systemd/user/mira-daemon.service" \
  || fail "l'unité ne suit pas le préfixe d'installation"

if command -v systemd-analyze >/dev/null; then
  checked="$tmp/mira-daemon.service"
  sed 's|^ExecStart=.*|ExecStart=/usr/bin/true|' "$unit" > "$checked"
  systemd-analyze --user verify "$checked" || fail "systemd-analyze verify"
fi
echo "OK : démon et unité présents et valides"
