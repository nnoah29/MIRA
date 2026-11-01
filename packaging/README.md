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
| AppImage | `MIRA-<v>-x86_64.AppImage` | `appimage/build.sh` | oui, lancée sur écran virtuel |
| Flatpak | dépôt local / Flathub | `flatpak/io.github.nnoah29.Mira.yml` | non : demande `flatpak-builder` et le runtime Freedesktop 24.08 |
| Snap | `mira-mail_<v>_amd64.snap` | `cd packaging && snapcraft` | non : demande `snapcraft` |

## Installer ou mettre à jour sur sa machine

```bash
packaging/install.sh              # installe la dernière release, ou la met à jour si elle est plus récente
packaging/install.sh 1.0.0        # une version précise
packaging/install.sh --force      # réinstalle même si la version est déjà là
packaging/install.sh --uninstall  # retire l'application (comptes et mails conservés)
```

Installation pour l'utilisateur courant, sans `sudo`, quelle que soit la distribution : l'archive `.tar.gz` de la release est vérifiée avec `SHA256SUMS`, puis extraite dans `~/.local/share/mira-mail/<version>/` ; `current` pointe vers la version active, la précédente est gardée pour revenir en arrière. Commande `~/.local/bin/mira-mail`, lanceur et icônes dans `~/.local/share`. Le dépôt étant privé, le script passe par `gh` s'il est connecté, sinon par `curl` (dépôt public seulement).

## Identifiants OAuth intégrés

`build-bundle.sh` et `android/build.sh` intègrent au moteur la section `[oauth]` de `~/.config/mira/config.toml` (ou du fichier désigné par `MIRA_OAUTH_CONFIG` ; `MIRA_OAUTH_CONFIG=` vide pour s'en passer). Le reste du fichier n'est pas repris. Le `build.rs` de `mira-assembly` la chiffre en AES-256-GCM avec une clé tirée à chaque compilation : ni l'identifiant ni le secret n'apparaissent en clair dans le binaire. La clé voyageant avec le binaire, c'est un masquage contre la lecture directe, pas une protection : Google ne tient pas le secret d'une app de bureau pour confidentiel, la sécurité du flux repose sur PKCE et la redirection vers `127.0.0.1`. Un `config.toml` présent chez l'utilisateur l'emporte, provider par provider.

## Démo en ligne (PWA)

```
packaging/web/build-demo.sh    # app compilée pour le web, prête dans packaging/out/web-demo/
```

À copier à la racine de la branche `web` (GitHub Pages, <https://nnoah29.github.io/MIRA/>) en gardant `404.html` et `.nojekyll`. L'app y tourne en démonstration (données fictives) : le moteur Rust n'existe pas dans un navigateur. Le script élague les fichiers `skwasm` de CanvasKit, inutiles pour un build JavaScript, et tamponne le cache du service worker avec le commit et l'heure : chaque déploiement repart d'un cache neuf.

## Android

```bash
packaging/android/build.sh    # moteur Rust (arm64-v8a, x86_64) + APK release, copiés dans packaging/out/
```

Prérequis : SDK et NDK Android, `rustup target add aarch64-linux-android x86_64-linux-android`, `cargo install cargo-ndk`. Les bibliothèques du moteur sont écrites dans `app/android/app/src/main/jniLibs/` (ignoré par git).

Sur Android, l'app passe au moteur son dossier privé (`getApplicationSupportDirectory`) et une clé de 32 octets. Le moteur range les mots de passe des comptes dans `secrets.bin`, chiffré en AES-256-GCM avec cette clé. La clé elle-même est tirée une fois au hasard et rangée chiffrée par une clé du Keystore Android (`MainActivity.kt`), qui ne quitte pas le matériel sécurisé. Sous Linux, rien ne change : trousseau du système et dossiers XDG.

L'APK release est encore signé avec la clé de debug : il faut une clé de publication avant le Play Store. L'identifiant Android reste `n.noah.mira`.

## Noms et identifiants

- Paquet et commande : **`mira-mail`**. Le nom `mira` est déjà pris dans Debian (assembleur de génomes qui installe `/usr/bin/mira`).
- Identifiant d'application : **`io.github.nnoah29.Mira`** (lanceur `.desktop`, icônes, fiche AppStream, identifiant GTK). Forme exigée par Flathub pour un projet hébergé sur GitHub.
- Installation : `/usr/lib/mira-mail/` (application), `/usr/bin/mira-mail` (lien relatif), lanceur, icônes 512 px et SVG, fiche AppStream.

## Licence

MIT (`LICENSE` à la racine), installée avec chaque paquet : `/usr/share/licenses/mira-mail/LICENSE`, et `/usr/share/doc/mira-mail/copyright` pour le `.deb`.

## AppImage

`appimagetool` n'est pas dans le dépôt : le placer dans le `PATH`, ou le télécharger depuis https://github.com/AppImage/appimagetool/releases et passer `APPIMAGETOOL=/chemin/appimagetool` à `build-all.sh`.

## Compatibilité

Le bundle demande **glibc 2.34** au minimum : Ubuntu 22.04+, Debian 12+, Fedora 35+, RHEL/Rocky/Alma 9+, Arch à jour. La bibliothèque du moteur référence aussi `pidfd_spawnp` (glibc 2.39), mais en lien **faible** : la bibliothèque standard Rust ne s'en sert que s'il existe. Le `.rpm` filtre donc cette exigence (`__requires_exclude`), sinon il refuserait de s'installer sur une glibc plus ancienne.

`libdartjni.so` (pont Java d'un plugin Android) est retiré du bundle : jamais chargé sous Linux, il ferait dépendre les paquets de `libjvm`.

Dépendances à l'exécution : GTK 3 et WebKitGTK 4.1 (`libwebkit2gtk-4.1`, corps HTML des messages). Recommandées : `zenity` ou `kdialog` (fenêtres d'ouverture et d'enregistrement de fichiers), `xdg-utils` (liens), un service *Secret Service* comme GNOME Keyring ou KWallet (mots de passe des comptes).

## Publier

1. Mettre la version dans `app/pubspec.yaml`, puis dans `common/io.github.nnoah29.Mira.metainfo.xml` (`<release>`), `arch/PKGBUILD` (`pkgver`), `flatpak/…yml` et `snap/snapcraft.yaml` (nom de l'archive / `version`).
2. `packaging/build-all.sh`.
3. Release GitHub `v<version>` avec le `.deb`, le `.rpm`, l'archive `.tar.gz` (et l'AppImage).
4. **AUR** : remplacer `sha256sums=('SKIP')` par la somme affichée par `tarball.sh`, régénérer `.SRCINFO` (`makepkg --printsrcinfo > .SRCINFO`), pousser `PKGBUILD` et `.SRCINFO` sur `ssh://aur@aur.archlinux.org/mira-mail-bin.git`.

## Points ouverts

- **Flatpak / Snap et fenêtres de fichiers** : le sélecteur de fichiers actuel lance `zenity`/`kdialog`, absents des bacs à sable. Joindre ou enregistrer un fichier n'y fonctionnera pas tant que l'app n'utilise pas le portail du bureau (`xdg-desktop-portal`).
- **Flathub** demande une construction depuis les sources, hors ligne (Rust et Flutter vendorisés), au lieu de l'archive précompilée.
- **Signature** : les paquets ne sont pas signés (dépôt APT/RPM signé à prévoir pour des mises à jour automatiques).
