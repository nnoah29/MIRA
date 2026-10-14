# Décisions prises par l'agent

Format : **AG-xx — titre** · contexte · options · choix · raison · retour arrière.

## AG-01 — Garde-fous écrits en Python 3 (bibliothèque standard)
- **Contexte** : `tools/check-norms` et `tools/check-architecture` doivent analyser le code Rust et `cargo metadata`.
- **Options** : crate Rust (avec `syn`) dans le workspace, ou script Python sans dépendance.
- **Choix** : Python 3, bibliothèque standard seulement.
- **Raison** : pas de nouvelle dépendance Rust ; l'outil n'est pas du code du moteur et n'est donc pas soumis à ses normes ; `python3` est présent ; un lexer maison suffit (chaînes, caractères, commentaires, accolades).
- **Retour arrière** : réécrire l'outil en crate Rust ; l'interface (code de sortie, format des messages) reste la même.

## AG-02 — Sous-agents lancés en `general-purpose` avec fichier de rôle
- **Contexte** : les rôles écrits dans `.claude/agents/` pendant la session ne sont pas chargés par l'outil Agent (types introuvables).
- **Choix** : lancer `general-purpose` (modèle sonnet pour implémenteurs et testkit, opus pour relecteur et qualité), en demandant de lire d'abord `.claude/agents/<rôle>.md`, et en interdisant explicitement les outils web dans la consigne des rôles qui n'y ont pas droit.
- **Raison** : seul moyen de déléguer sans redémarrer la session. Les définitions restent valables pour une session future.
- **Retour arrière** : relancer Claude Code ; les types `implementeur-rust`, etc. seront alors disponibles.

## AG-03 — Connexion en clair autorisée seulement vers la boucle locale
- **Contexte** : le testkit écoute en clair sur 127.0.0.1 ; la production exige TLS.
- **Choix** : `Security::Plaintext` existe dans le vocabulaire, mais le cœur refuse (`tls.plaintext_refused`) toute connexion en clair vers un hôte qui n'est pas `127.0.0.1`, `::1` ou `localhost`.
- **Raison** : tester le vrai client IMAP/SMTP contre de faux serveurs sans certificats, sans jamais affaiblir la production.
- **Retour arrière** : ajouter des certificats de test (crate `rcgen`) et supprimer la variante.

## AG-04 — Contrat d'API : énumérations `Request`/`Response` et un handler par opération
- **Contexte** : §11.8 du brief (un handler par opération, routage par `match` exhaustif) et D-30 (même contrat en processus ou séparé).
- **Choix** : `Engine::execute(Request) -> Result<Response, MiraError>` ; chaque variante est routée vers son propre fichier de handler. Les énumérations sont sérialisables (même forme pour CLI `--json`, Flutter, ou un futur transport).
- **Raison** : contrat testable par instantané JSON, invariant 6 vérifiable par énumération.
- **Retour arrière** : ajouter des méthodes typées au-dessus ; les handlers restent.

## AG-05 — Stockage : `Database` + transaction `Tx`, dépôts en blocs `impl Tx`
- **Contexte** : aucune fuite de `rusqlite` hors de `storage/` (annexe H) ; écritures atomiques composées par la synchro.
- **Choix** : `Database { Mutex<Connection> }`, `write(|tx| …)` / `read(|tx| …)` ; un fichier par opération ou groupe d'opérations d'une entité. Accès synchrone depuis le code async (requêtes courtes, un seul écrivain).
- **Raison** : simplicité et lisibilité pour un développeur C++ ; un seul écrivain cohérent avec le verrou de profil (D-27).
- **Retour arrière** : déplacer les appels dans `spawn_blocking` ou un fil dédié derrière la même API.

## AG-06 — Le client IMAP réel est testé contre le faux serveur, sans maquette du client
- **Contexte** : P10 (testable sans réseau) et annexe H (remplaçable par un faux).
- **Choix** : le faux est côté serveur (testkit sur 127.0.0.1). Le client IMAP/SMTP du moteur est un type concret encapsulé, testé de bout en bout contre le testkit.
- **Raison** : on teste le vrai code de protocole ; pas de trait asynchrone dynamique compliqué.
- **Retour arrière** : extraire un trait `MailBackend` (6.10) le jour où un second backend arrive.

## AG-07 — `cargo deny` sans la vérification des avis de sécurité
- **Contexte** : `cargo deny check advisories` télécharge la base d'avis depuis GitHub (réseau hors crates.io, règle 2).
- **Choix** : `./ci.sh` exécute `cargo deny check bans licenses sources` seulement.
- **Retour arrière** : ajouter `advisories` quand le réseau est autorisé.

## AG-08 — Écarts acceptés du vocabulaire (rapport J1)
- `MiraError.context` est un `Box<ErrorContext>` (clippy `result_large_err`) ; JSON inchangé.
- Marqueurs d'identifiants nommés `AccountMarker`… (évite le conflit avec `ActionKind`).
- `Timestamp` et `Id<K>` sérialisés de façon transparente (entier, chaîne).
- `EmailAddress` désérialisée avec validation ; espaces refusés dans la partie locale.
- `EventKind::category()` et `account()` en `#[rustfmt::skip]` pour tenir en 30 lignes (match exhaustif de 44 variantes).
- `EventFilter` : un filtre par compte exclut les événements sans compte ; `after` strict.
- **Temporaire** : `storage::json_columns` porte `#[allow(dead_code)]` jusqu'à son premier usage par les dépôts J2 (à retirer à la fusion de J2).

## AG-09 — Comportements des faux SMTP et OAuth (rapport J4 partie B)
- SMTP : `MAIL FROM` avant `AUTH` → `530` (comme un serveur de soumission réel) ; `fail_next` est une file FIFO ; une panne en fin de DATA n'enregistre pas le message ; `drop_after_data_once` enregistre puis coupe sans répondre.
- OAuth : `approve`/`deny` refusent toute requête invalide et tout `redirect_uri` qui n'est pas en boucle locale ; code à usage unique consommé même si le vérificateur est faux ; le rafraîchissement ne renouvelle pas le jeton de rafraîchissement ; jetons générés par compteur, jamais journalisés.
- Modules `oauth::pkce` et `oauth::form` publics (fonctions pures testées).

## AG-10 — Comportements du faux IMAP (rapport J4 partie A)
- Un seul commit pour tout le serveur (niveaux interdépendants).
- `connection_count()` = total des connexions acceptées ; `active_connections()` = connexions ouvertes (à utiliser pour l'invariant 25).
- QRESYNC exige `ENABLE QRESYNC` avant SELECT (sinon `BAD`), comme un serveur réel ; MODSEQ n'apparaît dans les FETCH qu'une fois CONDSTORE activé.
- `renumber` redémarre les UID à 1 et change UIDVALIDITY ; `deliver_dated` permet des dates internes contrôlées (fenêtre de 90 jours).
- `X-GM-LABELS` : noms des boîtes contenant le message (`INBOX` → `\Inbox`, boîtes `\All` exclues).
- Non supportés : BODYSTRUCTURE, sections numérotées, LSUB, UNSELECT, NAMESPACE, STARTTLS, X-GM-RAW.

## AG-11 — Écarts acceptés de J2a
- Migration `v002` : colonne `accounts.profile_id` (le schéma v1 l'avait oubliée) et index unique partiel « une seule identité par défaut par compte ».
- `keyring` 4.2 : la feature par défaut `v1` fournit déjà SecretService (zbus), aucune dépendance système.
- Clé d'idempotence réutilisée pour une **autre** commande → `invalid_input.idempotency_conflict` ; même commande → même action.
- File d'actions : `server_targets_json` et `undo_json` pas encore remplis (`PendingAction` n'a pas ces champs) → à ajouter en J7.
- Profil : nom vide, `.`, `..` ou contenant `/`, `\`, NUL refusé (traversée de répertoire).
- Dépôt d'événements dans `storage/repositories/events/` ; `src/events/` reste pour le bus (J8).

## AG-12 — Écarts acceptés de J3
- CLI : `run(args, &mut CliIo) -> i32` (au lieu de `CliDeps`/`CliOutcome`) ; l'injection des dépendances du moteur viendra en J9.
- Enveloppe JSON : clés dans l'ordre alphabétique de `serde_json` (`contract_version, data|error, ok`) — sans effet pour un lecteur JSON.
- Stratégie `Qresync` dès que QRESYNC est annoncé (il implique CONDSTORE) ; `COMPRESS` = `COMPRESS=DEFLATE`.
- `X-GM-LABELS` : les noms sont supposés déjà décodés (UTF-7 modifié) par le client IMAP.
- Registre : pas d'élimination des doublons d'identifiant ; `find` renvoie le premier.

## AG-13 — Écarts acceptés de J2b
- Revalidation (10.5) : `mark_mailbox_for_revalidation` passe les UID en négatif pour éviter la collision `UNIQUE(mailbox_id, uid)` quand le serveur renumérote à partir de 1 ; les lectures rendent l'UID positif.
- `purge_unrevalidated` rend les orphelins sans les supprimer ; l'appelant supprime (et désindexe).
- `upsert_thread(account, root_key, subject, candidate_id)` : le dépôt ne génère pas d'identifiant.
- Recherche : ET implicite mot par mot, tri par date (pas de pertinence) pour permettre la pagination par curseur.
- `Attachment.cache` déduit de `cache_path`. **Reste à faire (J10)** : supprimer les fichiers de cache des pièces jointes lors de `discard_account_mirror` et de la suppression d'un compte.
- Curseur : chaîne hexadécimale opaque (`mira1|<date>|<id>`), invalide → `invalid_input.cursor`.

## AG-14 — Façade : comptes, identités, authentification (rapport J8a)
- Serveurs d'un compte : `NetworkPolicy` (par domaine, pour les bancs d'essai) > serveurs saisis > serveurs du provider ; `Account.servers` mémorisé seulement si saisi ou imposé.
- Utilisateur IMAP = adresse complète ; la méthode d'authentification doit figurer dans celles du provider.
- `AddAccount` sérialisé (verrou), idempotent par clé (migration v005 `account_idempotency`) : même clé + autre adresse → `invalid_input.idempotency_conflict`.
- Test de connexion : IMAP seulement (SMTP non testé à l'ajout).
- `ResumeAccount` → `Offline` jusqu'à la prochaine connexion réussie ; changements d'identité signalés par `AccountUpdated`.
- `CancelOAuth` idempotent ; `CompleteOAuth` attend 300 s au plus, annulable.
- **Reste à faire** : brancher la première synchronisation sur `AddAccount` (J8b) ; achever au démarrage une suppression de compte interrompue ; supprimer les caches de pièces jointes (J10).

## AG-15 — Profils de build allégés
- `/home` a atteint 99 % (répertoires `target/` de 16 à 30 Go par worktree). `[profile.dev]` et `[profile.test]` : `debug = "line-tables-only"`, `incremental = false`. `target/` principal : 30 Go → 5 Go.

## AG-16 — File d'actions locale et brouillons (rapport J7b)
- Une action est résolue en `LocalIntent` (selon `ProviderQuirks`), déroulée en `LocalStep` idempotents ; `UndoRecord { intent, inverse }` dans `undo_json`. Le rebase (D-23) re-déroule l'intention et n'applique que les étapes absentes.
- Apparition provisoire (déplacement ou libellé hors ligne) : UID stocké < −2³², lu `0`, `needs_revalidation` ; `add_appearance` la remplace par l'apparition réelle lors de la synchro.
- `cancel` possible tant que l'action est `Pending` ; `submit(Send)` refusé (passer par `schedule_send`) ; archive/corbeille/spam sans boîte de rôle → `server.mailbox_missing` dès la soumission ; `RemoveFromMailbox` sur compte à dossiers = archiver.
- Nouveaux codes : `incompatible_state.draft_not_editable`, `incompatible_state.action_not_retryable`.
- Garde-fou A6 : `ymail` détecté seulement hors d'un identifiant (faux positif `EmptyMailbox`) ; auto-test étendu.

## AG-17 — Reprise sans les worktrees locaux
- **Contexte** : la session reprend dans un conteneur dont le clone ne contient pas les branches `feat/E-01-j5b-sync`, `feat/E-01-j9a-cli`, `feat/E-01-j7c-replay-send` (jamais poussées).
- **Choix** : réimplémenter J5b, J7c, J9a depuis les specs (`docs/specs/`) sur la branche de travail, par sous-agents en worktree.
- **Raison** : seule option disponible ; les specs sont complètes.
- **Retour arrière** : si les worktrees d'origine réapparaissent, comparer et fusionner l'avance.

## AG-18 — Push autorisé en session cloud
- **Contexte** : session dans un conteneur éphémère ; des worktrees non poussés ont été perdus (AG-17).
- **Choix** : sur instruction explicite de l'utilisateur (2026-10-14), la règle absolue 4 est levée pour `git push` : on pousse la branche de travail et les branches d'agents vers `origin`, jamais vers `dev`/`main`.
- **Retour arrière** : revenir à « pas de push » si l'utilisateur le redemande.

## AG-19 — CLI sur la façade (rapport J9a)
- `mira_cli::run(args, io, deps)` est async ; `CliDeps` porte profil, registre, trousseau, horloge, ids, config OAuth, réseau, stdin.
- `provider list|detect` reste local (registre injecté, sans ouvrir de profil) ; les autres commandes passent par `Engine::open/execute/shutdown`.
- Secrets uniquement via `--password-stdin` (1re ligne, 4 Ko). Clé d'idempotence de `account add` : `--key`, sinon `cli-<id>`.
- OAuth en deux processus (`oauth-start` puis `oauth-complete`) impossible : table des flux en mémoire du moteur ; à traiter avec une commande combinée (J9).
- Ajouter une variante de `Request` : module `commands/<domaine>/`, bras dans `request_builder.rs` et `render/mod.rs`, exemples du test invariant 6.

## AG-20 — Rejeu, envoi, reprise (rapport J7c)
- `Message-ID` d'envoi dans `ServerTargets.sent_message_id` (serde default, sans migration) ; un envoi relancé par l'utilisateur reçoit un nouveau `Message-ID`.
- Le rejeu s'arrête au premier échec passager ou d'authentification (ordre, inv. 19) ; définitif ou succès : on continue.
- Classement : `Authentication` et `secrets.secret_missing` → auth ; `Network/Tls/LocalStorage/Internal/Cancelled/Secrets` et `Server` retryable → passager ; le reste → définitif.
- Après acceptation SMTP aucune erreur ne remonte (copie Envoyés, événements, suppression du brouillon : `tracing::warn`) : jamais de renvoi pour un échec secondaire.
- Coupure `network.connection_lost` après DATA → vérification immédiate dans Envoyés (Uncertain ou Succeeded). Reprise : Envoyés illisible → reste `InProgress`.
- Copie Envoyés sans Bcc ; `SaveDraft` sans destinataire = sans effet.
- **À faire (J8)** : `Engine::open` construit un `ReplayContext` par compte, appelle `recover_interrupted_actions` puis `replay_actions` ; implémentation de production de `ReplayCredentials` au-dessus de `CredentialVault`.

## AG-21 — Synchronisation (rapport J5b)
- Test S11 nommé `scenario_s11_message_in_two_labels_is_one_message` (A6 interdit le nom du fournisseur) ; A9 interdit « fenêtre » dans le cœur : les commentaires disent « plage », identifiants `window_*`.
- Orphelins supprimés seulement en fin de passe complète (sinon un déplacement changerait l'identifiant interne) ; une passe limitée à la réception peut laisser un orphelin invisible jusqu'au relevé complet.
- Recoupement (P7) appliqué chaque fois qu'un candidat unique attend une revalidation dans la boîte (couvre l'apparition provisoire d'un déplacement hors ligne).
- Resynchronisation interrompue : pas de nouveau marquage si des apparitions sont déjà « à revalider » ; les apparitions provisoires survivent.
- QRESYNC : nouveaux via `UID SEARCH UID n:*` ; CONDSTORE/comparaison : `UID SEARCH ALL` ; repère sans compteur → comparaison complète.
- IDLE seulement s'il reste un permis de réserve (limite ≥ 3), sinon relevé périodique + Diagnostic `sync.realtime`.
- Erreur serveur sur une boîte : Diagnostic Warn, les autres continuent. Pas de trait `LocalIntentReplayer` : appel direct de `reapply_pending_local_effects`.
- Limites : annulation de `extend_window` testée seulement avant le premier lot.

## AG-22 — Façade lecture et maintenance (rapport J8b)
- Pas de migration v006 (table `remote_content_allowances` déjà en v001). Téléchargement dans `fetching/` (pas `reading/fetch/`, qui interdit le réseau). `maintenance/` pour purge, diagnostic, export.
- `FetchMessageBody`, `SaveAttachment`, `SearchServer` rendent `Response::Operation` tout de suite (tâche `tokio::spawn`) ; résultat de recherche serveur par `SearchRequest::GetServerSearchResult { operation }` (16 derniers en mémoire, 200 hits max) — **le CLI doit exposer cette variante (invariant 6)**.
- `GetSafeHtml` : assainit avec `allow_remote` si autorisation par message ou expéditeur ; `ExportProfile` : comptes, identités, autorisations, brouillons, actions (`format_version: 1`), sans secret ni miroir ; `ExportDiagnostic` sans adresse.
- Limite : `ImapAccess` utilise le jeton rangé sans rafraîchissement automatique.

## AG-23 — Façade écriture et rejeu de fond (rapport J8c)
- Trois domaines `Action`, `Compose`, `Queue` et une enveloppe `Response::Write` (boîtes en écriture dans `Action`). `ApplyAction` rend une action par compte ; lot atomique (`actions::submit_batch`), sélection par requête plafonnée à 10 000.
- `UndoAction`/`CancelAction`/`CancelSend` appellent `cancel` (état `Pending` seulement).
- `replayer/` : une tâche de fond par compte (récupération puis `replay_actions`), sonde 250 ms, réveil au submit/retry/résolution ; `ReplayCredentials` de production sur le trousseau ; état Connected/Offline/AuthRequired. `Engine::shutdown` arrête par `abort`, la reprise se fait au démarrage suivant.
- Fusion J8b+J8c : tests de contrat réconciliés ; exemples d'écriture nommés (`write_requests`, `write_responses`, `numbered`) ajoutés à l'instantané ; le CLI rend les nouvelles réponses en JSON en attendant J9b.
- **Reste** : première synchro sur `AddAccount`, domaine Synchro de la façade, commandes CLI des variantes de lecture/écriture.

## AG-24 — CLI lecture/écriture (rapport J9b)
- Un CLI vit le temps d'une commande : les opérations longues (`message fetch`, `save-attachment`, `search server`) sont toujours attendues (`--timeout`, 120 s) ; les actions de file seulement avec `--wait` (60 s) sur `action apply`, `mailbox create|rename|delete|empty`, `draft send|resolve-send`, `queue retry`. Action Failed → code 16 ; Uncertain ou délai → 13 (l'action reste en file).
- `search server-result` exposé pour le contrat mais ne retrouve rien d'un autre processus (code 11).
- Instants : ms Unix, `AAAA-MM-JJ` ou `AAAA-MM-JJTHH:MM[:SS][Z]`, UTC. Destinataires `adresse` ou `Nom <adresse>` ; valeur vide efface ; `--body-file`.
- `mailbox set-synced` : boîtes en arguments positionnels, boîte inconnue → code 10.
- Dépendance `base64` ajoutée à mira-cli.

## AG-25 — Façade Synchro et cycle de vie (rapport J8d)
- `BackgroundSync { Off, On(SupervisorConfig) }` dans `NetworkPolicy` : Off par défaut (tests), On dans `production_config`. Sans runtime tokio, synchro désactivée ; `StartSync`/`ExtendWindow`/`ResyncMailbox` → `incompatible_state.sync_disabled`.
- `Engine::shutdown` synchrone (abort) ; `Engine::shutdown_within(deadline)` async pour l'arrêt propre ; `Drop` interrompt les tâches.
- `PauseSync` = pause du compte ; `StartSync` sur compte en pause → `incompatible_state.account_paused`. `AddAccount` lance la première synchro ; une suppression de compte interrompue est achevée à l'ouverture (`accounts/deletion.rs`).
- Une demande de synchro (Account / Mailbox / Resync) est servie par la tâche du compte par une passe démarrant après elle (inv. 24).
- Secret absent pendant la synchro → `authentication.rejected` (AuthRequired).
- Limite héritée : un message arrivé entre la fin de la première passe et l'établissement d'IDLE n'est vu qu'au relevé suivant (correctif possible : passe de la réception juste après le démarrage d'IDLE).
- **À faire (J9c)** : CLI `sync`, politique `BackgroundSync` par commande (Off hors synchro, On pour `sync start --wait`).

## AG-26 — CLI Synchro, politique de synchro de fond, OAuth en un processus (rapport J9c)
- `execute_request` fixe `background_sync` par commande : `On` seulement pour `sync start|extend-window|resync` et `account add|add-oauth --sync-wait` ; `Off` ailleurs. Arrêt par `shutdown_within(5 s)` quand la synchro tournait.
- `sync start --wait` accepté sans effet (le CLI attend toujours : un processus qui rend la main tue la synchro).
- `Engine::open` avec synchro `On` lance une passe pour chaque compte connu : `sync start --mailbox X` synchronise aussi les autres boîtes (comportement du cœur).
- `mira account add-oauth` : StartOAuth, URL sur stderr, CompleteOAuth borné (`--timeout` 300 s, code 13 et CancelOAuth au délai), puis AddAccount.
- Le `warn!` « première synchro non lancée » d'`AddAccount` est masqué par une directive par défaut du CLI : à supprimer côté moteur quand la synchro est `Off`.
- **Reste** : API moteur pour attendre la première synchro lancée par `AddAccount`.

## AG-27 — Corrections du lot A (envoi, MIME, brouillons)
- HTML→texte plafonné à 2 Mo, recherche de balise fermante sans allocation ; `decode_entities` sans second défaut quadratique ; `parse_message` hors `db.write` ; module `body_storage/` partagé par la synchro et le téléchargement à la demande.
- `begin_action_attempt` conditionnel à `state='pending'` (annulation vs rejeu).
- Délai global SMTP de 120 s (`send_within`) : dépassement après connexion = `network.connection_lost` (incertain, jamais renvoyé) ; seul le dépassement de connexion TCP reste `network.timeout`.
- Retrait de la copie serveur du brouillon via une action `SaveDraft` de retrait (clé `remove-draft-copy-<id>`) ; la `SaveDraft` d'un brouillon supprimé purge la copie au lieu d'échouer.
- Erreurs SMTP « client » : défaut `server.protocol` (seul « challenge » reste authentification) ; échec du service de jetons → `network.unreachable` (passager).
- **Non traité** : purge de la copie du brouillon dans `recover_interrupted_send` ; fournisseur d'identifiants unique pour `ImapAccess` (rafraîchissement) ; audit #4 (toute erreur de transport après la connexion pour un `Send`, dont `ETIMEDOUT` après DATA, doit être incertaine) — à vérifier dans un lot D.

## AG-28 — Corrections du lot B (synchro, budget, lecteur IMAP, suppression)
- Un `ConnectionBudget` par compte, tenu par le moteur (`ConnectionBudgets`, clé = identifiant du compte ; `candidate:<adresse>` pour un test avant ajout), pris avant tout `ImapClient::connect` (rejeu, téléchargements, recherche serveur, tests de connexion, synchro, extension de plage). `ImapAccess::connect` rend un `LeasedClient` ; au plus 16 demandes explicites en attente par compte (`server.rate_limited`).
- Téléchargement à la demande : ignore les apparitions à revalider, compare `uid_validity`, exige le même Message-ID (sinon `not_found.message`).
- Orphelins : recoupement contre l'orphelin unique de la passe (sans jumeau vivant) ; pas de purge si une boîte a été ignorée. **Limite** : boîte de destination traitée avant la source (désarchivage vers la réception) crée encore un nouveau message — fusion en fin de passe non faite.
- Déplacement : « déjà arrivé » = même Message-ID + même taille + date interne à ±60 s ; Message-ID en plusieurs exemplaires sans UID fiable = ambigu, rien n'est touché. **Limite** : deux copies identiques à ±60 s sont tenues pour la même (fermer en persistant « tentative commencée »).
- Lecteur IMAP : ligne 1 Mo, littéral 64 Mo, réponse 64 Mo, allocation progressive, délai d'inactivité réarmé par bloc ; dépassement = connexion `broken`.
- Préchargement : échec Server d'un message journalisé et ignoré ; pas de préchargement > 10 Mo.
- VANISHED > 1 M d'UID ou avec `*` → repli sur la comparaison complète ; dates internes bornées 1900–9999, `checked_*`.
- Rétention des événements : 30 jours ou 100 000 lignes, à l'ouverture et après chaque passe ; opérations : 256 terminées conservées.
- S14 : suppression des fichiers de cache de pièces jointes, attente de la passe de rejeu (30 s, verrou par compte `replayer/gates.rs`) ; `extend_window` refuse si l'UIDVALIDITY a changé (`incompatible_state.mailbox_changed`) ; la synchro respecte `NetworkPolicy`.
- Non traité : `extend_window` s'exécute encore en parallèle de la tâche du compte ; attente du rejeu sans test déterministe ; repli VANISHED sans test d'intégration.

## AG-29 — Corrections du lot C (HTML, auth, divers)
- Contenu distant : liste blanche (`cid:` et `data:image/*` hors SVG dans les attributs auto-chargés), URL normalisée avant test, `UrlRelative::Deny` (les URL relatives retirées ne sont pas comptées dans `blocked_resources`), `data:` interdit dans `href`.
- Trousseau : `block_in_place` sur exécuteur multi-fils, appel direct en mono-fil ; `GetSafeHtml` et `CheckIntegrity` hors verrou (`spawn_blocking`).
- Rafraîchissement de jeton sérialisé par compte (table statique `OnceLock` `profil/compte`) ; un seul `is_loopback_host` (127.0.0.0/8, `::1`, localhost) ; OAuth : `state` faux → 400 et l'écouteur continue, une tâche par connexion (10 s), TTL 300 s, comparaison en temps constant ; réponse de jetons ≤ 1 Mo.
- Noms de pièces jointes assainis (`mime/attachment_name.rs`), types MIME validés ; export : `create_new` + suffixe aléatoire (pas de `O_NOFOLLOW`, qui exigerait `libc`) ; `AttachFile` : ouverture unique, `take(MAX+1)` ; `read_secret` du CLI strict.
- `UpdateAccount` avec d'autres serveurs : test de connexion avec le secret rangé avant écriture (pas de nouvelle saisie : un serveur hostile verrait le secret — défense complète = changement de contrat).
- Dépendances retirées : `thiserror` (mira-core), feature `json` de reqwest, `async-imap`, `futures`, `anyhow` du workspace.
- **Non traité** : revue #28 (`X-GM-EXT-1`, `\Inbox` en dur → particularités du provider), audit #17 (`zeroize`), mutex std de la base sur threads tokio hors trousseau/GetSafeHtml/CheckIntegrity.

## AG-30 — Lot D : un envoi dont l'issue est inconnue n'est jamais renvoyé
- Pour une action `Send`, toute erreur réseau autre que `network.unreachable` est « issue inconnue » → `settle_interrupted_send` (Message-ID cherché dans Envoyés : `Succeeded` ou `Uncertain`). `network.timeout` n'est plus produit par le client SMTP : échec de connexion TCP (délai compris) = `network.unreachable` (passager) ; panne dans la session = `network.connection_lost`. **Corrige AG-27** sur ce point.
- Toute future erreur réseau passagère d'un envoi doit être normalisée en `network.unreachable` avant `settle_failure` (fait pour le rafraîchissement de jeton).
- La copie serveur du brouillon est aussi purgée quand l'envoi est retrouvé dans Envoyés à la reprise (échec seulement journalisé).

## AG-31 — Scénarios S1–S7 via le CLI : écarts constatés
- S5 temps réel : un CLI de courte durée ne garde aucune écoute IDLE ouverte ; testé au plus près (relevé incrémental, arrivées concurrentes à `sync start`). Une commande `watch` de longue durée est un ajout possible (v1+).
- S6 : un message au corps préchargé a sa source complète en base : `message attachments` indique `not_downloaded` mais `save-attachment` réussit hors ligne depuis la source locale (« cache local d'abord », `fetching/attachment.rs`). Comportement conservé : le but de « indisponible hors ligne » vise les messages en en-tête seul.
- S4 : les corps de la réception sont préchargés juste après ses en-têtes, avant les en-têtes des autres boîtes ; le §10.2 ne fixe pas cet ordre (« préchargement des plus récents »), conservé.
- Coupure réseau des tests simulée en visant un port fermé (`go_offline`/`go_online`).

## AG-32 — Scénarios S8–S14 via le CLI : écarts constatés
- Horloge manuelle partagée entre commandes (avancée par `pump_clock` pendant `--wait`) ; pour S12 l'horloge est réglée sur l'heure réelle (le faux serveur date en temps réel).
- `sync extend-window --days` s'ajoute à la plage courante (30 → 120 jours).
- Limites via le CLI : le moteur est exclusif par profil, donc ni `operation cancel` pendant un `extend-window`, ni suppression de compte pendant une synchro, ne sont testables en processus ; `auth reauthenticate` + `oauth-complete` dans deux processus impossible (AG-19) ; testés au plus près.
- `action apply --wait` sur un compte `auth_required` attend jusqu'au `--timeout` (code 13) au lieu d'un code 15 immédiat : amélioration prévue.
- Effacement S14 : les tables sont vidées mais des fragments FTS5 et le WAL gardent des sujets/adresses tant que les pages libres ne sont pas écrasées : prévu `PRAGMA secure_delete` + checkpoint à la suppression d'un compte.

## AG-33 — Effacement des traces d'un compte, attente sur compte à réauthentifier
- `PRAGMA secure_delete=ON` à l'ouverture ; `Database::erase_traces()` = `optimize` de l'index FTS5 puis `wal_checkpoint(TRUNCATE)` (checkpoint bloqué par un lecteur : `warn`, journal non tronqué), appelé par `erase_account` et `discard_mirror_of`. Pas de `VACUUM` (réécriture complète, double espace disque) : les traces d'une base créée avant ce correctif ne sont pas purgées rétroactivement.
- `--wait` : si une action à jouer appartient à un compte `AuthRequired`, sortie immédiate `authentication.rejected` (code 15, action suggérée `reauthenticate`), l'action reste en file. Testé pour `action apply` et `mailbox create` (mêmes chemins pour `draft send|resolve-send`, `queue retry`). Deux assertions de S10 passent de 13 à 15 (elles figeaient l'ancien défaut).
