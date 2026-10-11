# J1 — Spécification du crate `mira-vocabulary`

Le vocabulaire commun (D-33) contient les types du domaine, les erreurs et événements du contrat, et l'interface provider. Il est **pur** : aucune entrée-sortie, aucune dépendance autre que `serde` (derive) et `thiserror`. Tous les crates en dépendent ; les providers ne dépendent **que** de lui.

Conventions pour tous les types :
- `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` (plus `Hash`, `Copy`, `PartialOrd`, `Ord` quand c'est utile ; pas `Eq` si un champ est flottant).
- Énumérations sérialisées en `snake_case` (`#[serde(rename_all = "snake_case")]`) ; énumérations à données en `#[serde(tag = "type", rename_all = "snake_case")]`.
- Champs publics (ce sont des structures de données). Pas de logique métier hors des quelques fonctions listées ici.
- Chaque `fn` : une ligne `///` en français ; ≤ 30 lignes ; ≤ 6 fonctions par fichier ; aucun commentaire dans les corps.

## Arborescence imposée

```
src/lib.rs                      //! + `pub mod` + réexports publics (pub use) des types principaux
src/ids/mod.rs                  Id<K> générique
src/ids/kinds.rs                marqueurs et alias de types
src/ids/keys.rs                 ProviderId, IdempotencyKey, Cursor, EventSeq
src/time.rs                     Timestamp
src/address/mod.rs              EmailAddress (validée)
src/address/entry.rs            AddressEntry (adresse brute d'un en-tête)
src/contract/version.rs         ContractVersion, CONTRACT_VERSION
src/error/category.rs           ErrorCategory
src/error/action.rs             SuggestedAction
src/error/context.rs            ErrorContext
src/error/mira_error.rs         MiraError (+ constructeurs)
src/error/codes/<catégorie>.rs  catalogue des codes stables (constantes), un fichier par catégorie
src/events/category.rs          EventCategory
src/events/kind.rs              EventKind (+ category())
src/events/event.rs             Event
src/events/filter.rs            EventFilter (+ matches())
src/domain/account.rs           Account, AccountState, SyncSettings
src/domain/auth.rs              AuthMethod, OAuthSettings, SecretKind, SecretRef
src/domain/servers.rs           Security, ServerEndpoint, ServerSettings
src/domain/identity.rs          Identity
src/domain/mailbox.rs           Mailbox, MailboxRole, SyncMarker
src/domain/message.rs           MessageSummary, Flags, ContentState, Unsubscribe
src/domain/appearance.rs        Appearance
src/domain/attachment.rs        Attachment, CacheState
src/domain/thread.rs            Thread
src/domain/draft.rs             Draft, DraftState, DraftAttachment
src/domain/action.rs            PendingAction, ActionKind, ActionState
src/domain/view.rs              StandardView, ViewRef
src/domain/page.rs              Page<T>, PageRequest
src/provider/descriptor.rs      ProviderDescriptor
src/provider/limits.rs          ProviderLimits
src/provider/quirks.rs          ProviderQuirks, MailboxModel, ArchiveStrategy
src/provider/raw.rs             RawMailbox, RawMessageAttributes, ServerMessageTranslation
src/provider/provider.rs        trait Provider
tests/<thème>.rs                tests d'intégration (≤ 6 tests par fichier)
```

`Cargo.toml` du crate : dépendances `serde` (workspace, derive) et `thiserror` (workspace) ; dev-dépendance `serde_json` (workspace).

## Identifiants (8.5, invariant 30)

- `pub struct Id<K> { value: String, #[serde(skip)] kind: PhantomData<K> }` avec `#[serde(transparent)]` impossible à cause de deux champs → implémenter `Serialize`/`Deserialize` à la main (sérialisé comme une simple chaîne), ou utiliser `#[serde(transparent)]` si le compilateur l'accepte avec `PhantomData` (c'est le cas : `PhantomData` est de taille nulle). Implémenter `Clone`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Debug` **à la main ou via derive avec des marqueurs qui dérivent tout**.
- Fonctions : `Id::from_raw(value: impl Into<String>) -> Self`, `as_str(&self) -> &str`, `into_string(self) -> String`, `impl Display`.
- Marqueurs (structures vides, dérivent tout) et alias : `ProfileId`, `AccountId`, `IdentityId`, `MailboxId`, `MessageId`, `AttachmentId`, `ThreadId`, `DraftId`, `ActionId`, `OperationId`, `ViewId`.
- La génération (UUID v7) n'est **pas** ici : elle est dans le cœur, injectable. Le vocabulaire ne fait que transporter des chaînes opaques.
- `ProviderId(pub String)` : clé du catalogue (`"gmail"`, `"outlook"`, `"yahoo"`, `"generic"`), pas un identifiant opaque. `ProviderId::new(&str)`, `as_str()`.
- `IdempotencyKey(pub String)` fournie par le client (12.2.3). `Cursor(pub String)` opaque (pagination). `EventSeq(pub u64)` numéro d'événement, ordonné.

## Temps

`Timestamp { pub millis: i64 }` (millisecondes depuis l'époque Unix, UTC) ; `from_millis`, `as_millis`, `plus_millis(i64)`. Ordonné.

## Adresses

- `EmailAddress` (champ privé, chaîne normalisée) : `EmailAddress::parse(&str) -> Result<EmailAddress, MiraError>` (catégorie `InvalidInput`, code `invalid_input.address`). Règles : supprimer les espaces autour ; exactement un `@` ; partie locale non vide ; domaine non vide contenant au moins un point, sans espace, sans point initial ni final ni `..` ; domaine mis en minuscules (partie locale conservée telle quelle). Unicode accepté (adresses internationales, 12.7). `as_str()`, `local_part()`, `domain()`.
- `AddressEntry { pub name: Option<String>, pub address: String }` : adresse telle que lue dans un en-tête reçu (peut être invalide, on n'échoue jamais sur un message reçu).

## Version du contrat (12.9, invariant 33)

`ContractVersion { pub major: u32, pub minor: u32 }`, `pub const CONTRACT_VERSION: ContractVersion = { major: 1, minor: 0 }`. `is_compatible_with(&self, client: ContractVersion) -> bool` : vrai si les majeures sont égales et que `client.minor <= self.minor`.

## Erreurs (12.6, invariant 31)

- `ErrorCategory` : `InvalidInput`, `NotFound`, `IncompatibleState`, `Network`, `Tls`, `Authentication`, `Server`, `LocalStorage`, `Secrets`, `Cancelled`, `Internal`. Méthode `as_str()` (`"invalid_input"`, …).
- `SuggestedAction` : `Reauthenticate`, `AcceptCertificate`, `FreeDiskSpace`, `RetryLater`, `CheckServerSettings`, `UnlockKeyring`, `ChooseProvider`, `ReportBug`.
- `ErrorContext { account: Option<AccountId>, message: Option<MessageId>, mailbox: Option<MailboxId>, operation: Option<OperationId>, action: Option<ActionId>, diagnostic_id: Option<String> }` avec `Default`.
- `ErrorCode(pub &'static str)` côté construction ; sérialisé comme chaîne. Pour la désérialisation côté client, utiliser `String` : donc `MiraError.code: String`.
- `MiraError { pub category, pub code: String, pub message: String, pub retryable: bool, pub suggested_action: Option<SuggestedAction>, pub context: ErrorContext }`, `#[derive(thiserror::Error)]` avec `#[error("{code}: {message}")]`.
  - `MiraError::new(category, code: ErrorCode, message: impl Into<String>) -> Self` (retryable = vrai par défaut pour `Network`, faux sinon).
  - `retryable(self, bool) -> Self`, `with_action(self, SuggestedAction) -> Self`, `with_context(self, ErrorContext) -> Self`.
  - **Le message ne doit jamais contenir de secret** : c'est la responsabilité de l'appelant, rappelée dans la doc `///` du constructeur.
- Catalogue des codes : un fichier par catégorie dans `src/error/codes/`, chacun ne contenant que des `pub const X: ErrorCode = ErrorCode("<catégorie>.<sujet>");`, plus `src/error/codes/mod.rs` avec `pub const ALL: &[ErrorCode]` qui les liste tous (sert au test d'unicité). Codes initiaux (d'autres viendront plus tard) :
  - invalid_input : `address`, `missing_parameter`, `parameter`, `contract_version`, `cursor`, `idempotency_conflict`
  - not_found : `account`, `message`, `mailbox`, `identity`, `draft`, `action`, `operation`, `provider`, `attachment`, `thread`
  - incompatible_state : `system_mailbox`, `account_deleting`, `profile_locked`, `multiple_labels_unsupported`, `body_unavailable_offline`, `action_not_cancellable`, `account_paused`
  - network : `offline`, `timeout`, `unreachable`, `connection_lost`
  - tls : `invalid_certificate`, `handshake`, `plaintext_refused`
  - authentication : `rejected`, `token_expired`, `token_revoked`, `app_password_required`, `oauth_cancelled`, `oauth_state_mismatch`
  - server : `quota_exceeded`, `message_too_large`, `rate_limited`, `mailbox_missing`, `extension_missing`, `rejected`, `protocol`
  - local_storage : `disk_full`, `corrupted`, `migration_required`, `io`
  - secrets : `keyring_unavailable`, `keyring_locked`, `secret_missing`
  - cancelled : `by_user`
  - internal : `bug`

## Événements (12.5, invariant 32)

- `EventCategory` : `Engine`, `Account`, `Sync`, `Mailbox`, `Message`, `Thread`, `Action`, `Send`, `Alert`, `Log`.
- `EventKind` (énumération taguée), **uniquement des identifiants et des petites valeurs, jamais de contenu de message ni de secret** :
  - Engine : `EngineStarted`, `EngineStopping`, `EngineFatalError { code: String }`
  - Account : `AccountAdded { account }`, `AccountUpdated { account }`, `AccountDeleted { account }`, `AccountStateChanged { account, state: AccountState }`
  - Sync : `SyncStarted { account, operation }`, `SyncProgress { account, operation, step: SyncStep, done: u64, total: u64 }`, `SyncFinished { account, operation }`, `SyncFailed { account, operation, code: String }`, `WentOffline { account }`, `BackOnline { account }`
  - `SyncStep` : `Mailboxes`, `Headers`, `Bodies`, `Attachments`, `Changes`, `ActionsReplay`
  - Mailbox : `MailboxCreated { account, mailbox }`, `MailboxUpdated {…}`, `MailboxDeleted {…}`, `MailboxCountersUpdated { account, mailbox, total: u64, unread: u64 }`
  - Message : `NewMessageReceived { account, message, mailbox }`, `MessageUpdated { account, message }`, `MessageDeleted { account, message }`, `BodyAvailable { account, message }`, `AttachmentProgress { account, attachment, operation, done: u64, total: u64 }`, `AttachmentReady { account, attachment, operation }`
  - Thread : `ThreadUpdated { account, thread }`
  - Action : `ActionQueued { account, action }`, `ActionApplied { account, action }`, `ActionFailed { account, action, code: String, reason: String }`, `ActionCancelled { account, action }`
  - Send : `SendScheduled { account, action, at: Timestamp }`, `SendDeparted { account, action }`, `SendFailed { account, action, code: String }`, `SentCopyStored { account, action }`, `SendUncertain { account, action }`
  - Alert : `QuotaFull { account }`, `CertificateNeedsApproval { account, host: String }`, `RateLimited { account }`, `MailboxResynchronized { account, mailbox }`
  - Log : `Diagnostic { level: LogLevel, target: String, text: String }` (texte de diagnostic **sans secret ni contenu**), `LogLevel` : `Error`, `Warn`, `Info`, `Debug`
  - Opérations longues : `OperationProgress { operation, done: u64, total: u64 }`, `OperationFinished { operation }`, `OperationFailed { operation, code: String }`, `OperationCancelled { operation }` (catégorie `Engine`).
- `EventKind::category(&self) -> EventCategory` et `EventKind::account(&self) -> Option<&AccountId>` (deux `match` exhaustifs ; si la fonction dépasse 30 lignes, découper par groupes de catégories dans des fonctions auxiliaires, ou mettre ces fonctions dans des fichiers séparés).
- `Event { pub seq: EventSeq, pub at: Timestamp, pub kind: EventKind }`.
- `EventFilter { pub accounts: Vec<AccountId>, pub categories: Vec<EventCategory>, pub after: Option<EventSeq> }` (liste vide = tout) ; `matches(&self, &Event) -> bool`.

## Domaine (ch. 8, D-19)

- `AccountState` : `Connected`, `Offline`, `AuthRequired`, `Paused`, `Error { code: String }`.
- `SyncSettings { pub window_days: u32 (défaut 90, D-21), pub prefetch_bodies: u32 (défaut 50), pub synced_mailboxes: Option<Vec<MailboxId>> (None = toutes) }` + `Default`.
- `Account { id, profile: ProfileId, address: EmailAddress, display_name: String, provider: ProviderId, auth: AuthMethodKind, servers: Option<ServerSettings> (comptes « Autre », 7.9), sync: SyncSettings, state: AccountState, created_at: Timestamp }`.
- `AuthMethodKind` : `OAuth2`, `AppPassword`, `Password`. `OAuthSettings { authorize_url: String, token_url: String, scopes: Vec<String> }`. `AuthMethod` : `OAuth2(OAuthSettings)`, `AppPassword`, `Password` ; `kind() -> AuthMethodKind`.
- `SecretKind` : `Password`, `AccessToken`, `RefreshToken`. `SecretRef { profile: ProfileId, account: AccountId, kind: SecretKind }` ; `storage_key(&self) -> String` = `"mira/<profile>/<account>/<kind>"` (D-10, 7.7). **Aucune valeur de secret** dans le vocabulaire.
- `Security` : `Tls`, `StartTls`, `Plaintext` (réservé à la boucle locale, vérifié par le cœur). `ServerEndpoint { host: String, port: u16, security: Security }`. `ServerSettings { imap: ServerEndpoint, smtp: ServerEndpoint }`.
- `Identity { id, account, address: EmailAddress, display_name: String, signature: Option<String>, is_default: bool }`.
- `MailboxRole` : `Inbox`, `Sent`, `Drafts`, `Trash`, `Junk`, `Archive`, `All`, `Flagged`, `Other` ; `is_system(&self) -> bool` (vrai pour Inbox, Sent, Drafts, Trash, Junk ; 9.9).
- `SyncMarker { uid_validity: u32, uid_next: u32, highest_modseq: Option<u64> }`.
- `Mailbox { id, account, name: String (nom serveur complet), display_name: String, delimiter: Option<String>, parent: Option<MailboxId>, role: MailboxRole, sync_enabled: bool, marker: Option<SyncMarker>, total: u64, unread: u64 }`.
- `Flags { seen: bool, flagged: bool, answered: bool, forwarded: bool }` + `Default` (D-14 : portés par le message).
- `ContentState` : `HeadersOnly`, `BodyPresent`.
- `Unsubscribe { https_url: Option<String>, mailto: Option<String>, one_click: bool }`.
- `MessageSummary { id, account, thread: Option<ThreadId>, from: Vec<AddressEntry>, to, cc, bcc, reply_to: Vec<AddressEntry>, subject: String, date: Option<Timestamp>, internal_date: Option<Timestamp>, message_id_header: Option<String>, in_reply_to: Option<String>, references: Vec<String>, flags: Flags, size: u64, preview: String, has_attachments: bool, unsubscribe: Option<Unsubscribe>, content: ContentState, mailboxes: Vec<MailboxId> }`.
- `Appearance { message: MessageId, mailbox: MailboxId, uid: u32, deleted: bool, needs_revalidation: bool }` (D-14 : seul « supprimé » vit ici).
- `CacheState` : `NotDownloaded`, `Cached`. `Attachment { id, message, filename: Option<String>, mime_type: String, size: u64, content_id: Option<String>, inline: bool, cache: CacheState }`.
- `Thread { id, account, subject: String, message_count: u32, unread_count: u32, last_date: Option<Timestamp>, participants: Vec<AddressEntry> }`.
- `DraftState` : `Editing`, `Scheduled { at: Timestamp }`, `Sending`. `DraftAttachment { filename: String, mime_type: String, size: u64 }`. `Draft { id, account, identity: IdentityId, to, cc, bcc: Vec<AddressEntry>, subject: String, body_text: String, body_html: Option<String>, attachments: Vec<DraftAttachment>, in_reply_to: Option<MessageId>, state: DraftState, updated_at: Timestamp }`.
- `ActionKind` (tagué) : `MarkRead { read: bool }`, `MarkFlagged { flagged: bool }`, `Move { to: MailboxId }`, `AddToMailbox { mailbox: MailboxId }`, `RemoveFromMailbox { mailbox: MailboxId }`, `Archive`, `Trash`, `DeletePermanently`, `MarkSpam { spam: bool }`, `CreateMailbox { name: String, parent: Option<MailboxId> }`, `RenameMailbox { mailbox: MailboxId, name: String }`, `DeleteMailbox { mailbox: MailboxId }`, `EmptyMailbox { mailbox: MailboxId }`, `SaveDraft { draft: DraftId }`, `Send { draft: DraftId }`.
- `ActionState` : `Pending`, `InProgress`, `Succeeded`, `Failed`, `Cancelled`, `Uncertain` (envoi interrompu, décision 9).
- `PendingAction { id, account, kind: ActionKind, targets: Vec<MessageId>, idempotency_key: IdempotencyKey, state: ActionState, attempts: u32, last_error: Option<String>, created_at: Timestamp, not_before: Timestamp }`.
- `StandardView` : `Unified`, `Unread`, `Flagged`. `ViewRef` : `Mailbox { mailbox: MailboxId }`, `Standard { view: StandardView }`.
- `PageRequest { cursor: Option<Cursor>, limit: u32 }` avec `Default` (limit 50) et `bounded_limit(&self) -> u32` (borne entre 1 et 500, invariant 29). `Page<T> { items: Vec<T>, next_cursor: Option<Cursor> }`.

## Interface provider (ch. 6, D-05, D-07, D-08)

- `ProviderLimits { max_connections: u32, min_poll_interval_secs: u32 }` + `Default` prudent (3 connexions, 300 s).
- `MailboxModel` : `Folders`, `Labels`. `ArchiveStrategy` : `MoveToArchiveMailbox`, `RemoveFromInbox`.
- `ProviderQuirks { mailbox_model: MailboxModel, archive: ArchiveStrategy, server_saves_sent_copy: bool, unreliable_extensions: Vec<String>, extra_fetch_attributes: Vec<String> }` + `Default` (dossiers, déplacement vers l'archive, pas de copie automatique, rien d'autre).
- `ProviderDescriptor { id: ProviderId, display_name: String, domains: Vec<String>, auth_methods: Vec<AuthMethod> (ordre de préférence), limits: ProviderLimits, quirks: ProviderQuirks }`.
- `RawMailbox { name: String, delimiter: Option<String>, attributes: Vec<String> }` (attributs bruts IMAP, par exemple `\\Sent`, `\\HasNoChildren`).
- `RawMessageAttributes { items: Vec<(String, String)> }` : couples (nom d'attribut FETCH en majuscules, valeur brute), par exemple `("X-GM-MSGID", "1278455344230334865")`.
- `ServerMessageTranslation { stable_id: Option<String>, labels: Vec<String>, thread_hint: Option<String> }` + `Default`.
- Trait (signatures seulement, aucune méthode avec corps) :

```rust
pub trait Provider: std::fmt::Debug + Send + Sync {
    /// Décrit le provider (données pures)
    fn descriptor(&self) -> ProviderDescriptor;
    /// Indique si le provider gère cette adresse, d'après son domaine
    fn handles_address(&self, address: &EmailAddress) -> bool;
    /// Donne les serveurs à utiliser pour cette adresse
    fn servers_for(&self, address: &EmailAddress) -> ServerSettings;
    /// Propose un rôle pour un dossier quand le service s'écarte de la norme
    fn mailbox_role_hint(&self, mailbox: &RawMailbox) -> Option<MailboxRole>;
    /// Traduit les attributs bruts propres au service en notions du domaine
    fn translate_attributes(&self, raw: &RawMessageAttributes) -> ServerMessageTranslation;
}
```

## Tests attendus (dans `tests/`, noms explicites, ≤ 6 par fichier)

- `tests/ids.rs` : sérialisation JSON d'un `AccountId` = chaîne simple ; égalité et ordre ; `Display`.
- `tests/address.rs` : table de cas (valides, majuscules dans le domaine, espaces, deux `@`, domaine sans point, point final, Unicode).
- `tests/error_codes.rs` : `invariant_31_error_codes_are_unique_and_prefixed_by_category` (tous les codes de `ALL` sont uniques et commencent par le nom d'une catégorie) ; une `MiraError` sérialisée contient `category`, `code`, `retryable`.
- `tests/events.rs` : `EventKind::category` couvre chaque variante (un cas par catégorie) ; `EventFilter` (par compte, par catégorie, `after`) ; sérialisation JSON taguée stable.
- `tests/contract.rs` : `invariant_33_contract_version_compatibility` ; `PageRequest::bounded_limit`.
- `tests/domain.rs` : `SecretRef::storage_key` contient profil et compte ; `MailboxRole::is_system` ; valeurs par défaut de `SyncSettings` (90 jours) et `ProviderLimits`.

Chaque fichier de test d'intégration commence par `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
