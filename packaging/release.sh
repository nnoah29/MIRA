#!/usr/bin/env bash
# Construit et vérifie tous les fichiers d'une release MIRA depuis son tag (docs/RELEASE.md §4 à 6) : packaging/release.sh X.Y.Z [options]
# Le build se fait dans une copie propre du tag (git worktree), jamais dans votre arbre de travail : la branche courante et vos modifications n'ont aucun effet.
set -Eeuo pipefail

usage() {
  cat <<'EOF'
Usage : packaging/release.sh X.Y.Z [options]

Prérequis : le tag annoté vX.Y.Z existe, est sur origin et dans l'historique de origin/main.
Résultat  : packaging/out/ (.deb, .rpm, .tar.gz, AppImage, 2 APK, SHA256SUMS, install.sh, journal).

  --check          vérifie seulement les prérequis ; ne construit rien, ne supprime rien
  --no-appimage    saute l'AppImage (appimagetool absent)
  --no-android     saute les APK
  --no-oauth       construit sans identifiants OAuth (essai seulement)
  --clean          repart d'un build à froid (supprime la copie de travail du tag)
  --draft          crée ensuite le brouillon de release GitHub (refusé si un format est sauté)
  --notes FICHIER  notes de la release pour --draft (défaut : /tmp/mira-release-X.Y.Z.md)
  -h, --help       cette aide

packaging/out/ est vidé avant la construction. Rien n'est publié sans --draft, et --draft ne publie jamais : il crée un brouillon.
EOF
}

version=""
check=0
appimage=1
android=1
oauth=1
clean=0
draft=0
notes=""
problems=()
warnings=()
stage="préparation"

die() {
  echo "ERREUR : $*" >&2
  exit 1
}

fail() {
  problems+=("$*")
}

warn() {
  warnings+=("$*")
}

step() {
  stage="$1"
  shift
  printf '\n== %s\n' "$stage"
  "$@"
}

trap 'echo "ÉCHEC pendant : $stage" >&2' ERR
trap '' HUP

while [ $# -gt 0 ]; do
  case "$1" in
    --check) check=1 ;;
    --no-appimage) appimage=0 ;;
    --no-android) android=0 ;;
    --no-oauth) oauth=0 ;;
    --clean) clean=1 ;;
    --draft) draft=1 ;;
    --notes)
      notes="${2:?--notes demande un fichier}"
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    -*) die "option inconnue : $1 (voir --help)" ;;
    *)
      [ -z "$version" ] || die "une seule version attendue"
      version="$1"
      ;;
  esac
  shift
done
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
  usage >&2
  exit 2
}

root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
tag="v$version"
notes="${notes:-/tmp/mira-release-$version.md}"
wt="$root/target/release-build/$tag"
out="$root/packaging/out"
wtout="$wt/packaging/out"

at_tag() {
  git -C "$root" show "$tag:$1" 2>/dev/null || true
}

tag_matches() {
  local text
  text="$(at_tag "$1")"
  grep -qE -- "$2" <<<"$text"
}

check_tag() {
  git -C "$root" fetch -q origin main || fail "impossible de joindre origin (git fetch origin main)"
  git -C "$root" rev-parse -q --verify "refs/tags/$tag" >/dev/null || {
    fail "le tag $tag n'existe pas (docs/RELEASE.md §3)"
    return 0
  }
  [ "$(git -C "$root" cat-file -t "refs/tags/$tag")" = tag ] || fail "le tag $tag n'est pas annoté"
  git -C "$root" ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null 2>&1 \
    || fail "le tag $tag n'est pas sur origin (git push origin $tag)"
  git -C "$root" merge-base --is-ancestor "$tag^{commit}" origin/main \
    || fail "le tag $tag n'est pas dans l'historique de origin/main"
}

previous_build() {
  local list t prev=""
  list="$(git -C "$root" tag --list 'v[0-9]*.[0-9]*.[0-9]*' --sort=-version:refname)"
  while read -r t; do
    [ -n "$t" ] && [ "$t" != "$tag" ] || continue
    [ "$(printf '%s\n%s\n' "$t" "$tag" | sort -V | tail -n1)" = "$tag" ] || continue
    prev="$t"
    break
  done <<<"$list"
  [ -n "$prev" ] || return 0
  git -C "$root" show "$prev:app/pubspec.yaml" 2>/dev/null | sed -n 's/^version: .*+\([0-9]*\).*/\1/p' || true
}

check_versions() {
  local pub build app prev
  pub="$(at_tag app/pubspec.yaml | sed -n 's/^version: *\([0-9.]*\)+.*/\1/p')"
  build="$(at_tag app/pubspec.yaml | sed -n 's/^version: .*+\([0-9]*\).*/\1/p')"
  app="$(at_tag app/lib/app_info.dart | sed -n "s/^const appVersion = '\(.*\)';/\1/p")"
  [ "$pub" = "$version" ] || fail "app/pubspec.yaml : version '$pub' au lieu de '$version'"
  [ "$app" = "$version" ] || fail "app/lib/app_info.dart : appVersion '$app' au lieu de '$version'"
  [ "$(at_tag packaging/VERSION | tr -d '[:space:]')" = "$version" ] || fail "packaging/VERSION différent de $version"
  tag_matches packaging/common/com.toffanoah.mira.metainfo.xml "<release version=\"$version\"" \
    || fail "metainfo.xml : aucune <release version=\"$version\">"
  tag_matches packaging/arch/PKGBUILD "^pkgver=$version\$" || fail "PKGBUILD : pkgver différent de $version"
  [ -n "$build" ] || fail "app/pubspec.yaml : numéro de build Android absent"
  prev="$(previous_build)"
  if [ -n "$build" ] && [ "${build:-0}" -le "${prev:-0}" ]; then
    fail "numéro de build Android $build non supérieur à celui de la version précédente (${prev:-0})"
  fi
}

need() {
  command -v "$1" >/dev/null || fail "outil absent : $1"
}

check_tools() {
  local t
  for t in git cargo flutter dpkg-deb rpmbuild sha256sum tar unzip; do need "$t"; done
  if [ "$appimage" = 1 ]; then
    if [ -n "${APPIMAGETOOL:-}" ] && [ -x "$APPIMAGETOOL" ]; then :; elif ! command -v appimagetool >/dev/null; then
      fail "appimagetool absent (APPIMAGETOOL=/chemin/appimagetool, ou --no-appimage)"
    fi
  fi
}

check_android() {
  local sdk="${ANDROID_HOME:-$HOME/Android/Sdk}" installed t
  command -v cargo-ndk >/dev/null || fail "cargo-ndk absent : cargo install cargo-ndk"
  [ -d "$sdk/ndk" ] && [ -n "$(ls "$sdk/ndk")" ] || fail "NDK Android introuvable dans $sdk/ndk (ANDROID_HOME)"
  installed="$(rustup target list --installed 2>/dev/null || true)"
  for t in aarch64-linux-android x86_64-linux-android; do
    grep -qx "$t" <<<"$installed" || fail "cible Rust absente : rustup target add $t"
  done
  warn "les APK sont signés avec la clé de debug : essais directs seulement, pas de Play Store"
}

check_oauth() {
  local cfg
  if [ "$oauth" = 0 ]; then
    export MIRA_OAUTH_CONFIG=
    warn "construit SANS identifiants OAuth : ne pas publier ces binaires"
    return 0
  fi
  cfg="${MIRA_OAUTH_CONFIG-${XDG_CONFIG_HOME:-$HOME/.config}/mira/config.toml}"
  if [ -n "$cfg" ] && [ -f "$cfg" ] && grep -q '^\[oauth\.' "$cfg"; then
    export MIRA_OAUTH_CONFIG="$cfg"
  else
    fail "configuration OAuth de production absente ou sans section [oauth.*] (MIRA_OAUTH_CONFIG, ou --no-oauth pour un essai)"
  fi
}

check_draft() {
  [ "$draft" = 1 ] || return 0
  if [ "$appimage" = 0 ] || [ "$android" = 0 ] || [ "$oauth" = 0 ]; then
    fail "--draft exige tous les formats et les identifiants OAuth (créez le brouillon à la main pour une release partielle)"
  fi
  command -v gh >/dev/null || fail "gh absent (--draft)"
  gh auth status >/dev/null 2>&1 || fail "gh non connecté (gh auth login)"
  [ -s "$notes" ] || fail "notes de release absentes ou vides : $notes (--notes FICHIER)"
  if gh release view "$tag" >/dev/null 2>&1; then fail "une release $tag existe déjà sur GitHub"; fi
}

report_problems() {
  if [ "${#problems[@]}" -gt 0 ]; then
    echo "Prérequis manquants :" >&2
    printf '  - %s\n' "${problems[@]}" >&2
    exit 1
  fi
}

preflight() {
  step "Prérequis pour $tag" check_tag
  report_problems
  check_versions
  check_tools
  if [ "$android" = 1 ]; then check_android; fi
  check_oauth
  check_draft
  if [ "${#warnings[@]}" -gt 0 ]; then printf 'Avertissement : %s\n' "${warnings[@]}" >&2; fi
  report_problems
  echo "Prérequis : OK"
}

prepare_tree() {
  if [ "$clean" = 1 ] && [ -e "$wt" ]; then git -C "$root" worktree remove --force "$wt"; fi
  git -C "$root" worktree prune
  if [ -e "$wt/.git" ]; then
    git -C "$wt" checkout -q --detach -f "$tag"
    git -C "$wt" reset -q --hard "$tag"
    git -C "$wt" clean -qfd
  else
    mkdir -p "$(dirname "$wt")"
    git -C "$root" worktree add -q --detach "$wt" "$tag"
  fi
  [ "$(git -C "$wt" rev-parse HEAD)" = "$(git -C "$root" rev-parse "$tag^{commit}")" ] || die "la copie de travail n'est pas sur $tag"
  [ -z "$(git -C "$wt" status --porcelain)" ] || die "la copie de travail de $tag est modifiée"
  rm -rf "$wtout"
  mkdir -p "$wtout"
}

build_android() {
  (cd "$wt" && packaging/android/build.sh)
  mv "$wtout/app-arm64-v8a-release.apk" "$wtout/mira-mail-$version-android-arm64-v8a.apk"
  mv "$wtout/app-x86_64-release.apk" "$wtout/mira-mail-$version-android-x86_64.apk"
}

build_all() {
  step "Bundle Linux (moteur Rust + app Flutter)" bash -c 'cd "$1" && packaging/build-bundle.sh' _ "$wt"
  step "Paquet .deb" bash -c 'cd "$1" && packaging/deb/build.sh' _ "$wt"
  step "Paquet .rpm" bash -c 'cd "$1" && packaging/rpm/build.sh' _ "$wt"
  step "Archive .tar.gz" bash -c 'cd "$1" && packaging/tarball.sh' _ "$wt"
  if [ "$appimage" = 1 ]; then step "AppImage" bash -c 'cd "$1" && packaging/appimage/build.sh' _ "$wt"; fi
  if [ "$android" = 1 ]; then step "APK Android" build_android; fi
}

contains() {
  grep -qF -- "$2" <<<"$1"
}

expect() {
  local what="$1"
  shift
  "$@" || die "contrôle échoué : $what"
}

verify_bundle() {
  local b="$wtout/bundle" build ver
  build="$(at_tag app/pubspec.yaml | sed -n 's/^version: .*+\([0-9]*\).*/\1/p')"
  ver="$(cat "$b/data/flutter_assets/version.json")"
  expect "bundle : exécutable mira" test -x "$b/mira"
  expect "bundle : moteur libmira_bridge.so" test -f "$b/lib/libmira_bridge.so"
  expect "bundle : libdartjni.so absent" test ! -e "$b/lib/libdartjni.so"
  expect "bundle : version $version dans version.json" contains "$ver" "\"version\":\"$version\""
  expect "bundle : build $build dans version.json" contains "$ver" "\"build_number\":\"$build\""
}

verify_deb() {
  local f="$wtout/mira-mail_${version}_amd64.deb" listing
  expect ".deb présent" test -f "$f"
  expect ".deb : version" test "$(dpkg-deb -f "$f" Version)" = "$version"
  expect ".deb : nom" test "$(dpkg-deb -f "$f" Package)" = mira-mail
  listing="$(dpkg-deb -c "$f")"
  expect ".deb : moteur" contains "$listing" usr/lib/mira-mail/lib/libmira_bridge.so
}

verify_rpm() {
  local f="$wtout/mira-mail-${version}-1.x86_64.rpm" listing
  expect ".rpm présent" test -f "$f"
  expect ".rpm : version" test "$(rpm -qp --qf '%{VERSION}' "$f" 2>/dev/null)" = "$version"
  listing="$(rpm -qlp "$f" 2>/dev/null)"
  expect ".rpm : moteur" contains "$listing" libmira_bridge.so
}

verify_tarball() {
  local f="$wtout/mira-mail-${version}-x86_64.tar.gz" listing n="mira-mail-${version}-x86_64"
  expect "archive présente" test -f "$f"
  listing="$(tar -tzf "$f")"
  expect "archive : application" contains "$listing" "$n/app/mira"
  expect "archive : moteur" contains "$listing" "$n/app/lib/libmira_bridge.so"
  expect "archive : fiche AppStream" contains "$listing" "$n/share/com.toffanoah.mira.metainfo.xml"
}

verify_appimage() {
  local f="$wtout/MIRA-${version}-x86_64.AppImage"
  expect "AppImage présente" test -f "$f"
  expect "AppImage exécutable" test -x "$f"
  expect "AppImage : binaire ELF" test "$(head -c4 "$f" | tail -c3)" = ELF
}

verify_apks() {
  local abi f listing
  for abi in arm64-v8a x86_64; do
    f="$wtout/mira-mail-$version-android-$abi.apk"
    expect "APK $abi présent" test -f "$f"
    listing="$(unzip -l "$f")"
    expect "APK $abi : moteur" contains "$listing" "lib/$abi/libmira_bridge.so"
  done
}

verify_all() {
  verify_bundle
  verify_deb
  verify_rpm
  verify_tarball
  if [ "$appimage" = 1 ]; then verify_appimage; fi
  if [ "$android" = 1 ]; then verify_apks; fi
  echo "Contrôles des paquets : OK"
}

assets=()

collect_assets() {
  assets=("mira-mail_${version}_amd64.deb" "mira-mail-${version}-1.x86_64.rpm" "mira-mail-${version}-x86_64.tar.gz")
  if [ "$appimage" = 1 ]; then assets+=("MIRA-${version}-x86_64.AppImage"); fi
  if [ "$android" = 1 ]; then assets+=("mira-mail-${version}-android-arm64-v8a.apk" "mira-mail-${version}-android-x86_64.apk"); fi
}

publish_files() {
  local f
  collect_assets
  cp "$wt/packaging/install.sh" "$wtout/install.sh"
  (cd "$wtout" && sha256sum "${assets[@]}" install.sh >SHA256SUMS)
  for f in "${assets[@]}" install.sh SHA256SUMS; do cp "$wtout/$f" "$out/$f"; done
  (cd "$out" && sha256sum -c --quiet SHA256SUMS)
  echo "SHA256SUMS : OK"
}

create_draft() {
  local f files=()
  for f in "${assets[@]}" SHA256SUMS install.sh; do files+=("$out/$f"); done
  (cd "$root" && gh release create "$tag" "${files[@]}" --title "MIRA $version" \
    --notes-file "$notes" --verify-tag --draft)
}

summary() {
  echo
  echo "== Terminé en $((SECONDS / 60)) min $((SECONDS % 60)) s : $out"
  (cd "$out" && ls -lh "${assets[@]}" SHA256SUMS install.sh)
  echo
  echo "Somme de l'archive pour l'AUR (packaging/arch/PKGBUILD, à la place de SKIP) :"
  (cd "$out" && grep " mira-mail-${version}-x86_64.tar.gz\$" SHA256SUMS)
  echo
  echo "Reste à faire à la main (docs/RELEASE.md §5 à 7) : installer et lancer les paquets sur une machine propre,"
  echo "vérifier le brouillon GitHub, le publier et le marquer Latest, puis mettre à jour l'AUR."
  if [ "${#warnings[@]}" -gt 0 ]; then printf 'Avertissement : %s\n' "${warnings[@]}"; fi
}

preflight
if [ "$check" = 1 ]; then exit 0; fi

rm -rf "$out"
mkdir -p "$out"
exec > >(tee -a "$out/release-$version.log") 2>&1

step "Copie de travail propre de $tag" prepare_tree
build_all
step "Contrôle des paquets" verify_all
step "Somme de contrôle et fichiers à publier" publish_files
if [ "$draft" = 1 ]; then step "Brouillon de release GitHub" create_draft; fi
summary
