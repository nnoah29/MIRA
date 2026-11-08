# Windows, phase 5b : le démon sous Windows

> **Statut :** décisions adoptées par l'utilisateur (« recommandations adoptées, exécution native »). Suite de `2026-11-07-windows-phase-5a-moteur-embarque-design.md`. macOS exclu.

## 1. Objectif

Fermer la fenêtre sous Windows et continuer à synchroniser et à notifier, comme sous Linux. Le travail est découpé en trois lots livrables séparément :

- **5b-1 (ce plan)** : `mira-daemon` compile et tourne sous Windows ; l'app s'y attache par TCP loopback et jeton, et le lance à la demande.
- **5b-2** : notifications du système (toast Windows) portées par le démon, clic qui rouvre l'app.
- **5b-3** : icône de la zone de notification, démarrage à l'ouverture de session (mode résident), question du premier lancement.

## 2. Décisions

| # | Décision | Raison |
|---|---|---|
| 1 | Transport Windows : TCP sur 127.0.0.1, port libre, jeton local. Pas de pipe nommé. | Dart n'ouvre pas de pipe nommé sans FFI ; le TCP loopback et le jeton sont déjà dans l'architecture parente. |
| 2 | Le démon publie son port dans `daemon.port`, à côté de `daemon.token`, dans le dossier du profil (`%LOCALAPPDATA%\mira\profiles\<profil>`). | Le client doit découvrir le port ; le dossier est privé à l'utilisateur par défaut sous Windows. |
| 3 | Instance unique : le fichier du port est ouvert en partage lecture seule pendant toute la vie du démon (`share_mode(FILE_SHARE_READ)`). Un second démon échoue à l'ouvrir (code 3). Il est supprimé à l'arrêt. | Sans socket Unix, rien d'autre n'empêche deux démons d'écraser le port l'un de l'autre. |
| 4 | Le port est publié dès qu'un écouteur TCP est lié, sur tous les systèmes ; Windows écoute en TCP par défaut (`tcp_port` absent vaut 0), Linux seulement sur demande. | Un seul code, testable sous Linux. |
| 5 | `zbus` et `ksni` ne sont des dépendances que sous Linux. Le module `desktop` n'a, hors Linux, que `Desktop` sans récepteur ni icône ; la sonde du trousseau est `NeverLocked` (le magasin Windows ne se verrouille pas). | Le démon compile et tourne sans bureau Linux ; 5b-2 et 5b-3 branchent les adaptateurs Windows. |
| 6 | App : le démon devient un choix possible sous Windows (`canUseDaemon`), moteur embarqué en repli (`EngineMode.auto`). | Même politique que Linux ; l'embarqué de la phase 5a reste le filet. |
| 7 | Le démon est lancé à la demande (`--exit-when-idle`), processus normal dont la sortie est vidée. | `inheritStdio` n'a pas de console à hériter dans une app graphique Windows. |
| 8 | Le zip Windows embarque `mira-daemon.exe` à côté de `mira.exe`. | L'app le cherche à côté de son exécutable. |
| 9 | CI : `windows.yml` compile `mira-daemon` et lance le test du fichier de port. | Seule vérification Windows possible depuis cette machine. |

## 3. Fichier du port

Contenu : le numéro de port en décimal, sans espace superflu. Aucun secret : le jeton reste dans son propre fichier. Un port périmé (démon planté) donne un refus de connexion, que l'app traite comme `DaemonAbsent`.

## 4. Hors périmètre de 5b-1

Toasts (5b-2), icône et démarrage automatique (5b-3), signature, installeur, TLS du jeton TCP (v4).

## 5. Tests

- Rust : écriture et relecture du fichier de port, refus d'un second démon, suppression à l'arrêt, résolution des chemins avec `daemon.port` (exécutables sous Linux).
- Dart : chemins Windows (`LOCALAPPDATA`), lecture du port, point d'accès TCP, binaire `mira-daemon.exe`.
- Windows (GitHub) : `cargo build -p mira-daemon`, test du fichier de port, `flutter build windows`.
