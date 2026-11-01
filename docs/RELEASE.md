# Publier une release MIRA

Ce guide couvre une release stable MIRA, de l'intégration de `dev` dans
`main` jusqu'aux paquets publiés. Toutes les commandes partent de la racine du
dépôt. Une release doit toujours pouvoir être reconstruite à partir de son tag
`vX.Y.Z`.

> Ne publiez jamais avec des modifications locales. Ne forcez jamais un tag
> déjà public : une erreur se corrige par une nouvelle version.

## 1. Préparer la release sur `dev`

Choisir la version selon Semantic Versioning : majeure pour une incompatibilité,
mineure pour une fonctionnalité compatible, correctif pour un bug. Le numéro
après `+` dans `app/pubspec.yaml` est le numéro de build Android : il doit
augmenter à chaque APK publié.

```bash
export VERSION=2.0.1
export ANDROID_BUILD=19

git fetch --prune --tags origin
git status --short                 # doit être vide
git switch dev
git pull --ff-only origin dev
git log --oneline origin/main..HEAD
```

Vérifier que cette liste contient exactement ce qui doit sortir : pas de WIP,
de fonctionnalité incomplète ou de correctif critique absent.

Mettre à jour la version dans un commit de release sur `dev` :

| Fichier | Valeur à mettre à jour |
| --- | --- |
| `app/pubspec.yaml` | `version: $VERSION+$ANDROID_BUILD` |
| `app/lib/app_info.dart` | `const appVersion = '$VERSION';` |
| `packaging/VERSION` | `$VERSION` |
| `packaging/common/io.github.nnoah29.Mira.metainfo.xml` | Ajouter en tête de `<releases>` : `<release version="$VERSION" date="AAAA-MM-JJ"/>` |
| `packaging/arch/PKGBUILD` | `pkgver=$VERSION` |
| `packaging/snap/snapcraft.yaml` | `version` et le nom d'archive source |
| `packaging/flatpak/io.github.nnoah29.Mira.yml` | le nom d'archive source |

Les versions des crates Rust (`0.1.0`) sont indépendantes de la version
produit. Ne les modifier que si un crate doit aussi être publié.

Préparer les notes de version dans un fichier temporaire, par exemple
`/tmp/mira-release-$VERSION.md` : changements visibles, migrations ou
incompatibilités, limitations connues et installation. Une base de liste est :

```bash
git log --reverse --format='- %s' "v$(git tag --sort=-version:refname | head -1)..HEAD"
```

Valider et pousser le commit de release :

```bash
make check-all
git diff --check
git status --short
git add app/pubspec.yaml app/lib/app_info.dart packaging/VERSION \
  packaging/common/io.github.nnoah29.Mira.metainfo.xml \
  packaging/arch/PKGBUILD packaging/snap/snapcraft.yaml \
  packaging/flatpak/io.github.nnoah29.Mira.yml
git commit -m "chore(release): version $VERSION"
git push origin dev
```

Si Flatpak ou Snap est explicitement différé, ne pas inclure ses fichiers au
commit. Les trois versions utilisateur (`pubspec`, `app_info.dart` et
`packaging/VERSION`) doivent en revanche toujours être égales.

## 2. Fusionner `dev` dans `main`

Refaire les contrôles sur la branche à fusionner :

```bash
git switch dev
git pull --ff-only origin dev
git status --short                 # vide
make check-all
```

Créer une pull request GitHub de `dev` vers `main`, vérifier son diff, les
checks CI et la revue applicable, puis la fusionner par **commit de merge**.
Ne pas supprimer `dev` : elle reste la branche d'intégration.

```bash
gh pr create --base main --head dev \
  --title "Release v$VERSION" \
  --body "Validation locale : make check-all. Version : $VERSION."
# Après revue, fusionner dans GitHub ou avec : gh pr merge --merge
```

Contrôler la vraie branche de production après la fusion :

```bash
git switch main
git pull --ff-only origin main
git status --short
make check-all
git log --oneline -1
```

Si `main` échoue, corriger sur `dev`, refaire la PR et ne pas poser le tag.

## 3. Poser le tag de release

Poser un tag annoté sur la révision validée de `main`, avant de produire les
artefacts :

```bash
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin "v$VERSION"
git show --no-patch "v$VERSION"
```

Tous les fichiers distribués doivent être produits depuis ce tag, sans
modification locale.

## 4. Construire les artefacts

### Avec le script (recommandé)

`packaging/release.sh` fait les étapes 4 à 6 de ce guide, depuis n'importe quelle
branche : il construit dans une copie propre du tag (`target/release-build/vX.Y.Z`,
git worktree), jamais dans votre arbre de travail.

```bash
packaging/release.sh 2.1.0 --check     # vérifie seulement les prérequis
packaging/release.sh 2.1.0             # construit, contrôle, écrit SHA256SUMS
packaging/release.sh 2.1.0 --draft     # idem, puis crée le brouillon GitHub
```

Il refuse de démarrer si le tag n'est pas annoté, sur `origin` et dans `origin/main`,
si les versions (`pubspec`, `app_info.dart`, `VERSION`, `metainfo`, `PKGBUILD`) ne
correspondent pas, si le numéro de build Android n'augmente pas, ou s'il manque un
outil (`appimagetool`, `cargo-ndk`, cibles Rust Android, NDK, identifiants OAuth de
production). `--no-appimage`, `--no-android` et `--no-oauth` servent aux essais ;
`--draft` les refuse. Après la construction, il contrôle chaque fichier (version,
moteur présent, version du bundle), produit `SHA256SUMS` avec `install.sh` et
écrit `packaging/out/release-X.Y.Z.log`. Il ne publie jamais : `--draft` crée un
brouillon. Les étapes 5 (essais sur machine propre) et 7 (AUR) restent manuelles.

### À la main

Vérifier d'abord que la machine de release possède les outils nécessaires :

```bash
git status --short                 # vide
git describe --exact-match --tags  # v$VERSION
command -v cargo flutter dpkg-deb rpmbuild appimagetool sha256sum gh
```

Les scripts de build peuvent intégrer la section `[oauth]` du fichier désigné
par `MIRA_OAUTH_CONFIG`. Vérifier que c'est la configuration de production et
ne jamais la copier ni la commiter. Une valeur vide construit délibérément des
binaires sans identifiants OAuth.

`packaging/out/` est ignoré par Git et ne doit contenir aucun reste d'une
ancienne release :

```bash
rm -rf packaging/out
mkdir -p packaging/out
```

Construire explicitement chaque format annoncé. `build-all.sh` peut ignorer
silencieusement RPM ou AppImage quand l'outil manque ; il ne suffit donc pas
pour une release complète.

```bash
packaging/build-bundle.sh
packaging/deb/build.sh
packaging/rpm/build.sh
packaging/tarball.sh
packaging/appimage/build.sh
packaging/android/build.sh
```

Donner aux APK des noms publics qui indiquent leur architecture :

```bash
mv packaging/out/app-arm64-v8a-release.apk \
  "packaging/out/mira-mail-$VERSION-android-arm64-v8a.apk"
mv packaging/out/app-x86_64-release.apk \
  "packaging/out/mira-mail-$VERSION-android-x86_64.apk"
```

Créer le manifeste à partir des seuls assets à téléverser :

```bash
cd packaging/out
sha256sum \
  "mira-mail_${VERSION}_amd64.deb" \
  "mira-mail-${VERSION}-1.x86_64.rpm" \
  "mira-mail-${VERSION}-x86_64.tar.gz" \
  "MIRA-${VERSION}-x86_64.AppImage" \
  "mira-mail-${VERSION}-android-arm64-v8a.apk" \
  "mira-mail-${VERSION}-android-x86_64.apk" > SHA256SUMS
cd ../..
cp packaging/install.sh packaging/out/install.sh
sha256sum packaging/out/install.sh >> packaging/out/SHA256SUMS
sha256sum -c packaging/out/SHA256SUMS
```

`install.sh` et `SHA256SUMS` sont obligatoires : le README propose de
télécharger ce script depuis la dernière release et le script télécharge puis
vérifie l'archive `.tar.gz` avec ce manifeste.

## 5. Vérifier les paquets

Un build vert ne valide pas une installation. Exécuter au minimum :

```bash
dpkg-deb --info "packaging/out/mira-mail_${VERSION}_amd64.deb"
dpkg-deb --contents "packaging/out/mira-mail_${VERSION}_amd64.deb"
rpm -qpi "packaging/out/mira-mail-${VERSION}-1.x86_64.rpm"
rpm -qlp "packaging/out/mira-mail-${VERSION}-1.x86_64.rpm"
tar -tzf "packaging/out/mira-mail-${VERSION}-x86_64.tar.gz" | head
file "packaging/out/MIRA-${VERSION}-x86_64.AppImage"
unzip -l "packaging/out/mira-mail-${VERSION}-android-arm64-v8a.apk" | head
sha256sum -c packaging/out/SHA256SUMS
```

Puis installer et lancer les formats concernés sur une VM ou une machine
propre : démarrage, profil vierge, version affichée, rendu HTML/WebKit,
compte de test et fermeture. Installer les APK sur l'ABI correspondante (ou
un émulateur) et vérifier leur démarrage et le moteur Rust.

### Blocage Android / Play Store

Le script actuel construit des APK signés avec la clé de debug. Ils peuvent
servir aux essais directs, mais ne doivent pas être envoyés au Play Store ni
présentés comme une chaîne de mise à jour de production. Avant cela, configurer
une clé de publication conservée hors dépôt ; un APK signé par une autre clé
ne peut pas mettre à jour une installation existante.

## 6. Créer puis publier la release GitHub

Créer d'abord un brouillon, lié au tag existant :

```bash
ls -lh packaging/out
sha256sum -c packaging/out/SHA256SUMS

gh release create "v$VERSION" \
  "packaging/out/mira-mail_${VERSION}_amd64.deb" \
  "packaging/out/mira-mail-${VERSION}-1.x86_64.rpm" \
  "packaging/out/mira-mail-${VERSION}-x86_64.tar.gz" \
  "packaging/out/MIRA-${VERSION}-x86_64.AppImage" \
  "packaging/out/mira-mail-${VERSION}-android-arm64-v8a.apk" \
  "packaging/out/mira-mail-${VERSION}-android-x86_64.apk" \
  packaging/out/SHA256SUMS packaging/out/install.sh \
  --title "MIRA $VERSION" \
  --notes-file "/tmp/mira-release-$VERSION.md" \
  --verify-tag --draft
```

Dans le brouillon GitHub, confirmer le tag, les huit assets, les notes et
l'absence du statut *pre-release*. Télécharger au moins l'archive publiée,
vérifier sa somme et tester `install.sh` dans un compte ou une VM jetable.
Publier alors le brouillon dans GitHub et le marquer *Latest release*.

```bash
gh release view "v$VERSION" --json tagName,isDraft,isPrerelease,isLatest,assets,url
```

## 7. Canaux secondaires et incident

Après la release GitHub, mettre à jour le dépôt AUR `mira-mail-bin` :
`pkgver=$VERSION`, somme de l'archive publiée à la place de `SKIP`, puis :

```bash
makepkg --printsrcinfo > .SRCINFO
makepkg -f
git add PKGBUILD .SRCINFO
git commit -m "mira-mail-bin $VERSION"
git push
```

Flatpak et Snap ne font pas partie de la release standard tant qu'ils ne sont
pas construits et testés séparément : Flatpak demande un build source compatible
Flathub et le portail de fichiers, Snap son environnement `snapcraft`.

Après publication, vérifier `releases/latest`, le lien d'installation du
README et une installation sans sudo. Tout correctif démarre sur `dev`.
Si une release publique est mauvaise, ne modifiez pas le tag : documentez le
problème, retirez son statut *Latest* si nécessaire et publiez une version
corrective.

## Checklist

- [ ] `dev` puis `main` sont propres et `make check-all` est vert.
- [ ] Toutes les versions et le build Android sont cohérents.
- [ ] La PR `dev` → `main` est revue et fusionnée.
- [ ] Le tag annoté `vX.Y.Z` pointe sur `main` validée.
- [ ] Tous les assets, `SHA256SUMS` et `install.sh` sont construits depuis ce tag.
- [ ] Les paquets et APK ont été installés/testés sur des environnements propres.
- [ ] La release GitHub est publiée et *Latest*.
- [ ] AUR et les canaux optionnels ne sont mis à jour qu'après validation.
