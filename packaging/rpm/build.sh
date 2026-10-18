#!/usr/bin/env bash
# Paquet .rpm à partir du bundle : packaging/out/mira-mail-<version>-1.x86_64.rpm
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="$(cd "$here/../out" && pwd)"
top="$out/rpmbuild"

rm -rf "$top"
mkdir -p "$top"
rpmbuild -bb "$here/mira-mail.spec" \
  --define "_topdir $top" \
  --define "mira_version $(cat "$out/VERSION")" \
  --define "mira_bundle $out/bundle" \
  --define "mira_packaging $(cd "$here/.." && pwd)"
find "$top/RPMS" -name '*.rpm' -exec cp {} "$out/" \;
ls "$out"/*.rpm
