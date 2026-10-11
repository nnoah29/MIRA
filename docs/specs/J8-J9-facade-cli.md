# J8 — Façade du contrat d'API · J9 — CLI complet

Références : ch. 12 entier, D-26, D-27, D-29, D-30 (décision de départ 5 : bibliothèque async + CLI), 7.5–7.9 (comptes, identités, secrets), 15.2 (OAuth PKCE en boucle locale), S1, S2, S3, S14 ; invariants 6, 15, 27–33 ; AG-04 (énumérations `Request`/`Response`, un handler par opération).

## J8 — `mira-core/src/api/` et modules de service

### Ouverture et cycle de vie

- `Engine::open(config: EngineConfig, client_version: ContractVersion) -> Result<Engine, MiraError>` : vérifie la compatibilité (`CONTRACT_VERSION.is_compatible_with(client_version)`, sinon `invalid_input.contract_version`, invariant 33), résout les chemins du profil, prend le **verrou de profil** (D-27), ouvre la base, émet `EngineStarted`, reprend le travail interrompu (actions `InProgress` → vérification ; envois incertains, 10.11).
- `EngineConfig { profile: ProfileId, environment: ProfileEnvironment, registry: ProviderRegistry, secrets: Arc<dyn SecretStore>, clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>, oauth_client: OAuthClientConfig, network: NetworkPolicy }` — tout est injecté par l'assemblage (vraies implémentations) ou par les tests (fausses). `OAuthClientConfig { client_ids: BTreeMap<ProviderId, String>, endpoint_overrides: BTreeMap<ProviderId, OAuthSettings> }` : l'identifiant d'application appartient à la configuration, pas au provider (6.8) ; les tests remplacent les adresses OAuth par celles du faux serveur. `NetworkPolicy { server_overrides: BTreeMap<String, ServerSettings> }` permet aux tests de rediriger un compte vers le testkit.
- `Engine::execute(Request) -> Result<Response, MiraError>` (async) : `match` exhaustif, **un handler par variante** dans `api/handlers/<domaine>/<operation>.rs`.
- `Engine::subscribe(EventFilter) -> EventSubscription` (rattrapage depuis la base après `filter.after`, puis direct) ; `Engine::shutdown(deadline)`.

### Opérations (`Request`, une variante par ligne du §12.4 « v1 »)

| Domaine | Variantes |
|---|---|
| Moteur | `GetContractInfo`, `GetStatus`, `Shutdown` |
| Providers | `ListProviders`, `DetectProvider { address }` |
| Comptes | `PrepareAccount { address, provider: Option<ProviderId> }`, `TestConnection { draft: AccountDraft, credential: CredentialInput }`, `AddAccount { draft: AccountDraft, credential: CredentialInput, idempotency_key }`, `UpdateAccount { account, changes: AccountChanges }`, `PauseAccount { account }`, `ResumeAccount { account }`, `DeleteAccount { account }`, `ListAccounts` |
| Authentification | `StartOAuth { address, provider }` → `{ flow, authorize_url }`, `CompleteOAuth { flow }`, `CancelOAuth { flow }`, `StoreAppPassword { account, password }`, `Reauthenticate { account }` |
| Identités | `ListIdentities { account }`, `AddIdentity {…}`, `UpdateIdentity {…}`, `SetDefaultIdentity { identity }` |
| Boîtes | `ListMailboxes { account }`, `CreateMailbox {…, idempotency_key}`, `RenameMailbox {…}`, `DeleteMailbox {…}`, `SetSyncedMailboxes { account, mailboxes }`, `EmptyMailbox { mailbox, idempotency_key }` |
| Lecture | `ListMessages { view, filters, page }`, `GetMessage { message }`, `GetMessageBody { message }`, `FetchMessageBody { message }` (opération longue), `GetRawSource { message }`, `ListAttachments { message }`, `SaveAttachment { attachment, destination }` (opération longue, écriture atomique via fichier temporaire puis renommage, 12.7), `GetSafeHtml { message }`, `AllowRemoteContent { scope: Message(id) \| Sender(address) }` |
| Fils | `ListThreads { mailbox, page }`, `GetThread { thread }` |
| Recherche | `Search { query, filters, page }`, `SearchServer { account, query }` (opération longue, annulable) |
| Actions | `ApplyAction { kind, targets: TargetSelection, idempotency_key }` (lot : liste de messages ou résultat d'une requête), `UndoAction { action }` |
| Rédaction | `CreateDraft { account, identity, mode: New \| Reply{message} \| ReplyAll{message} \| Forward{message} }`, `UpdateDraft { draft, changes }`, `AttachFile { draft, path }`, `RemoveAttachment { draft, position }`, `DeleteDraft`, `GetDraft`, `ListDrafts { account }`, `SendDraft { draft, at: Option<Timestamp>, idempotency_key }`, `CancelSend { action }`, `ListOutbox { account }`, `ResolveUncertainSend { action, resend }` |
| File d'actions | `ListActions { account, states }`, `RetryAction { action }`, `CancelAction { action }` |
| Synchro | `StartSync { scope: All \| Account \| Mailbox }` (opération longue), `PauseSync { account }`, `GetSyncStatus`, `ExtendWindow { mailbox, days }` (longue), `ResyncMailbox { mailbox }` (longue) |
| Opérations | `GetOperation { operation }`, `CancelOperation { operation }` |
| Événements | `GetEvents { filter, limit }` (pour le CLI et les clients sans abonnement) |
| Maintenance | `GetStorageStats`, `PurgeAttachmentCache`, `RebuildSearchIndex`, `CheckIntegrity`, `ExportDiagnostic { destination }` (sans secret), `ExportProfile { destination }` (D-29 : configuration + intentions en JSON, sans secret ni miroir) |

Règles transversales (12.2) : lectures locales (invariant 27) ; écritures → `ActionId` immédiat (invariant 28) ; listes paginées (29) ; identifiants opaques (30) ; erreurs du contrat (31) ; événements numérotés sans secret ni contenu (32). `CredentialInput` = `Password(String) | AppPassword(String) | OAuthFlow(flow)` : la valeur est transmise **une seule fois**, rangée dans le trousseau, jamais renvoyée ni journalisée (S2, P9) ; `Debug` de `CredentialInput` masqué.

### Comptes (7.5–7.9, S1–S3, S14)

- `AddAccount` : provider (détecté ou choisi) → serveurs (provider, ou saisis pour « Autre », ou `NetworkPolicy`) → secret rangé (`SecretRef` profil + compte) → **test de connexion** → enregistrement du compte **et** de son identité par défaut (invariant 11) → `AccountAdded` → première synchro lancée (S4). Échec du test : rien n'est enregistré, le secret est supprimé.
- `DeleteAccount` (S14) : marque `deleting`, interrompt la synchro du compte, supprime ses données locales (cascade), ses secrets (`SecretStore::delete` pour chaque `SecretKind`), émet `AccountDeleted` ; aucune commande n'est envoyée au serveur ; les autres comptes ne sont pas touchés.
- OAuth (15.2) : `StartOAuth` crée un flux (verifier PKCE aléatoire, `state` aléatoire), ouvre un écouteur TCP sur `127.0.0.1:0`, renvoie l'URL d'autorisation (`redirect_uri = http://127.0.0.1:<port>/callback`). `CompleteOAuth` attend la redirection (délai borné), vérifie le `state` (`authentication.oauth_state_mismatch`), échange le code contre les jetons (`reqwest` vers `token_url`), range les jetons dans le trousseau (`AccessToken`, `RefreshToken`) sous l'identifiant de compte réservé par le flux. `CancelOAuth` ferme l'écouteur (`authentication.oauth_cancelled`).
- Rafraîchissement automatique et silencieux (7.7, S10) : `auth::refresh(account)` ; échec → `AuthRequired`.

### Tâches longues (12.2.4)

`OperationRegistry` : `start(kind) -> OperationId`, progression (`OperationProgress`), fin (`OperationFinished` / `OperationFailed`), annulation coopérative (`CancelOperation` → drapeau observé par la tâche, `OperationCancelled`, aucun fichier partiel exposé).

### Tests J8 (contrat)

`contract_snapshot_requests_and_responses` (instantané JSON versionné de chaque variante : un exemple sérialisé par variante, comparé à un fichier `tests/snapshots/contract_v1.json` — toute modification incompatible casse le test), `invariant_6_every_request_has_a_cli_command` (en J9), `invariant_15_no_secret_in_events_errors_or_responses` (scénario complet avec mot de passe et jetons, puis recherche des valeurs dans la base, les événements, les réponses JSON et les journaux capturés), `invariant_27_list_and_read_work_offline` (faux serveur arrêté), `invariant_28_actions_return_while_offline`, `invariant_30_ids_are_opaque_strings`, `invariant_32_events_numbered_and_resumable`, `invariant_33_incompatible_client_version_refused`, `scenario_s1_add_oauth_account`, `scenario_s2_add_app_password_account`, `scenario_s3_add_other_account_with_manual_servers`, `scenario_s14_delete_account_keeps_others_and_server`, verrou de profil (second moteur refusé).

## J9 — CLI `mira`

- Une sous-commande par variante de `Request` : `mira <domaine> <verbe> [options]` (par exemple `mira account add --address … --password-stdin`, `mira message list --mailbox <id> --limit 20 --cursor …`, `mira action apply mark-read --message <id> --key <clé>`, `mira draft send <id> --at …`, `mira sync start --account <id> --wait`, `mira events --after <seq>`). Les secrets ne passent **jamais** en argument de ligne de commande (visibles dans `ps`) : `--password-stdin`.
- `--json` : enveloppe versionnée `{contract_version, ok, data | error}` = reflet exact du contrat ; `--profile <nom>` ; erreurs sur stderr (sortie humaine) ; codes de sortie 0 / 10–19 / 84.
- `run(args, io, deps)` testable en processus avec des dépendances injectées (faux trousseau, testkit, horloge manuelle).
- Tests : `invariant_6_every_request_has_a_cli_command` (énumération de toutes les variantes de `Request` via un exemple sérialisé par variante, chacune doit être produite par au moins une ligne de commande), et **S1 à S14 exécutés via le CLI** contre le testkit (`tests/scenario_sNN_*.rs`).
