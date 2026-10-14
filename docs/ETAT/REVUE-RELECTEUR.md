# Revue indépendante (relecteur) — HEAD f91f2d4

Chemins relatifs à `engine/crates/mira-core/src` (C). Chaque point : vérifier d'abord en lisant le code, test de non-régression d'abord, puis correctif minimal.

## Bloquant
1. `C/mime/html_text.rs:18-24` (`hidden_length`, appelé par `strip_tags`) : copie en minuscules de tout le reste du document à chaque `<style>`/`<script>` → coût quadratique ; appelé sous `db.write` (`C/sync/download/prefetch.rs:46`, `body.rs:37`, `C/fetching/body.rs:19`) → gel du moteur à chaque démarrage. Corriger : recherche de balise fermante insensible à la casse sans allocation ; plafonner le HTML converti (1–2 Mo) ; `parse_message` HORS de `db.write`. Test : 5 Mo de `<style></style>` converti en < 1 s.

## Important
2. Course annulation/rejeu (`C/actions/replay/run.rs:27-28`, `attempt.rs:10-19`, `storage/repositories/actions/update.rs:12`) : `begin_attempt` réécrit `InProgress` sans vérifier `Pending` → un envoi annulé peut partir. Corriger : `UPDATE … WHERE id=? AND state='pending'` + contrôle du nombre de lignes, sinon ignorer.
3. SMTP sans délai après connexion (`C/smtp/transport.rs:13,31`, `send.rs:26`) : envelopper `send_raw` dans `tokio::time::timeout` (120 s) ; un dépassement après connexion doit donner `network.connection_lost` (incertain), JAMAIS `network.timeout` (passager → renvoi, inv. 26).
4. CLI : `BackgroundSync::On` (`mira-assembly/src/engine_config.rs:26`, `mira-cli/src/deps.rs:45`) et rejeu démarré à chaque commande puis `abort` (`mira-cli/src/commands/execution.rs:25-27`, `C/replayer/handle.rs:23-28`, `C/api/engine.rs:89-93` : `shutdown_within` n'attend que la synchro). Corriger : synchro ET rejeu désactivés par défaut dans le CLI (sauf `sync …`, commandes avec `--wait`) ; arrêt gracieux du rejeu : ne jamais interrompre entre `deliver` et `complete_attempt`.
5. Téléchargement à la demande avec UID périmé (`C/fetching/locate.rs:20-34`, `download.rs:15-16`) : ignorer les apparitions `needs_revalidation`, comparer `uid_validity` du SELECT au repère, vérifier le `Message-ID` de la source avant de ranger.
6. Invariant 25 : connexions IMAP hors budget du compte (`C/replayer/session.rs:38-41`, `C/fetching/access.rs:36`, test de connexion). Un `ConnectionBudget` par compte possédé par le moteur, pris avant tout `ImapClient::connect`.
7. Invariant 7 / D-13 cas 2 : sans identifiant stable, un déplacement fait ailleurs change l'identifiant interne (`C/sync/identity/recognize.rs:38-53`). Autoriser le recoupement (Message-ID+date+taille, candidat unique) contre les orphelins de la passe avant leur purge.
8. Orphelins purgés alors qu'une boîte a été ignorée (`C/sync/download/account.rs:24-31,45`) : ne pas purger si une boîte a été ignorée.
9. Déplacement qui purge la source par coïncidence de `Message-ID` (`C/actions/replay/execute/moves.rs:43-57,60-76`, `locate.rs:85-88`) : n'appliquer « déjà arrivé » qu'après une tentative précédente (`attempts > 0` / reprise depuis `InProgress`), recouper taille et date interne ; `find_uid` ne doit pas prendre `.first()` parmi des doublons.
10. Contenu distant contournable (`C/html/remote.rs:29-35`, `sanitize.rs:32-44`) : `<img src="ht&#9;tps://…">`, URL relatives `/\h`, `\\h`. Inverser la logique : n'autoriser que `cid:` et `data:image/*` après analyse `url::Url` (relatif = distant) ; `url_relative(UrlRelative::Deny)` ; tests.
11. Noms/recherches non ASCII non encodés (`C/imap/protocol/quoting.rs:9-17`, `C/imap/operations/search.rs:47-48`) : UTF-7 modifié (RFC 3501 §5.1.3) pour les noms de boîtes (encodage et décodage) ; `SEARCH CHARSET UTF-8` avec littéral.
12. Copie serveur du brouillon jamais supprimée (`C/send/execute.rs:43-58`, `C/drafts/delete.rs:10-17`) : purger dans Brouillons par Message-ID `<draft-…@mira.local>` après envoi et à la suppression, sans faire échouer l'envoi.
13. S14 incomplet (`C/accounts/deletion.rs:10-22`, `C/storage/repositories/accounts/write.rs:83-95`, `C/api/handlers/account/delete.rs:14-18`) : supprimer les fichiers de cache de pièces jointes (AG-13) ; arrêter ET attendre la tâche de rejeu du compte avant d'effacer.

## Mineur
14. `C/auth/renewal.rs:14-39` : rafraîchissement de jeton non sérialisé par compte → `tokio::Mutex` par compte + revérification de l'expiration.
15. `C/auth/loopback.rs:13-38` : analyse d'URL à la main (`http://evil.com\@127.0.0.1/`) → `reqwest::Url`.
16. Deux `is_loopback_host` (`C/auth/loopback.rs:7`, `C/imap/connection/policy.rs:10`) → une seule fonction.
17. Flux OAuth : entrées sans expiration (`C/auth/flow_table/lifecycle.rs:14-22`), `/callback?state=x` tue le flux (`C/auth/flow/redirect.rs:73-75`) → expiration 300 s, ignorer un `state` faux et continuer d'écouter.
18. `C/auth/tokens/http.rs:32-42` : lire la réponse par morceaux avec `MAX_BODY`.
19. `C/imap/protocol/reader.rs:16-55` : borner la ligne (1 Mo hors littéraux, 64 Mo au total).
20. `C/imap/parse/uid_set.rs:3-15` : VANISHED tronqué en silence > 1 M d'UID, `*` ignoré.
21. `C/maintenance/export_file.rs:29-36` : `create_new(true)` + nom unique (comme `fetching/atomic_write.rs`).
22. Duplication du rangement de corps (`C/sync/download/body.rs:25`, `C/fetching/ingest.rs:47`) : `ingest_source` ne remplit pas l'aperçu ; deux fournisseurs d'identifiants, `ImapAccess` sans rafraîchissement du jeton.
23. `C/smtp/errors.rs:47-61` : défaut = `server.protocol`, pas authentification.
24. `C/send/deliver.rs:17-19` : échec de rafraîchissement du jeton → pas `network.connection_lost` (faux incertain) ; utiliser `network.unreachable`/`timeout`.
25. `C/drafts/scheduling.rs:221` : rejeu de `schedule_send` avec la même clé après succès → renvoyer l'action existante.
26. `C/operations/registry.rs:37-49` : purger la table des opérations terminées.
27. `C/sync/window/extend.rs:104` : comparer la validité de la boîte au repère ; exécution parallèle à la tâche du compte.
28. Frontière : `X-GM-EXT-1` (`C/imap/operations/labels.rs:9`) et `\Inbox` (`C/actions/replay/execute/labels.rs:13`) codés en dur → venir des particularités du provider.
29. `C/api/handlers/account/changes.rs:25-26` : changer les serveurs exige un test de connexion ou nouvelle saisie du secret.
30. `C/sync/session/connection.rs:22-26` : la synchro ignore `NetworkPolicy`.
31. Appels bloquants sur threads tokio (keyring D-Bus, mutex std de la base) → `spawn_blocking` pour le trousseau.
