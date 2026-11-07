# Phase 5a : Windows, moteur embarqué

> **Statut :** adoptée le 2026-11-07 sans relecture séparée (l'utilisateur a demandé d'enchaîner spec, plan et implémentation). Elle réalise l'étape 1 de l'option 3 : l'app Windows utilise le pont FFI, sans démon. Le démon Windows (TCP loopback, démarrage à la session, notifications natives, zone de notification) est l'étape 5b, dans un second plan. macOS est hors périmètre.

## Décisions

| # | Décision | Raison |
|---|---|---|
| 1 | Windows 5a = moteur embarqué seul, comme Android. | Seul périmètre que l'on peut préparer sans machine Windows. |
| 2 | Profil sous `%LOCALAPPDATA%\mira\profiles` (repli de `XDG_DATA_HOME`). `MIRA_HOME` garde la priorité. | Aucune variable `HOME` sous Windows. |
| 3 | Fichiers et dossiers privés : droits 0600/0700 sous Unix ; sous Windows, héritage des ACL du dossier utilisateur. | Le dossier `%LOCALAPPDATA%` n'est lisible que par l'utilisateur. |
| 4 | Secrets : trousseau `keyring` (Gestionnaire d'identifiants Windows, déjà compilé via `windows-native-keyring-store`). | Aucun nouveau code. |
| 5 | Dart : bibliothèque `mira_bridge.dll` ; politique d'ouverture = moteur embarqué hors Linux ; rendu HTML sans vue web (widgets) ; notifications par `flutter_local_notifications` avec `WindowsInitializationSettings`. | Réutilise les chemins Android. |
| 6 | Dossier `app/windows` généré par `flutter create --platforms=windows`, la DLL du pont copiée à côté de l'exécutable par CMake. | Cible Flutter standard. |
| 7 | Vérification : tests Linux ciblés pour le code partagé ; compilation Windows par un workflow GitHub `windows-latest` (aucun compilateur C croisé ici). | `ring`, `zstd` et SQLite demandent un compilateur C Windows. |

## Hors périmètre

Démon Windows, zone de notification, démarrage à la session, installeur (MSIX ou Inno Setup), signature, macOS.
