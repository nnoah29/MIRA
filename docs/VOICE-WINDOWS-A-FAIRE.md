# Dictée sous Windows : ce qu'il reste à faire (guide pas à pas)

Rédigé le 2026-11-18 sur la machine Linux de développement, pour être suivi **seul** sur une machine Windows. Rien de ce
qui est décrit ici comme « à vérifier » n'a été essayé sous Windows : ce document a été écrit sans y avoir accès.
Contexte et décision de conception : `docs/VOICE-WINDOWS.md`. Fonctionnement général de la dictée : `docs/VOICE.md`.

Légende : `- [ ]` à faire, **À VÉRIFIER** = hypothèse non testée, **RISQUE** = point où je m'attends à un problème.

## 0. En une page

**Objectif.** La dictée de MIRA se comporte sous Windows comme sous Linux. Le moteur ne change pas : sherpa-onnx et les
modèles Kroko (hors ligne, français et anglais). Les API vocales natives de Windows ont été étudiées et écartées
(identité de paquet MSIX exigée, dictée libre dans le nuage de Microsoft, ou interface dépréciée ou en anglais seulement).

**Pourquoi il y a du travail quand même.** Le code de la dictée est en Dart commun et marche sous Linux, mais il n'a
**jamais été exécuté sous Windows**. Les écarts probables sont dans la plateforme : interception de la touche Espace,
permission du micro, téléchargement et extraction du modèle, DLL natives, accents dans le nom d'utilisateur, antivirus.

**Ce que tu dois faire, dans l'ordre :**

1. Préparer la machine (§ 3) puis récupérer la branche et construire (§ 4).
2. Dérouler les essais manuels (§ 5) en notant ce qui marche et ce qui casse.
3. Corriger ce qui casse (§ 6 donne les pistes), et adapter les tests (§ 7).
4. Relever les mesures (§ 8), mettre à jour la documentation (§ 9), ouvrir la PR (§ 10).

## 1. Où en est le code

Branche à utiliser : **`feat/voix-appui-espace`** (elle contient le code et cette documentation). Elle part de `dev` au
commit `947f4af` (version 2.4.0). Commits de la branche, du plus ancien au plus récent :

| Commit | Contenu |
|---|---|
| `13aaa9e` | arrêt « jusqu'au bout » : relâcher avant la fin du chargement du modèle ne perd plus les mots |
| `f02decb` | modèle gardé chaud (3 min par défaut), mode de dictée (continue ou phrase par phrase), réglages mémorisés |
| `42dd35c` | Espace maintenu : appui court = espace, appui long (400 ms) = dictée |
| `1e28bef` | appui long sur le bouton micro (pour le tactile) |
| `9a85963` | réglages dans l'onglet Voix (Espace maintenu, délai de maintien, durée de garde) |
| `ac57818` | mise à jour de `docs/VOICE.md` |
| `bd150fd` | fusion du document de conception Windows |

Avant cette branche, la version 2.4.0 contient déjà : micro ouvert dès le clic avec réserve audio, barres de niveau
sonore, démarrage automatique après téléchargement du modèle, chargement du modèle deux fois plus rapide.

**Validé sous Linux** : 1 291 tests Flutter passent (sans démon ni modèles), la suite du moteur est verte, essai manuel
avec un vrai micro. **Non validé** : tout ce qui touche Windows, Android et macOS. L'essai d'Espace maintenu avec le vrai
clavier sous Linux était en cours quand ce document a été écrit : vérifie dans `docs/VOICE.md` s'il a été noté comme validé.

## 2. Parité visée avec Linux

Chaque ligne est un comportement à retrouver tel quel sous Windows (la case se coche une fois essayé, voir § 5).

- [ ] Cliquer sur le micro lance la **dictée continue**, le bouton affiche les barres animées, recliquer l'arrête.
- [ ] **Le micro écoute dès le clic**, les premiers mots dits pendant le chargement du modèle ne sont pas perdus.
- [ ] **Espace maintenu** (≥ 400 ms) dicte phrase par phrase ; un appui court reste un espace ; aucun espace parasite.
- [ ] L'**appui long sur le bouton micro** fait la même chose (utile au tactile).
- [ ] Arrêter va **jusqu'au bout** : le texte est validé même si on relâche avant la fin du chargement.
- [ ] Le **modèle reste chargé** après la dictée (3 min par défaut) : la suivante démarre sans attente.
- [ ] Les **réglages** de l'onglet Voix s'appliquent (langue, Espace maintenu, délai, garde du modèle).
- [ ] **Téléchargement du modèle** vérifié (SHA-256), extraction, suppression, retéléchargement proposé si absent.
- [ ] Français **et** anglais.
- [ ] Les messages d'erreur sont clairs : micro refusé, micro absent, modèle absent, échec de la reconnaissance.

## 3. Préparer la machine Windows

Machine : Windows 10 ou 11 en x64 (l'ARM64 est un plus, voir § 6). Comptes : un accès à GitHub (`nnoah29/MIRA`).

- [ ] **Git pour Windows**.
- [ ] **Mode développeur activé** (Paramètres › Système › Pour les développeurs). Flutter en a besoin pour créer les liens
      symboliques des plugins ; sans lui, `flutter pub get` échoue avec une erreur de lien symbolique.
- [ ] **Visual Studio** (2022 ou plus récent), charge de travail **« Développement Desktop en C++ »** (MSVC, kit de
      développement Windows, CMake). `packaging/windows/build.ps1` mentionne Visual Studio 2026 et un drapeau de
      compilation pour le plugin `webview_all_windows`.
- [ ] **Rust** via `rustup`, cible `x86_64-pc-windows-msvc`. Le fichier `rust-toolchain.toml` impose la version **1.99.0**
      (installée automatiquement par `rustup` dans le dépôt).
- [ ] **Flutter 3.47.7**, canal stable (version épinglée par la CI, voir `.github/workflows/windows.yml`). Puis
      `flutter config --enable-windows-desktop` et `flutter doctor` : tout doit être au vert côté Windows.
- [ ] **PowerShell 7** (`pwsh`) pour les scripts du dépôt.
- [ ] **Inno Setup 6**, seulement pour tester l'installeur (`choco install innosetup`).
- [ ] Un **micro** qui fonctionne (casque, micro intégré), et idéalement un **second micro** (casque Bluetooth ou USB)
      pour les essais de changement de périphérique.

Rappel utile pour plus tard : le dossier du cache de paquets Dart est `%LOCALAPPDATA%\Pub\Cache`. Les DLL de la
reconnaissance y sont (voir § 7, test d'intégration).

## 4. Récupérer le code et construire

```powershell
git clone https://github.com/nnoah29/MIRA.git
cd MIRA
git fetch origin
git switch feat/voix-appui-espace
packaging\windows\build.ps1
```

`build.ps1` construit le pont (`mira_bridge.dll`), le démon (`mira-daemon.exe`) puis l'application, et écrit :
`app\build\windows\x64\runner\Release\mira.exe`. Il avertit s'il n'y a pas d'identifiants OAuth intégrés
(`%APPDATA%\mira\config.toml`) : **sans importance pour la dictée**, seule la connexion Google ou Microsoft est touchée.
Si `build.ps1` pose problème, les étapes à la main sont dans `.github/workflows/windows.yml` (pont, démon, `flutter build windows --release`).

- [ ] La construction réussit. **RISQUE** : erreur de compilation du plugin `webview_all_windows` (drapeau
      `_SILENCE_EXPERIMENTAL_COROUTINE_DEPRECATION_WARNINGS`, déjà géré par `build.ps1`).
- [ ] Vérifier que les DLL de la dictée sont **à côté de `mira.exe`** dans `Release\` : `onnxruntime.dll`,
      `onnxruntime_providers_shared.dll`, `sherpa-onnx-c-api.dll`, `sherpa-onnx-cxx-api.dll`, ainsi que les DLL des plugins Flutter (`*_plugin.dll`, dont celle du micro).
      Une DLL absente se traduirait par une dictée qui échoue en silence (message « La dictée a rencontré une erreur »).
- [ ] Lancer `mira.exe`, ouvrir Réglages › Voix : la section existe.

Pour tester l'installeur ensuite : `packaging\windows\build-installer.ps1 -Version 0.0.0 -OutDir out` puis
`packaging\windows\test-installer.ps1 -Setup (Resolve-Path out\mira-mail-0.0.0-windows-x64-setup.exe)`. L'installeur
copie tout le dossier `Release` (`Source: "{#Stage}\*"`, `recursesubdirs`), donc les DLL partent avec.

## 5. Essais manuels (ce qu'il faut observer)

Fais chaque essai, coche, et **note le résultat réel** à côté (même quand ça marche). Pour chaque échec, garde le message
exact et l'étape. Le fichier de test idéal est un brouillon vide et le champ du chat IA.

### 5.1 Modèle

- [ ] **W1 — Téléchargement.** Réglages › Voix › Télécharger (français). La barre avance, le SHA-256 est vérifié, le modèle
      passe à « Installé ». Le dossier créé est `%APPDATA%\...\voice-models\` (le chemin exact vient de
      `getApplicationSupportDirectory` ; relève-le). **RISQUE** : l'antivirus (Windows Defender) verrouille un fichier au moment
      du `rename` du dossier temporaire (`voice_model_store.dart`, `_tmp(e).rename(target.path)`) : erreur « accès refusé ».
- [ ] **W2 — Suppression puis retéléchargement.** Supprimer le modèle, retenter le micro : la boîte de dialogue de
      téléchargement apparaît, et la dictée démarre toute seule après l'installation.
- [ ] **W3 — Anglais.** Même chose pour le modèle anglais, puis changer de langue de dictée.

### 5.2 Dictée continue (bouton micro)

- [ ] **W4 — Première dictée.** Cliquer le micro et parler **tout de suite** (« Bonjour Marie, je vous confirme notre
      rendez-vous de jeudi à quatorze heures. »). Les barres s'animent immédiatement. Le texte apparaît après le chargement
      du modèle (mesure : § 8), avec les premiers mots. Recliquer arrête et valide.
- [ ] **W5 — Deuxième dictée juste après** : démarrage sans attente (modèle chaud).
- [ ] **W6 — Un accent ou un caractère spécial** dans la phrase, et un mot anglais : relever ce qui est reconnu.
- [ ] **W7 — Dictée longue** de 5 minutes ou plus : elle s'arrête d'elle-même à 5 minutes (`maxDuration`).

### 5.3 Espace maintenu (le point le plus incertain)

- [ ] **W8 — Frappe normale.** Taper une phrase avec des espaces : chaque espace apparaît **une fois**, sans retard visible.
- [ ] **W9 — Espace maintenu.** Maintenir Espace (≈ 0,5 s), parler, relâcher : le texte arrive à la bonne place, et
      **aucun espace en trop** n'apparaît ni pendant ni après. **RISQUE principal** (voir R1, § 6).
- [ ] **W10 — Appui court** : un seul espace.
- [ ] **W11 — Frappe rapide** : taper « a b c d » très vite : l'ordre des caractères est correct (l'espace retenu passe
      avant la lettre suivante).
- [ ] **W12 — Au milieu d'un texte**, avec une sélection : un appui court remplace la sélection par un espace ; un appui
      long ne casse pas le texte existant.
- [ ] **W13 — Répétition automatique** : maintenir Espace plus de 2 s ; pas d'espaces multiples.
- [ ] **W14 — Ctrl+Espace, Alt+Espace, Maj+Espace** : ne déclenchent pas la dictée. **AltGr** (clavier français) : un appui
      sur Espace avec AltGr reste normal.
- [ ] **W15 — Composition de texte** (IME, par exemple clavier japonais ou chinois si disponible) : Espace n'est pas intercepté.
- [ ] **W16 — Pendant une dictée continue**, Espace reste un espace normal.
- [ ] **W17 — Sans modèle installé** : maintenir Espace ne fait rien et n'ouvre aucune boîte de dialogue.
- [ ] **W18 — Réglage coupé** (Réglages › Voix › Espace maintenu › Désactivé) : Espace redevient normal.
- [ ] **W19 — Délai** réglé à 300 puis 600 ms : le seuil change vraiment.
- [ ] **W20 — Brouillon** : le texte dicté marque le brouillon comme modifié et il est enregistré.
- [ ] **W21 — Chat IA** : maintenir Espace dans le champ de l'assistant dicte, Entrée envoie.

### 5.4 Appui long (tactile ou souris)

- [ ] **W22 — Appui long sur le bouton micro** à la souris (maintenir ≥ 0,5 s) : dictée phrase par phrase, relâcher valide.
      Sur un **écran tactile** si la machine en a un : même chose au doigt, et l'info-bulle ne s'affiche pas à la place.

### 5.5 Micro

- [ ] **W23 — Micro refusé.** Paramètres › Confidentialité et sécurité › Microphone : couper « Autoriser les applications
      de bureau à accéder à votre microphone », relancer, essayer la dictée. **Note exactement ce qui se passe** (erreur ?
      silence ?). **RISQUE R2** : sous Windows, le plugin répond toujours « autorisé » (`hasPermission` renvoie `true` dans
      `record_windows_plugin.cpp`), donc MIRA ne peut pas afficher « Accès au micro refusé » par ce biais.
- [ ] **W24 — Aucun micro** (débrancher, ou le désactiver dans Paramètres › Son) : message clair, pas de plantage.
- [ ] **W25 — Changement de micro** pendant la dictée (débrancher le casque) : arrêt propre, le texte déjà dicté est conservé.
- [ ] **W26 — Casque Bluetooth** : la qualité du son tombe en mode mains libres ; relever si la reconnaissance se dégrade.
- [ ] **W27 — Micro par défaut** changé dans Windows : la dictée suivante utilise le nouveau.

### 5.6 Robustesse

- [ ] **W28 — Fermer le brouillon** pendant la dictée : arrêt sans écriture parasite ni plantage.
- [ ] **W29 — Quitter MIRA** pendant la dictée : aucun processus `mira.exe` ne reste (Gestionnaire des tâches), le voyant
      du micro de Windows s'éteint.
- [ ] **W30 — Mise en veille** puis réveil de la machine pendant le mode « modèle chaud » : la dictée suivante marche.

## 6. Risques connus, avec la piste de correction

**R1 — Espace maintenu insère quand même un espace (priorité haute).**
`lib/ui/push_to_talk_space.dart` retourne `KeyEventResult.handled` sur l'appui d'Espace pour que le champ de texte
n'insère rien. Les tests automatiques ne simulent pas le système de saisie de la plateforme : sous Linux l'hypothèse
« handled = rien n'est inséré » était à confirmer à la main. **Sous Windows**, Flutter reçoit aussi un message de caractère
(`WM_CHAR`) pour Espace ; **À VÉRIFIER** qu'il est supprimé quand le cadre applicatif a traité la touche.
*Si un espace parasite apparaît pendant ou après la dictée*, piste de repli : au démarrage de la dictée (`_startDictation`),
comparer le texte du champ à celui enregistré à l'appui (`_before`) et **retirer l'espace inséré à la position du curseur**
avant de lancer la dictée ; faire de même au relâchement. Si l'espace parasite apparaît à l'appui court (en double), c'est
l'insertion manuelle de `_insertSpace` qui s'ajoute à celle de la plateforme : dans ce cas ne rien insérer soi-même et
retourner `ignored` sur l'appui court.
Dernier recours : une touche dédiée à la place d'Espace (par exemple Ctrl+Espace).

**R2 — Micro refusé non détectable.** `record_windows` répond toujours `true` à `hasPermission`. Comportement réel à relever
au W23. Piste : dans `DictationController._onSamples`, si tous les paquets d'une fenêtre de ~2 s ne contiennent que des zéros
exacts (silence numérique, ce qui n'arrive qu'avec un micro bloqué ou coupé), arrêter la dictée avec
`DictationProblem.microphoneUnavailable` et un message qui oriente vers Paramètres › Confidentialité › Microphone. Il y a
déjà `audioLevel` (`voice/audio_level.dart`) ; le niveau est 0 pour du silence numérique.

**R3 — Les tests d'intégration créent des liens symboliques.** `voice_sherpa_integration_test.dart` et
`voice_sherpa_warm_test.dart` utilisent `Link('...').createSync(...)` (fonction `storeWithModels`). Sous Windows cela exige
des droits (mode développeur ou administrateur) et peut échouer. Piste simple : `MIRA_VOICE_MODELS` a **la même
disposition** que la racine du magasin de modèles (un sous-dossier par modèle) ; construire directement
`VoiceModelStore(root: Directory(modelsDir))` au lieu de créer des liens.

**R4 — DLL de la reconnaissance introuvables sous `flutter test`.** Sous Linux il faut `LD_LIBRARY_PATH` vers le dossier du
paquet `sherpa_onnx_linux`. Sous Windows la recherche passe par `PATH` : ajouter
`%LOCALAPPDATA%\Pub\Cache\hosted\pub.dev\sherpa_onnx_windows-2.0.1\windows\x64` au `PATH` avant `flutter test`.

**R5 — Chemins avec accents ou espaces.** Si le nom d'utilisateur Windows contient un accent (`C:\Users\Noé\...`), le chemin
du modèle est passé à la bibliothèque native de sherpa-onnx. **À VÉRIFIER** qu'elle l'accepte (encodage UTF-8 contre page de
code Windows). À tester avec un compte de test dont le nom contient un accent. Piste : copier le modèle dans un chemin ASCII,
ou convertir le chemin en nom court (8.3).

**R6 — Antivirus.** Windows Defender peut analyser les 70 Mo de `encoder.onnx` à chaque lecture : un premier chargement très
lent est à relever (§ 8). Le `rename` du dossier temporaire peut échouer sur un fichier verrouillé (W1) : piste, réessayer
quelques fois avec une courte attente, au lieu d'échouer.

**R7 — Mémoire et chargement sur une machine modeste.** Sous Linux : environ 1,7 s de chargement et 130 Mo. Sous un Windows
avec 4 Go de mémoire, le chargement peut être beaucoup plus long. Relever les mesures (§ 8) ; si le chargement dépasse 4 s,
c'est ce qui justifie l'étude d'un modèle plus léger (quantifié, environ quatre fois plus petit) ou d'un préchauffage.

**R8 — Windows ARM64.** Le plugin fournit `windows\arm64` (mêmes quatre DLL), mais ni la compilation ni la dictée n'y sont
testées. Un appareil ARM64 serait un bonus ; sinon à documenter comme non testé.

**R9 — Mise en veille de la machine** : un isolate qui garde le modèle pendant la veille. À observer (W30).

**R10 — Extraction du modèle en mémoire.** `voice_model_archive.dart` lit toute l'archive de 57 Mo en mémoire puis la
décompresse dans un isolate : pic de mémoire d'environ 200 Mo pendant l'installation. Acceptable, mais à relever sur une
machine modeste.

## 7. Tests automatiques

Les tests de la dictée sont dans `app/test/` : `voice_*_test.dart`, `push_to_talk_space_test.dart`, `model_keeper_test.dart`.
Lancer ceux qui ne demandent ni micro ni modèle :

```powershell
cd app
flutter test test\voice_controller_test.dart test\voice_graceful_stop_test.dart test\voice_immediate_start_test.dart `
  test\voice_level_test.dart test\voice_mode_test.dart test\voice_push_preferences_test.dart `
  test\voice_push_settings_test.dart test\push_to_talk_space_test.dart test\voice_mic_long_press_test.dart `
  test\voice_mic_button_test.dart test\voice_wiring_test.dart test\model_keeper_test.dart
```

- [ ] **Les tests existants passent sous Windows** (ils n'utilisent ni micro ni modèle). Tout échec ici est un vrai défaut
      de portabilité (chemins, fins de ligne, `Platform`). **Ne pas modifier ni ignorer un test pour le faire passer** : corriger le code.
- [ ] **Tests d'intégration de la reconnaissance** (ignorés tant que `MIRA_VOICE_MODELS` n'est pas défini) : télécharger
      l'archive du modèle français (`voice_model_catalog.dart` donne l'URL et le SHA-256), la décompresser **en entier**
      (elle contient `test_wavs\0.wav` dont le test a besoin), puis :
      ```powershell
      $env:MIRA_VOICE_MODELS = "C:\chemin\vers\dossier-contenant-les-modeles"
      $env:PATH += ";$env:LOCALAPPDATA\Pub\Cache\hosted\pub.dev\sherpa_onnx_windows-2.0.1\windows\x64"
      flutter test test\voice_sherpa_integration_test.dart test\voice_sherpa_warm_test.dart
      ```
      Corriger R3 d'abord si les tests plantent sur `Link.createSync`.
- [ ] **À ajouter dans `.github/workflows/windows.yml`** (étape après « Application Windows ») : un job qui télécharge l'archive
      épinglée (comme l'étape « Serveur local » le fait pour `llama-server`), vérifie le SHA-256, la décompresse, règle
      `MIRA_VOICE_MODELS` et `PATH`, puis lance les deux tests d'intégration ci-dessus. Le workflow ne se déclenche que sur les
      PR qui touchent `app/**`, `engine/**`, `bridge/**` ou `packaging/windows/**`.
- [ ] **Si un comportement propre à Windows est corrigé, ajouter son test d'abord** (échec visible, puis correction).
- [ ] Si un test ne peut pas passer à cause de la plateforme, utiliser `skip:` avec la raison exacte, **jamais** supprimer le
      test, et le noter dans ce document (§ 9) : le fichier `docs/ETAT/BLOCAGES.md` cité dans les règles du projet n'existe pas dans le dépôt.

## 8. Mesures à relever

À remplir sur la machine de test, et à reporter dans `docs/VOICE.md` (section « Statut par plateforme »). Pour la mémoire :
Gestionnaire des tâches (colonne « Mémoire (jeu de travail) »), ou `Get-Process mira | Select-Object WorkingSet64`.

| Mesure | Linux (référence) | Windows : résultat |
|---|---|---|
| Machine (processeur, mémoire, disque) | i5-1135G7, 8 cœurs | |
| Chargement du modèle, première fois (à froid) | 1,7 à 2,0 s | |
| Démarrage de la dictée suivante (modèle chaud) | ≈ 1 ms | |
| Premier texte visible après le clic (W4) | ≈ 2 s | |
| Mémoire de MIRA au repos | ≈ 250 à 280 Mo | |
| Mémoire pendant une dictée (modèle chargé) | + ≈ 130 Mo | |
| Mémoire 4 min après la dictée (modèle libéré) | retour au niveau d'avant | |
| CPU de MIRA au repos | ≈ 0,1 % | |
| CPU pendant la dictée | | |
| Taille de l'installeur (avec les DLL) | | |
| Durée du téléchargement du modèle, extraction comprise | | |

## 9. Critères pour passer Windows de « non testé » à « validé »

| Critère | Seuil |
|---|---|
| La dictée démarre et s'arrête sans plantage | 20 essais de suite, 0 échec |
| Premier texte visible après le clic | moins de 4 s sur la machine de test, relevé aussi sur une machine modeste |
| Phrase de test reconnue en entier, y compris « Bonjour Marie » | oui |
| Espace maintenu : aucun espace parasite, aucun caractère perdu | W8 à W15 tous cochés |
| Micro refusé ou absent | message clair, aucun plantage (W23, W24) |
| Micro débranché en cours de route | arrêt propre, texte conservé (W25) |
| Modèle supprimé | le micro propose de le retélécharger (W2) |
| Installeur | dictée fonctionnelle après une installation propre, sans DLL manquante |
| Tests de la voix | tous verts sous Windows, intégration incluse |

Quand tout est coché : mettre à jour la ligne Windows du tableau « Statut par plateforme » de `docs/VOICE.md` (de
« non testé » à « validé », avec la date, la machine et la version de Windows), cocher la section 2 de ce document, et noter
les mesures du § 8.

## 10. Règles de travail du projet

- **Branche** : continuer sur `feat/voix-appui-espace`, ou une branche `feat/<id>-<sujet>` à part issue de `dev` pour les correctifs
  propres à Windows. Ne jamais travailler directement sur `dev` ni `main`.
- **Commits** : en français, `type(domaine): résumé` (par exemple `fix(voix): Espace maintenu sous Windows`), un commit par
  fonctionnalité ou correctif, corps = ce qui a été mesuré ou testé. **Sans** ligne `Co-Authored-By`.
- **Tests** : écrire le test qui échoue d'abord, puis corriger le code. Ne jamais affaiblir ni supprimer un test pour passer au vert.
- **Documentation** : en français ; notes de release en anglais.
- **CI** : `windows.yml` se lance sur la PR vers `dev` et vérifie la compilation, l'installeur et le démon. La CI complète
  du moteur (`./ci.sh`) se lance sous Linux (ou en CI GitHub) ; sous Windows, lancer les tests Flutter ciblés (§ 7).
- **Pousser** la branche, ouvrir une PR vers `dev`, attendre les vérifications GitHub vertes. Un test de contrat peut être
  rouge si le contrat d'API a changé : voir l'historique de `app/test/rust_bridge_test.dart`.

## 11. Dépannage

| Symptôme | Piste |
|---|---|
| `flutter pub get` échoue sur un lien symbolique | activer le mode développeur Windows (§ 3) |
| La construction échoue dans `webview_all_windows` | `$env:CL = "/D_SILENCE_EXPERIMENTAL_COROUTINE_DEPRECATION_WARNINGS"` |
| « La dictée a rencontré une erreur » dès le clic | DLL absente à côté de `mira.exe` (§ 4), ou chemin du modèle refusé (R5) |
| Le texte n'apparaît jamais, mais les barres bougent | le modèle charge très longtemps (R6, R7) ou a échoué : lancer `mira.exe` depuis un terminal et lire sa sortie |
| Les barres ne bougent pas | micro bloqué ou muet (R2), ou mauvais périphérique par défaut |
| Un espace apparaît en trop avec Espace maintenu | R1 |
| Le téléchargement échoue à la fin | antivirus qui verrouille le dossier (R6) ; relancer, ou exclure temporairement le dossier |
| `flutter test` plante sur `Link.createSync` | R3 |
| `flutter test` ne trouve pas `sherpa-onnx-c-api.dll` | R4 |
| `mira.exe` ne démarre pas, rien ne s'affiche | lancer depuis un terminal pour lire l'erreur ; **À VÉRIFIER** : sous Linux l'application n'accepte qu'une instance (une seconde se fond dans la première), ce que le lanceur Windows ne semble pas imposer |

## 12. Fichiers à connaître

| Fichier | Rôle |
|---|---|
| `app/lib/voice/dictation_controller.dart` | état de la dictée, micro avant modèle, arrêt jusqu'au bout, niveau sonore |
| `app/lib/voice/sherpa_recognizer.dart`, `sherpa_isolate.dart`, `sherpa_config.dart` | reconnaissance dans un isolate, modèle gardé chaud |
| `app/lib/voice/model_keeper.dart` | échéance de libération du modèle |
| `app/lib/voice/record_audio_source.dart` | micro (plugin `record`, sur Windows : `record_windows`) |
| `app/lib/voice/voice_model_store.dart`, `voice_model_archive.dart` | téléchargement, SHA-256, extraction, suppression |
| `app/lib/voice/voice_preferences.dart` | réglages mémorisés (langue, Espace maintenu, délai, garde) |
| `app/lib/ui/push_to_talk_space.dart` | Espace maintenu (la logique de touche, **R1**) |
| `app/lib/ui/mic_button.dart`, `dictation_bars.dart` | bouton micro, appui long, barres de niveau |
| `app/lib/ui/voice_settings.dart`, `voice_push_settings.dart` | onglet Voix |
| `app/windows/flutter/generated_plugins.cmake` | plugins natifs Windows (`record_windows`, `sherpa_onnx_windows`) |
| `packaging/windows/` | `build.ps1`, `build-installer.ps1`, `test-installer.ps1`, `mira.iss` |
| `.github/workflows/windows.yml` | CI Windows |
