# Démon, phase 4 : démon résident, notifications et icône

> **Statut :** spec à relire. Elle précise la phase 4 de `2026-11-04-architecture-demon-design.md` (autorité sur le reste : transport, protocole, v4). Chaque choix ci-dessous a été validé avec l'utilisateur le 2026-11-07.

## 1. Objectif et critère de fin

Fermer l'app : la synchronisation et les notifications continuent. Cela suppose un démon supervisé, des notifications et une icône portées par le démon, une voie de mise à jour propre et un démon livré dans les paquets.

**Périmètre :** Linux de bureau, paquets deb, rpm, arch et tarball pour le mode résident. AppImage, flatpak et snap n'ont que le démon à la demande ; flatpak et snap sont repoussés après la phase 4 (l'utilisateur envisage de les abandonner).

**Hors périmètre :** TLS du jeton TCP et moteur distant (v4), Windows et macOS (phase 5), cible web.

**Invariant :** le contrat JSON ne change que par un ajout compatible (1.6 vers 1.7, §4). Android et iOS gardent le pont FFI.

## 2. Décisions

| # | Décision | Raison |
|---|---|---|
| 1 | Toute la logique de notification vit dans le moteur (`mira-host`), pas dans Dart. | Une seule source de vérité, servie par le démon et par le pont. |
| 2 | L'affichage système est fourni par l'hôte (`NotificationSink`) : D-Bus pour le démon, Dart pour Android et le repli embarqué. | Android n'a d'API de notification qu'en Java/Kotlin ; le dépôt interdit `unsafe`. |
| 3 | Le focus de la fenêtre est une **présence** (`set_presence`), mécanisme distinct des notifications. | Réutilisable (relevé, multi-appareils en v4). |
| 4 | Au premier lancement, l'app demande une fois « Garder MIRA actif en arrière-plan ? », « oui » pré-sélectionné. | Un client mail qui reste actif doit l'avoir été dit. |
| 5 | Réponse « non » : le démon reste l'unique chemin Linux ; l'app le lance, il s'arrête 30 s après le départ du dernier client. | Un seul chemin à tester et à maintenir. |
| 6 | `keyring_locked` : réessai périodique (5 s croissant jusqu'à 60 s) et sur `retry_unlock`, une seule notification d'attente. | La synchro reprend seule dès le déverrouillage. |
| 7 | Icône : menu minimal (Ouvrir MIRA, ligne d'état, Quitter MIRA), sans pastille ni pause. | Simplicité ; chaque ajout est une surface de test. |
| 8 | `update_required` : redémarrage automatique si aucun autre client n'est attaché, sinon demande. | §7.3 de la spec parente. |
| 9 | Fichier de configuration : démarrage seulement (TOML dans le dossier MIRA, options de ligne de commande prioritaires). Le comportement reste dans `EngineSettings`. | Une source de vérité par paramètre ; l'app n'écrit jamais dans le fichier. |
| 10 | Tests : faux récepteur pour les décisions, vrai bus D-Bus privé (`dbus-run-session`) pour les adaptateurs ; son absence fait échouer le test, sans `#[ignore]`. | Teste aussi la couche réelle sans bureau. |

## 3. Démon et supervision

- **Résident** (réponse « oui », paquets deb, rpm, arch, tarball) : l'app exécute `systemctl --user enable --now mira-daemon.service`. L'unité, livrée par le paquet, est `Type=simple`, `Restart=on-failure`, `WantedBy=default.target`. Le démon démarre à chaque login et est relancé s'il plante.
- **À la demande** (réponse « non », AppImage, flatpak, snap) : l'app lance le démon, qui s'arrête 30 s après le départ du dernier client. Un client qui revient dans ce délai l'annule.
- **Configuration** (`<dossier MIRA>/daemon.toml`) : dossier de données, profil, chemin de la socket, niveau de journal, `resident`. Les options de ligne de commande l'emportent. L'app ne l'écrit jamais, sauf la clé `resident` qui suit le choix de la décision 4.
- **États de `hello_ok`** : `starting`, `ready`, `keyring_locked`, `update_required`. Le démon en produit désormais les quatre.
- **`set_poll_secs`** : écrit dans `EngineSettings`, pris en compte sans redémarrage ; état courant annoncé dans `hello_ok`.
- **`update_required`** : le démon accepte la connexion mais n'autorise que `host_op shutdown`. L'app redémarre le démon (`systemctl --user restart` en résident, `shutdown` puis relance sinon), automatiquement si aucun autre client n'est attaché, sinon après avoir prévenu.
- **Sortie immédiate** : l'app surveille le processus qu'elle a lancé (ou l'état de l'unité) et affiche l'erreur dès sa fin, au lieu d'attendre 20 s.

## 4. Présence, réglages, notifieur

**Présence (`mira-host`).** Opération d'hôte `set_presence { focused }` ajoutée au protocole. L'hôte garde `client_id` vers état et expose `is_active()` (au moins un client au premier plan). Une connexion qui tombe retire son entrée. Le pont FFI gagne une fonction équivalente (régénération de `flutter_rust_bridge`).

**Réglages (contrat 1.7).** `EngineSettings` reçoit `notifications { enabled, preview, sound, when_focused }`, avec les défauts actuels (activé, aperçu, son, faux). Les heures calmes et comptes en sourdine existent déjà ; l'événement `NewMessageReceived` porte `silenced` (à vérifier au plan : sinon le notifieur relit les réglages). Migration une fois : au premier lancement avec un moteur 1.7, l'app recopie `notify`, `notifyPreview`, `notifySound`, `notifyFocused` dans le moteur puis ne les lit plus. Test de compatibilité 1.6 vers 1.7 (champ absent).

**Notifieur (`mira-host`).** Il écoute les événements. Sur `NewMessageReceived` non silencieux, avec `enabled` et (`when_focused` ou aucune présence active), il lit l'en-tête du message et compose `Notification { id, title, body, sound, open_target }`. Sans `preview`, le corps est générique. Aucun secret dans la notification ni dans les journaux. Il la remet à un `NotificationSink` :

- **Démon** : récepteur D-Bus (`org.freedesktop.Notifications`) ; le clic relance l'app avec le message à ouvrir.
- **Pont** (Android, repli Linux) : récepteur qui renvoie la notification à Dart, qui l'affiche avec `flutter_local_notifications` sans aucune règle.

Le code Dart de décision (`newArrivals`, `NotifySettings.isQuiet`, etc.) est supprimé.

**Point à vérifier au plan :** l'app accepte-t-elle un argument « ouvrir ce message » et une seconde instance transmet-elle à la première ? Sinon, petite tâche ajoutée.

## 5. Icône de zone de notification

Dans `mira-daemon`, protocole StatusNotifierItem sur D-Bus, derrière un trait comme le récepteur. Menu : « Ouvrir MIRA » (aussi le clic gauche), ligne d'état non cliquable, « Quitter MIRA » (arrête le démon proprement ; en résident il reste arrêté jusqu'au prochain login ou une réactivation depuis l'app). Sans hôte de zone de notification (GNOME sans extension), l'icône est absente : le démon l'écrit une fois dans son journal et continue. Bibliothèque : `zbus` est déjà dans l'arbre ; le choix d'une crate StatusNotifierItem ou d'une écriture directe se fait au plan, après `cargo-deny`, consigné dans AG-92.

## 6. Paquets

deb, rpm, arch et tarball embarquent `mira-daemon` à côté de l'app et l'unité dans `/usr/lib/systemd/user/` (le script du tarball l'installe dans `~/.config/systemd/user/`). L'app active ou désactive le mode résident par `systemctl --user enable|disable` et ne touche jamais les fichiers du paquet. À la mise à jour du paquet, aucun script : `update_required` est traité au prochain lancement.

## 7. Tests

Ciblés, jamais la suite complète sans l'accord de l'utilisateur.

- **Décisions de notification** : table de cas (`enabled`, `silenced`, `preview`, `sound`, `when_focused`, présence), corps générique sans `preview`, aucun secret.
- **Présence** : plusieurs clients, tombée de connexion.
- **Protocole** : `set_presence`, `update_required` (seul `shutdown` passe), `retry_unlock`, `set_poll_secs` persisté, `keyring_locked` puis `ready` avec un faux trousseau.
- **Cycle de vie** : arrêt après délai de grâce (horloge simulée), retour d'un client à temps, sortie immédiate détectée.
- **Adaptateurs D-Bus** : notification et icône contre `dbus-run-session`.
- **Contrat** : jeu existant vert sur ses trois chemins ; compatibilité 1.6 vers 1.7.
- **Sécurité** : « aucun secret dans les trames » étendu aux jetons OAuth, avec un scénario OAuth complet contre le testkit.
- **Dart** : migration des préférences, question du premier lancement, redémarrage sur `update_required` ; `flutter analyze` propre, `tools/i18n-audit` à 0 manquant.
- **Unité et paquets** : `systemd-analyze --user verify`, présence du binaire et de l'unité dans chaque paquet.

## 8. Tâches

Une branche par groupe, un commit par tâche, fichiers disjoints. Dépendances : 1, puis 2 et 3 en parallèle, puis 4 et 5, puis 6, puis 7. La tâche 8 est libre.

1. **Fondations du démon** : fichier de configuration, états de l'hôte, `keyring_locked` et `retry_unlock`, `set_poll_secs`, `update_required` limité à `shutdown`, démon à la demande, sortie immédiate.
2. **Présence** : `set_presence` (protocole, hôte, pont FFI).
3. **Réglages de notification** : contrat 1.7 et migration.
4. **Notifieur** : cœur et faux récepteur ; récepteur D-Bus du démon ; récepteur du pont et affichage Dart.
5. **Icône** : trait, adaptateur D-Bus, menu.
6. **Côté app** : question du premier lancement, activation de l'unité, redémarrage sur `update_required`, suppression de l'ancien code de notification.
7. **Paquets et documentation** : unité, intégration aux quatre formats, `PROTOCOLE-DEMON.md`, `ARCHITECTURE-IMPLEMENTEE.md`, AG-92.
8. **Test OAuth dans les trames.**

## 9. Dettes réglées et dettes laissées

**Réglées :** démon sans supervision, `update_required` sans issue, `keyring_locked` / `retry_unlock` / `set_poll_secs`, fichier de configuration, attente de 20 s sur sortie immédiate, notifications et icône, packaging, OAuth dans les trames.

**Laissées :** jeton TCP en clair (TLS et appairage en v4), Windows et macOS (phase 5), flatpak et snap (après la phase 4).
