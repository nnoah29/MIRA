#!/usr/bin/env bash
# Archive précompilée (bundle + lanceur + icônes), à publier en release GitHub : out/mira-mail-<version>-x86_64.tar.gz
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="$here/out"
version="$(cat "$out/VERSION")"
name="mira-mail-$version-x86_64"
stage="$out/$name"

rm -rf "$stage"
mkdir -p "$stage/share"
cp -a "$out/bundle" "$stage/app"
cp "$here"/common/com.toffanoah.mira.{desktop,png,svg,metainfo.xml} "$here/common/mira-daemon.service.in" "$here/../LICENSE" "$stage/share/"
tar -C "$out" -czf "$out/$name.tar.gz" "$name"
rm -rf "$stage"
sha256sum "$out/$name.tar.gz"
