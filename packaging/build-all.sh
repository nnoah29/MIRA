#!/usr/bin/env bash
# Construit tous les paquets réalisables sur cette machine dans packaging/out/.
# Formats natifs : .deb, .rpm, archive précompilée (PKGBUILD AUR). AppImage si appimagetool est disponible.
# Flatpak et Snap : voir packaging/README.md (outils et runtimes dédiés).
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
"$here/build-bundle.sh"
"$here/deb/build.sh"
command -v rpmbuild >/dev/null && "$here/rpm/build.sh" || echo "rpmbuild absent : .rpm ignoré"
"$here/tarball.sh"
if [ -n "${APPIMAGETOOL:-}" ] || command -v appimagetool >/dev/null; then
  "$here/appimage/build.sh"
else
  echo "appimagetool absent : AppImage ignorée"
fi
ls -lh "$here/out" | grep -E '\.(deb|rpm|tar\.gz|AppImage)$'
