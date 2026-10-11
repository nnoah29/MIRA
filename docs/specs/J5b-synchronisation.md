# J5b — Synchronisation descendante, identité, temps réel

Références : ch. 10 entier (10.2 à 10.6, 10.9 à 10.14), D-13, D-20 à D-24, P4, P6, P7 ; invariants 7, 8, 20, 21, 23, 24, 25 ; scénarios S4, S5, S11, S12, S13. Pré-requis : J2 (dépôts), J3 (registre, capacités, rôles), J5a (`imap/`, `mime/`), J6 (`threads::assign_thread`) — si J6 n'est pas encore fusionné, appeler une fonction `threads::assign_thread` minimale (racine = premier `References`, sinon `In-Reply-To`, sinon `Message-ID`, sinon objet normalisé) et le signaler.

## 1. Contexte et découpage (`mira-core/src/sync/`)

- `SyncContext { db: Arc<Database>, provider: Arc<dyn Provider>, account: Account, clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>, events: EventSink }` (l'`EventSink` écrit l'événement en base puis le diffuse ; s'il n'existe pas encore, un trait minimal `EventSink { fn emit(&self, kind: EventKind) }` dans `events/` suffit).
- `sync/download/` : étapes de la descente (D-20) ; `sync/strategy/` : détection des changements (D-22) ; `sync/identity/` : reconnaissance d'un message (D-13, P7) ; `sync/resync/` : renumérotation (10.5) ; `sync/window/` : fenêtre et « remonter plus loin » (D-21) ; `sync/realtime/` : IDLE et relevé périodique (10.6) ; `sync/supervisor/` : une tâche par compte, budget de connexions, backoff, états du compte.

## 2. Descente d'un compte (S4)

1. `list_mailboxes` → `upsert_mailbox` avec `infer_mailbox_role` ; les boîtes disparues côté serveur sont supprimées du miroir (le serveur gagne, D-23) et un événement `MailboxDeleted` est émis.
2. Pour chaque boîte synchronisée, **réception d'abord**, puis les autres (10.12) : `sync_mailbox`.
3. Préchargement des corps des `prefetch_bodies` (défaut 50) messages les plus récents de la réception : `fetch_source` → `store_raw_source` (zstd) → `mime::parse_message` → `store_parsed`, pièces jointes décrites, index FTS (objet, adresses, texte), aperçu, `ContentState::BodyPresent`, événement `BodyAvailable`.
4. Événements `SyncStarted`, `SyncProgress { step, done, total }` (au moins une fois par étape et par lot de 100), `SyncFinished` ou `SyncFailed { code }`.

## 3. Synchroniser une boîte

- **Première fois** (pas de repère) : `uids_since(maintenant − window_days)` ; en-têtes par lots de 100, du plus récent au plus ancien ; pour chaque en-tête : `provider.translate_attributes(extra)` → identité (§4) → création du message, de l'apparition, des correspondances `server_ids`, du fil, de l'index FTS (objet + adresses), drapeaux (D-14 : drapeaux du serveur portés par le message). Pas d'événement `NewMessageReceived` lors de la première synchronisation (seulement la progression).
- **Ensuite**, stratégie selon `EffectiveCapabilities` (J3) :
  - `Qresync` : `select(resync = repère)` → `vanished` (retirer les apparitions, purger les messages orphelins, `MessageDeleted`) et `changed` (drapeaux) ; nouveaux UID ≥ ancien `uid_next` → en-têtes → messages, événement `NewMessageReceived` pour chacun.
  - `Condstore` : `fetch_flags(fenêtre, changed_since = highest_modseq)` ; nouveaux par UID ; suppressions par différence entre les UID du serveur dans la fenêtre et les apparitions locales.
  - `FullCompare` : `fetch_flags(fenêtre)` et comparaison complète ; mêmes règles.
  - Chaque repli choisi est journalisé (`tracing::info!` et événement `Diagnostic`) (10.14).
- **Le serveur gagne** (D-23, invariant 21) : les drapeaux du serveur écrasent le miroir, puis les actions en attente sont **rejouées localement par-dessus** (`actions::reapply_pending_local_effects(account)` — si J7 n'est pas fusionné, prévoir un point d'appel qui ne fait rien et le signaler).
- **Repère** (invariant 20) : le nouveau `SyncMarker` est écrit **dans la même transaction que les données qu'il couvre, ou après** ; jamais avant. Un test interrompt la synchro (coupure du faux serveur au milieu des en-têtes) et vérifie qu'une reprise ne perd ni ne duplique rien.
- `UIDVALIDITY` différent du repère → §5.

## 4. Identité d'un message (D-13, P7, invariants 7 et 23)

Ordre : (1) `stable_id` traduit par le provider → `find_by_server_id(stable)` ; (2) UID dans la même boîte et même `UIDVALIDITY` → `find_by_server_id(uid)` ; (3) seulement pendant une resynchronisation : recoupement `find_by_fingerprint(Message-ID, date, taille)` — **exactement un** candidat qui attend une revalidation → même message ; zéro ou plusieurs → **nouveau message** (doublon plutôt que fusion à tort). Un message reconnu dans une nouvelle boîte reçoit une **nouvelle apparition** (S11 : un seul message, deux apparitions). L'identifiant interne ne change jamais pour une raison serveur.

## 5. Renumérotation (10.5, S13)

`mark_mailbox_for_revalidation` → relire la fenêtre → pour chaque message du serveur : identité (§4, recoupement autorisé) → `revalidate_appearance(nouvel UID)` ou création → `purge_unrevalidated` (les messages sans apparition quittent le miroir) → `forget_uid_ids_for_mailbox` puis nouvelles correspondances UID → alerte `MailboxResynchronized`. Les actions en attente (qui référencent des `MessageId` et des cibles serveur) survivent.

## 6. Fenêtre (D-21, S12)

`extend_window(mailbox, extra_days)` : tâche longue (identifiant d'opération, `OperationProgress`, annulable par un `CancellationToken` coopératif — `tokio_util` n'est pas autorisé : un `Arc<AtomicBool>` suffit) qui télécharge les en-têtes de la période précédente. La fenêtre du compte est élargie seulement à la fin réussie.

## 7. Temps réel et supervision (10.6, 10.12, invariants 24 et 25)

- `ConnectionBudget` par compte : sémaphore `tokio::sync::Semaphore` de `max_connections` permis ; toute connexion IMAP en prend un (invariant 25, testé en comptant les connexions du faux serveur).
- Une seule synchronisation active par compte (invariant 24) : verrou par compte (`tokio::sync::Mutex` ou ensemble des comptes actifs) ; une demande pendant une synchro en cours est fusionnée (pas de seconde tâche).
- `realtime` : si `Idle` effectif, une connexion dédiée en IDLE sur la réception (renouvelée toutes les 25 min), le travail se fait sur une seconde connexion ; un changement déclenche la synchro de la réception (S5 : `NewMessageReceived`). Sinon relevé périodique au `min_poll_interval_secs` du provider.
- Erreurs réseau : compte `Offline` + `WentOffline`, reconnexion avec backoff exponentiel (1 s, 2 s, 4 s … plafonné à 300 s, injectable pour les tests) ; retour → `Connected` + `BackOnline` ; après toute coupure, resynchronisation (10.6).
- Erreur d'authentification : demander un rafraîchissement au module `auth` (point d'extension `CredentialProvider` : `async fn imap_login(&self, account) -> Result<ImapLogin, MiraError>` et `async fn refresh(&self, account) -> Result<bool, MiraError>`) ; échec → `AuthRequired` + `AccountStateChanged` ; les actions restent en file (S10).
- Arrêt propre : `SyncSupervisor::shutdown(deadline)` termine les tâches dans un délai borné (10.11).

## 8. Tests exigés (contre `FakeImapServer`, avec les 4 jeux de capacités du §10.14)

`scenario_s4_first_sync_downloads_inbox_first_with_progress`, `scenario_s5_idle_new_message_emits_event`, `scenario_s11_gmail_message_in_two_labels_is_one_message`, `scenario_s12_extend_window_downloads_older_headers`, `scenario_s13_uidvalidity_change_resynchronizes_mailbox`, `invariant_7_internal_id_stable_across_move_and_renumber`, `invariant_20_marker_advanced_after_data`, `invariant_21_server_wins_on_flags`, `invariant_23_ambiguous_fingerprint_keeps_two_messages`, `invariant_24_single_sync_task_per_account`, `invariant_25_connections_within_provider_limit`, suppression ailleurs retirée du miroir (S8 côté synchro), reprise après coupure au milieu d'une synchro, pour chaque jeu de capacités (complet, sans QRESYNC, sans MOVE, minimal) : même état final du miroir.
