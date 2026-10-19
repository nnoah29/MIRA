#!/usr/bin/env bash
# APK Android de MIRA : moteur Rust pour arm64-v8a et x86_64 (émulateur), puis app Flutter en release.
# Prérequis : NDK Android, `rustup target add aarch64-linux-android x86_64-linux-android`, `cargo install cargo-ndk`.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
ndk_root="${ANDROID_HOME:-$HOME/Android/Sdk}/ndk"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ndk_root/$(ls "$ndk_root" | sort -V | tail -1)}"

. "$root/packaging/oauth-credentials.sh"
echo "== Moteur pour Android (NDK : $ANDROID_NDK_HOME)"
(cd "$root/bridge" && cargo ndk -t arm64-v8a -t x86_64 -P 24 \
  -o "$root/app/android/app/src/main/jniLibs" build --release)

echo "== Application (APK release)"
(cd "$root/app" && flutter build apk --release --split-per-abi --target-platform android-arm64,android-x64)

mkdir -p "$root/packaging/out"
cp "$root"/app/build/app/outputs/flutter-apk/app-*-release.apk "$root/packaging/out/"
ls -lh "$root"/packaging/out/*.apk
