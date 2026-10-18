# Paquets Linux de MIRA

Tous les formats partent du même **bundle** : l'application Flutter compilée en release, plus la bibliothèque du moteur (`libmira_bridge.so`) dans `lib/`. L'application cherche le moteur à côté de son exécutable, l'installation fonctionne donc depuis n'importe quel dossier.

```bash
packaging/build-all.sh        # bundle + .deb + .rpm + archive précompilée (+ AppImage si appimagetool est là)
```

Tout est produit dans `packaging/out/` (ignoré par git).

| Format | Fichier produit | Commande | Testé ici |
|---|---|---|---|
| Bundle | `out/bundle/` | `build-bundle.sh` | oui |
| `.deb` (Debian, Ubuntu, Mint, Pop!_OS) | `mira-mail_<v>_amd64.deb` | `deb/build.sh` | oui, lancé sur écran virtuel |
| `.rpm` (Fedora, RHEL/Rocky/Alma 9, openSUSE) | `mira-mail-<v>-1.x86_64.rpm` | `rpm/build.sh` | construit, dépendances vérifiées |
| PKGBUILD (Arch, Manjaro, EndeavourOS ; AUR) | `mira-mail-bin` | `arch/PKGBUILD` | `makepkg` local, lancé sur écran virtuel |
| Archive précompilée (source du PKGBUILD) | `mira-mail-<v>-x86_64.tar.gz` | `tarball.sh` | oui |
| AppImage | `MIRA-<v>-x86_64.AppImage` | `appimage/build.sh` | non : demande `appimagetool` |
| Flatpak | dépôt local / Flathub | `flatpak/io.github.nnoah29.Mira.yml` | non : demande `flatpak-builder` et le runtime Freedesktop 24.08 |
| Snap | `mira-mail_<v>_amd64.snap` | `cd packaging && snapcraft` | non : demande `snapcraft` |

## Noms et identifiants

- Paquet et commande : **`mira-mail`**. Le nom `mira` est déjà pris dans Debian (assembleur de génomes qui installe `/usr/bin/mira`).
- Identifiant d'application : **`io.github.nnoah29.Mira`** (lanceur `.desktop`, icônes, fiche AppStream, identifiant GTK). Forme exigée par Flathub pour un projet hébergé sur GitHub.
- Installation : `/usr/lib/mira-mail/` (application), `/usr/bin/mira-mail` (lien relatif), lanceur, icônes 512 px et SVG, fiche AppStream.

## Compatibilité

Le bundle demande **glibc 2.34** au minimum : Ubuntu 22.04+, Debian 12+, Fedora 35+, RHEL/Rocky/Alma 9+, Arch à jour. La bibliothèque du moteur référence aussi `pidfd_spawnp` (glibc 2.39), mais en lien **faible** : la bibliothèque standard Rust ne s'en sert que s'il existe. Le `.rpm` filtre donc cette exigence (`__requires_exclude`), sinon il refuserait de s'installer sur une glibc plus ancienne.

`libdartjni.so` (pont Java d'un plugin Android) est retiré du bundle : jamais chargé sous Linux, il ferait dépendre les paquets de `libjvm`.

Dépendances à l'exécution : GTK 3. Recommandées : `zenity` ou `kdialog` (fenêtres d'ouverture et d'enregistrement de fichiers), `xdg-utils` (liens), un service *Secret Service* comme GNOME Keyring ou KWallet (mots de passe des comptes).

## Publier

1. Mettre la version dans `app/pubspec.yaml`, puis dans `common/io.github.nnoah29.Mira.metainfo.xml` (`<release>`), `arch/PKGBUILD` (`pkgver`), `flatpak/…yml` et `snap/snapcraft.yaml` (nom de l'archive / `version`).
2. `packaging/build-all.sh`.
3. Release GitHub `v<version>` avec le `.deb`, le `.rpm`, l'archive `.tar.gz` (et l'AppImage).
4. **AUR** : remplacer `sha256sums=('SKIP')` par la somme affichée par `tarball.sh`, régénérer `.SRCINFO` (`makepkg --printsrcinfo > .SRCINFO`), pousser `PKGBUILD` et `.SRCINFO` sur `ssh://aur@aur.archlinux.org/mira-mail-bin.git`.

## Points ouverts

- **Licence** : aucune licence n'est encore choisie ; les métadonnées indiquent `LicenseRef-proprietary`. À remplacer (`metainfo.xml`, `PKGBUILD`, `.spec`, `snapcraft.yaml`) dès qu'elle est décidée. Flathub et les dépôts de distributions en exigent une.
- **Flatpak / Snap et fenêtres de fichiers** : le sélecteur de fichiers actuel lance `zenity`/`kdialog`, absents des bacs à sable. Joindre ou enregistrer un fichier n'y fonctionnera pas tant que l'app n'utilise pas le portail du bureau (`xdg-desktop-portal`).
- **Flathub** demande une construction depuis les sources, hors ligne (Rust et Flutter vendorisés), au lieu de l'archive précompilée.
- **Signature** : les paquets ne sont pas signés (dépôt APT/RPM signé à prévoir pour des mises à jour automatiques).
