# Guide Rust pour développeur C++

Ce guide aide un développeur C++ à lire le moteur MIRA. Il ne remplace pas *The Rust Programming Language* : il prend les notions que l'on croise tout le temps dans ce dépôt, les compare à leur équivalent C++, et les illustre par de **vrais extraits** du code (chemins relatifs à la racine du dépôt, lignes comprises). Si un extrait ne correspond plus à la ligne citée, le fichier a évolué depuis la rédaction : cherchez le nom de la fonction.

Plan : [1. Dépôt et build](#1-dépôt-et-build) · [2. Propriété et emprunts](#2-propriété-et-emprunts-ownership-borrowing) · [3. Durées de vie](#3-durées-de-vie-lifetimes) · [4. Erreurs](#4-erreurs--result-et-) · [5. Option](#5-option) · [6. Enums et match](#6-enums-à-données-et-match-exhaustif) · [7. Traits et génériques](#7-traits-et-génériques) · [8. Partage et concurrence](#8-partage-et-concurrence--arc-mutex-arcdyn-trait) · [9. Fermetures](#9-fermetures) · [10. async/await](#10-asyncawait-et-tokio) · [11. serde](#11-sérialisation--serde) · [12. Tests](#12-tests) · [13. Conventions du projet](#13-conventions-du-projet) · [14. Une requête de bout en bout](#14-lire-une-requête-de-bout-en-bout) · [15. Glossaire](#15-glossaire-c--rust)

## 1. Dépôt et build

| C++ | Rust | Dans MIRA |
|---|---|---|
| `CMakeLists.txt`, cibles | `Cargo.toml`, *crates* | `engine/Cargo.toml` déclare le **workspace** (`members = ["crates/*"]`) ; chaque dossier de `engine/crates/` est un crate. |
| bibliothèque / exécutable | crate `lib` / crate `bin` | `mira-core` est une bibliothèque ; `mira-cli` a une bibliothèque (`src/lib.rs`) **et** un binaire `mira` (`src/main.rs`). |
| `find_package`, vcpkg, Conan | dépendances dans `[dependencies]`, `Cargo.lock` | `[workspace.dependencies]` fixe les versions une fois ; un crate écrit `serde.workspace = true`. |
| fichier `.hpp` + `.cpp` | un seul fichier `.rs` | **Pas de fichier d'en-tête** : le compilateur lit les crates entiers, l'interface est ce qui est marqué `pub`. |
| `namespace`, `#include` | `mod`, `use` | un fichier = un module ; `pub mod x;` déclare le sous-module et va chercher `x.rs` ou `x/mod.rs`. |
| `public` / `private` | `pub` / (rien) / `pub(crate)` | par défaut **privé au module** ; `pub(crate)` = visible dans tout le crate seulement. |

Les modules du CLI se déclarent ainsi :

Extrait de `engine/crates/mira-cli/src/lib.rs:4-16` :

```rust
pub mod argument_context;
pub mod argument_error;
pub mod arguments;
pub mod civil_date;
pub mod cli_io;
pub mod commands;
pub mod deps;
pub mod exit_codes;
pub mod json_envelope;
pub mod logging;
pub mod output;
pub mod render;
pub mod usage;
```

Le compilateur et le gestionnaire de paquets sont le même outil, `cargo` :

```bash
cd engine
cargo build -p mira-cli          # compile un crate et ses dépendances
cargo test --workspace           # tous les tests
cargo test -p mira-cli --test exit_codes   # un seul fichier de tests d'intégration
cargo clippy --all-targets -- -D warnings  # linter (équivalent clang-tidy), avertissements = erreurs
cargo fmt                        # formateur (équivalent clang-format)
```

Chaque crate de provider n'a qu'une dépendance, et c'est voulu (frontière d'architecture, vérifiée par `tools/check-architecture`) :

Extrait de `engine/crates/mira-provider-gmail/Cargo.toml:13-14` :

```rust
[dependencies]
mira-vocabulary.workspace = true
```

Le crate `mira-assembly` est la *composition root* : il réexporte ce dont les clients ont besoin, si bien que `mira-cli` ne dépend ni du cœur ni d'un provider directement.

Extrait de `engine/crates/mira-assembly/src/lib.rs:4-14` :

```rust
pub mod composition;
pub mod engine_config;

pub use composition::build_registry;
pub use engine_config::production_config;
pub use mira_core::registry::{ProviderRegistry, resolve_account_provider};
pub use mira_core::{
    accounts, api, auth, capabilities, fetching, html, mailboxes, operations, ports, profile,
    reading, registry, storage, sync,
};
pub use mira_vocabulary;
```

## 2. Propriété et emprunts (ownership, borrowing)

**Idée centrale** : chaque valeur a exactement un propriétaire ; quand il sort de portée, la valeur est détruite (c'est le RAII de C++, mais systématique et vérifié à la compilation). Il n'y a ni `delete`, ni ramasse-miettes.

| C++ | Rust |
|---|---|
| passage par valeur (copie) | par défaut le passage **déplace** (*move*) : l'original n'est plus utilisable. Une copie explicite se fait par `.clone()` (ou implicitement pour les petits types `Copy`). |
| `const T&` | `&T` : emprunt en lecture, autant qu'on veut. |
| `T&` | `&mut T` : emprunt en écriture, **un seul à la fois** et aucun emprunt en lecture en même temps. |
| `unique_ptr<T>` | `Box<T>` (alloué sur le tas, propriétaire unique) |
| pointeur nul / dangling | n'existe pas en Rust « sûr » : une référence est toujours valide. |
| destructeur | trait `Drop` (rarement écrit à la main : la destruction des champs est automatique) |

Un exemple de RAII sans une ligne de destructeur : le verrou d'un profil est un fichier ouvert et verrouillé ; quand la valeur `ProfileLock` est détruite, le fichier se ferme et le verrou est libéré.

Extrait de `engine/crates/mira-core/src/profile/lock.rs:11-15` :

```rust
/// Verrou tenu tant que la valeur existe ; libéré à sa destruction.
#[derive(Debug)]
pub struct ProfileLock {
    _file: File,
}
```

Une méthode qui prend `self` (sans `&`) **consomme** l'objet : après l'appel, il n'existe plus pour l'appelant, ce que le compilateur impose. C'est exactement ce que l'on veut pour fermer le moteur :

Extrait de `engine/crates/mira-core/src/api/engine.rs:86-92` :

```rust
    /// Ferme le moteur sans attendre : les tâches de fond sont interrompues, le verrou du profil est libéré
    pub fn shutdown(self) -> Result<(), MiraError> {
        self.sync.abort_all();
        self.replayers.stop();
        self.events.emit(EventKind::EngineStopping)?;
        Ok(())
    }
```

Ici `engine.shutdown()` déplace `engine` : on ne peut plus s'en servir après (en C++ ce serait une convention, ici c'est une erreur de compilation). Notez aussi `&mut CliIo<'_>` et `&CliDeps` dans la signature suivante : le CLI **emprunte** ses flux et ses dépendances, il n'en prend pas possession.

Extrait de `engine/crates/mira-cli/src/commands/execution.rs:20-36` :

```rust
/// Ouvre le moteur du profil (synchro de fond selon la commande), déroule la commande, puis le ferme.
async fn execute_request(
    request: Request,
    command: &Commands,
    profile: &str,
    io: &mut CliIo<'_>,
    deps: &CliDeps,
) -> Result<Response, MiraError> {
    let mut config = deps.engine_config(ProfileId::from_raw(profile));
    let configured = config.network.background_sync;
    let config_mode = background_sync_for(command, configured);
    config.network.background_sync = config_mode;
    let engine = Engine::open(config, CONTRACT_VERSION)?;
    let outcome = run_steps(&engine, command, request, io).await;
    close_engine(engine, config_mode).await;
    outcome
}
```

Quand plusieurs propriétaires sont nécessaires, on partage explicitement (`Arc`, voir section 8) et l'on **clone le pointeur**, pas la donnée :

Extrait de `engine/crates/mira-cli/src/deps.rs:48-60` :

```rust
    /// Fabrique la configuration du moteur pour le profil demandé.
    pub fn engine_config(&self, profile: ProfileId) -> EngineConfig {
        EngineConfig {
            profile,
            environment: self.environment.clone(),
            registry: self.registry.clone(),
            secrets: Arc::clone(&self.secrets),
            clock: Arc::clone(&self.clock),
            ids: Arc::clone(&self.ids),
            oauth: self.oauth.clone(),
            network: self.network.clone(),
        }
    }
```

`Arc::clone(&self.secrets)` incrémente un compteur atomique (comme copier un `shared_ptr`) ; `self.registry.clone()` et `self.oauth.clone()` copient vraiment la valeur. Le type dit tout : il n'y a pas de copie cachée.

## 3. Durées de vie (lifetimes)

Une référence ne doit pas survivre à ce qu'elle désigne. Le compilateur le vérifie ; la plupart du temps il le déduit seul (*elision*). On ne voit une annotation `'a` que lorsqu'un **type stocke une référence** : elle dit « cette structure ne vit pas plus longtemps que les flux qu'elle emprunte ».

Extrait de `engine/crates/mira-cli/src/cli_io.rs:6-10` :

```rust
/// Sortie standard et sortie d'erreur d'une exécution du CLI.
pub struct CliIo<'a> {
    pub out: &'a mut dyn Write,
    pub err: &'a mut dyn Write,
}
```

`CliIo<'a>` contient deux `&'a mut dyn Write` : en C++ ce serait une struct avec deux `std::ostream&`, avec le risque de référence pendante que Rust interdit. Même idée pour l'entrée standard prêtée à l'analyse d'une commande (`ArgumentContext`) :

Extrait de `engine/crates/mira-cli/src/argument_context.rs:13-17` :

```rust
/// Entrée standard et générateur d'identifiants prêtés à la construction d'une requête.
pub struct ArgumentContext<'a> {
    pub stdin: &'a mut dyn Read,
    pub ids: &'a dyn IdGenerator,
}
```

`'_` signifie « durée de vie déduite, peu importe laquelle » (par exemple `&mut CliIo<'_>` dans les signatures). Règle pratique : si vous voyez `'a`, lisez « emprunté à quelqu'un d'autre » ; si vous voyez `'static`, la donnée est valable pour toute la durée du programme (par exemple les chaînes littérales, de type `&'static str`).

## 4. Erreurs : `Result` et `?`

Rust n'a pas d'exceptions. Une fonction qui peut échouer **le dit dans son type** : `Result<T, E>` est soit `Ok(T)` soit `Err(E)` (un `std::expected<T, E>`). L'opérateur `?` signifie « si c'est une erreur, la retourner tout de suite à mon appelant ; sinon me donner la valeur » : c'est la propagation d'une exception, mais visible et locale.

Extrait de `engine/crates/mira-core/src/api/engine.rs:44-46` :

```rust
    pub fn open(config: EngineConfig, client_version: ContractVersion) -> Result<Self, MiraError> {
        check_client_version(client_version)?;
        let (paths, lock, database) = open_profile(&config.profile, &config.environment)?;
```

Dans le projet, l'erreur est toujours la même : `MiraError`, l'erreur du contrat d'API (catégorie, code stable, message sans secret). `thiserror` génère l'affichage :

Extrait de `engine/crates/mira-vocabulary/src/error/mira_error.rs:10-20` :

```rust
/// Erreur structurée : catégorie, code stable, message sans secret, rejouable ou non.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{code}: {message}")]
pub struct MiraError {
    pub category: ErrorCategory,
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub suggested_action: Option<SuggestedAction>,
    pub context: Box<ErrorContext>,
}
```

On convertit les erreurs des bibliothèques en `MiraError` avec `map_err` (par exemple `Connection::open(path).map_err(storage_error)?` dans `engine/crates/mira-core/src/storage/database.rs:22`), et une `Option` vide en erreur avec `ok_or_else` :

Extrait de `engine/crates/mira-core/src/auth/client_config.rs:36-41` :

```rust
        let client_id = self
            .client_ids
            .get(&descriptor.id)
            .cloned()
            .ok_or_else(|| refusal("identifiant d'application OAuth absent de la configuration"))?;
        Ok((settings, client_id))
```

Le CLI transforme enfin la catégorie en code de sortie par un `match` exhaustif (section 6) :

Extrait de `engine/crates/mira-cli/src/exit_codes.rs:30-45` :

```rust
/// Donne le code de sortie d'une catégorie d'erreur.
pub fn exit_code_for(category: ErrorCategory) -> i32 {
    match category {
        ErrorCategory::InvalidInput => INVALID_INPUT,
        ErrorCategory::NotFound => NOT_FOUND,
        ErrorCategory::IncompatibleState => INCOMPATIBLE_STATE,
        ErrorCategory::Network => NETWORK,
        ErrorCategory::Tls => TLS,
        ErrorCategory::Authentication => AUTHENTICATION,
        ErrorCategory::Server => SERVER,
        ErrorCategory::LocalStorage => LOCAL_STORAGE,
        ErrorCategory::Secrets => SECRETS,
        ErrorCategory::Cancelled => CANCELLED,
        ErrorCategory::Internal => INTERNAL,
    }
}
```

`panic!` existe (il interrompt le thread, un peu comme `std::terminate` après une violation d'invariant) mais **est interdit hors tests** dans ce dépôt, comme `unwrap()` et `expect()` qui paniquent sur une erreur : `tools/check-forbidden-patterns` les cherche. On utilise `?`, `unwrap_or_else`, `ok_or_else`, `map_err`.

## 5. `Option`

Pas de pointeur nul, pas de valeur sentinelle : une valeur facultative est un `Option<T>` (`Some(valeur)` ou `None`), comme `std::optional<T>`, mais on **doit** traiter le cas vide pour accéder à la valeur. Dans le CLI, une option de ligne de commande absente est un `Option` :

Extrait de `engine/crates/mira-cli/src/commands/view_args.rs:20-28` :

```rust
#[derive(Debug, Args)]
pub struct ViewArgs {
    /// Identifiant de la boîte à lister.
    #[arg(long, conflicts_with = "view")]
    pub mailbox: Option<String>,
    /// Vue standard sur tous les comptes.
    #[arg(long, value_enum)]
    pub view: Option<StandardViewName>,
}
```

Dans `MiraError`, `suggested_action: Option<SuggestedAction>` (section 4) et dans `Page<T>`, `next_cursor: Option<Cursor>` expriment « il n'y en a pas ». Méthodes courantes : `map`, `ok_or_else` (vers `Result`), `unwrap_or_else`, `is_some()`.

## 6. Enums à données et `match` exhaustif

Un `enum` Rust est une **union étiquetée** : chaque variante peut porter des données différentes, c'est un `std::variant` avec une syntaxe de première classe. Le contrat d'API en est fait : une requête est une variante de `Request`, qui contient une variante par domaine.

Extrait de `engine/crates/mira-core/src/api/contract/request.rs:17-37` :

```rust
/// Toute requête qu'un client peut adresser au moteur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "domain", content = "request", rename_all = "snake_case")]
pub enum Request {
    Engine(EngineRequest),
    Provider(ProviderRequest),
    Account(AccountRequest),
    Identity(IdentityRequest),
    Mailbox(MailboxRequest),
    Message(MessageRequest),
    Thread(ThreadRequest),
    Search(SearchRequest),
    Maintenance(MaintenanceRequest),
    Auth(AuthRequest),
    Operation(OperationRequest),
    Event(EventRequest),
    Action(ActionRequest),
    Compose(ComposeRequest),
    Queue(QueueRequest),
    Sync(SyncRequest),
}
```

Une variante peut aussi avoir des champs nommés. Ici, « quels messages vise une action » est soit une liste, soit le résultat d'une requête :

Extrait de `engine/crates/mira-core/src/api/contract/write/target_selection.rs:8-14` :

```rust
/// Messages visés par une action ; une requête est résolue en base au moment de la soumission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TargetSelection {
    Messages { messages: Vec<MessageId> },
    Query { query: ListQuery },
}
```

`match` est l'équivalent d'un `std::visit`, mais le compilateur **refuse de compiler** si une variante n'est pas traitée. Ajouter un domaine à `Request` oblige donc à mettre à jour tous les `match` : c'est la base de l'extensibilité de ce code (un handler par opération, un `match` pour les distribuer).

Extrait de `engine/crates/mira-core/src/api/router.rs:12-32` :

```rust
/// Dirige une requête vers le routeur de son domaine
pub(crate) async fn route(engine_ref: &Engine, request: Request) -> Result<Response, MiraError> {
    match request {
        Request::Engine(request) => engine::route(engine_ref, request).await,
        Request::Provider(request) => provider::route(engine_ref, request).await,
        Request::Account(request) => account::route(engine_ref, request).await,
        Request::Identity(request) => identity::route(engine_ref, request).await,
        Request::Mailbox(request) => mailbox::route(engine_ref, request).await,
        Request::Message(request) => message::route(engine_ref, request).await,
        Request::Thread(request) => thread::route(engine_ref, request).await,
        Request::Search(request) => search::route(engine_ref, request).await,
        Request::Maintenance(request) => maintenance::route(engine_ref, request).await,
        Request::Auth(request) => auth::route(engine_ref, request).await,
        Request::Operation(request) => operation::route(engine_ref, request).await,
        Request::Event(request) => event::route(engine_ref, request).await,
        Request::Action(request) => action::route(engine_ref, request).await,
        Request::Compose(request) => compose::route(engine_ref, request).await,
        Request::Queue(request) => queue::route(engine_ref, request).await,
        Request::Sync(request) => sync::route(engine_ref, request).await,
    }
}
```

Le `match` peut aussi destructurer plusieurs valeurs à la fois et lier des variables ; ici le cas « aucune des deux options » est un `Err` :

Extrait de `engine/crates/mira-cli/src/commands/view_args.rs:36-47` :

```rust
    /// Traduit les options en référence de vue ; l'une des deux est obligatoire.
    pub fn view_ref(&self) -> Result<ViewRef, MiraError> {
        match (&self.mailbox, self.view) {
            (Some(mailbox), _) => Ok(ViewRef::Mailbox {
                mailbox: MailboxId::from_raw(mailbox.as_str()),
            }),
            (None, Some(name)) => Ok(ViewRef::Standard {
                view: standard_view(name),
            }),
            (None, None) => Err(invalid_argument("indiquer --mailbox ou --view")),
        }
    }
```

Quand on veut vraiment un cas par défaut on écrit `_`, mais le projet préfère énumérer toutes les variantes pour être alerté à l'ajout d'une nouvelle :

Extrait de `engine/crates/mira-cli/src/commands/background_policy.rs:8-28` :

```rust
/// Mode de synchro de fond du moteur pour cette commande ; `configured` est le mode des dépendances.
pub fn background_sync_for(command: &Commands, configured: BackgroundSync) -> BackgroundSync {
    match command {
        Commands::Sync(command) => sync::background_mode(command, configured),
        Commands::Account(command) => account::background_mode(command, configured),
        Commands::Engine(_)
        | Commands::Provider(_)
        | Commands::Identity(_)
        | Commands::Auth(_)
        | Commands::Operation(_)
        | Commands::Mailbox(_)
        | Commands::Message(_)
        | Commands::Thread(_)
        | Commands::Search(_)
        | Commands::Maintenance(_)
        | Commands::Action(_)
        | Commands::Draft(_)
        | Commands::Queue(_)
        | Commands::Events(_) => BackgroundSync::Off,
    }
}
```

## 7. Traits et génériques

Un **trait** décrit un comportement : c'est à la fois une classe abstraite pure (méthodes virtuelles, via `dyn Trait`) et un *concept* C++20 (contrainte sur un paramètre générique). On l'implémente pour un type avec `impl Trait for Type`, **en dehors** de la définition du type (pas d'héritage de données).

Toute la frontière « cœur / providers » est un trait. Il est dans `mira-vocabulary`, sans entrée-sortie :

Extrait de `engine/crates/mira-vocabulary/src/provider/provider.rs:11-27` :

```rust
/// Un service de messagerie connu du moteur ; seul `mira-assembly` les connaît tous.
pub trait Provider: Debug + Send + Sync {
    /// Décrit le provider (données pures).
    fn descriptor(&self) -> ProviderDescriptor;

    /// Indique si le provider gère cette adresse, d'après son domaine.
    fn handles_address(&self, address: &EmailAddress) -> bool;

    /// Donne les serveurs à utiliser pour cette adresse.
    fn servers_for(&self, address: &EmailAddress) -> ServerSettings;

    /// Propose un rôle pour une boîte quand le service s'écarte de la norme.
    fn mailbox_role_hint(&self, mailbox: &RawMailbox) -> Option<MailboxRole>;

    /// Traduit les attributs bruts propres au service en notions du domaine.
    fn translate_attributes(&self, raw: &RawMessageAttributes) -> ServerMessageTranslation;
}
```

et un provider est une structure sans état qui l'implémente :

Extrait de `engine/crates/mira-provider-gmail/src/provider.rs:13-21` :

```rust
/// Provider gmail (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct GmailProvider;

impl Provider for GmailProvider {
    /// Décrit le service : domaines, OAuth2, limites, libellés.
    fn descriptor(&self) -> ProviderDescriptor {
        descriptor()
    }
```

`Debug + Send + Sync` après `Provider:` sont des **supertraits** : tout implémenteur doit aussi savoir s'afficher en debug, et être transférable entre threads (`Send`) et partageable (`Sync`). Le compilateur déduit ces deux marqueurs : ils remplacent la documentation « cette classe est thread-safe ». Autre exemple, le port du trousseau, que les tests remplacent par une version en mémoire :

Extrait de `engine/crates/mira-core/src/ports/secrets/store.rs:7-17` :

```rust
/// Trousseau de secrets ; ne se replie jamais sur un stockage en clair (7.7).
pub trait SecretStore: Send + Sync + std::fmt::Debug {
    /// Enregistre ou remplace un secret
    fn put(&self, key: &SecretRef, value: &SecretValue) -> Result<(), MiraError>;

    /// Relit un secret, ou rien s'il n'existe pas
    fn get(&self, key: &SecretRef) -> Result<Option<SecretValue>, MiraError>;

    /// Supprime un secret ; un secret absent n'est pas une erreur
    fn delete(&self, key: &SecretRef) -> Result<(), MiraError>;
}
```

**Deux façons d'appeler un trait** :

- *statique* (génériques, `fn f<T: Trait>(x: T)` ou `impl Trait`) : le compilateur génère une version par type, comme un template (monomorphisation) ;
- *dynamique* (`dyn Trait`, derrière un pointeur : `Box<dyn T>`, `Arc<dyn T>`, `&dyn T`) : appel par table virtuelle, comme une fonction virtuelle.

Le moteur utilise `Arc<dyn Trait>` pour tout ce qui est injecté (trousseau, horloge, générateur d'identifiants), afin que les tests y mettent des fausses implémentations :

Extrait de `engine/crates/mira-cli/src/deps.rs:16-26` :

```rust
/// Tout ce que le CLI reçoit de l'extérieur ; les tests y mettent un faux trousseau.
pub struct CliDeps {
    pub environment: ProfileEnvironment,
    pub registry: ProviderRegistry,
    pub secrets: Arc<dyn SecretStore>,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    pub oauth: OAuthClientConfig,
    pub network: NetworkPolicy,
    pub stdin: Box<dyn Read + Send>,
}
```

Les **génériques** ressemblent aux templates, mais les contraintes sont explicites (`<K>`, `T: Trait`) et vérifiées à la définition, pas à l'instanciation. Les identifiants opaques du domaine sont un type générique `Id<K>` où `K` n'est qu'une étiquette de type (`PhantomData` = « je porte `K` sans en stocker »), le « *tag type* » du C++ :

Extrait de `engine/crates/mira-vocabulary/src/ids/mod.rs:11-26` :

```rust
/// Identifiant opaque d'une entité, typé par son marqueur `K` ; sérialisé comme une chaîne.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id<K> {
    value: String,
    kind: PhantomData<K>,
}

impl<K> Id<K> {
    /// Enveloppe une chaîne déjà produite (le vocabulaire ne génère jamais d'identifiant).
    pub fn from_raw(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            kind: PhantomData,
        }
    }
```

`AccountId`, `MessageId`… sont des alias de `Id<AccountMarker>`, etc. : le compilateur interdit de passer un identifiant de compte là où l'on attend un identifiant de message. `value: impl Into<String>` dit « n'importe quel type convertible en `String` ». Enfin `#[derive(Debug, Clone, PartialEq, Eq, Hash, …)]` demande au compilateur de **générer** les implémentations de ces traits (affichage de debug, copie, `operator==`, hachage) : l'équivalent d'un `= default`, mais pour tout.

## 8. Partage et concurrence : `Arc`, `Mutex`, `Arc<dyn Trait>`

| C++ | Rust |
|---|---|
| `shared_ptr<T>` | `Arc<T>` (compteur atomique, thread-safe) ; `Rc<T>` est la version non atomique. |
| `std::mutex` à côté de la donnée | `Mutex<T>` **contient** la donnée : on ne peut pas l'atteindre sans verrouiller. |
| `lock_guard` | `MutexGuard<T>` rendu par `.lock()` ; le verrou est libéré à la fin de sa portée. |
| `shared_ptr<mutex<map>>` | `Arc<Mutex<HashMap<…>>>` |
| `shared_ptr<Base>` | `Arc<dyn Trait>` |
| `std::atomic<T>` | `AtomicBool`, `AtomicI64`… |

La base de données d'un profil est derrière un `Mutex` : une seule connexion SQLite, un seul écrivain à la fois.

Extrait de `engine/crates/mira-core/src/storage/database.rs:13-17` :

```rust
/// Base d'un profil ; un seul écrivain à la fois (D-27, AG-05).
#[derive(Debug)]
pub struct Database {
    connection: Mutex<Connection>,
}
```

Et la table des budgets de connexions est partagée entre toutes les tâches du moteur :

Extrait de `engine/crates/mira-core/src/sync/session/budgets.rs:9-22` :

```rust
#[derive(Debug, Clone, Default)]
pub struct ConnectionBudgets {
    by_key: Arc<Mutex<HashMap<String, ConnectionBudget>>>,
}

impl ConnectionBudgets {
    /// Budget associé à la clé (identifiant du compte, ou adresse d'un compte pas encore ajouté).
    pub fn budget_for(&self, key: &str, limit: u32) -> ConnectionBudget {
        let mut table = self.by_key.lock().unwrap_or_else(|p| p.into_inner());
        table
            .entry(key.to_string())
            .or_insert_with(|| ConnectionBudget::new(limit))
            .clone()
    }
```

`.lock()` rend un `Result` : un `Mutex` Rust est « empoisonné » si un thread a paniqué en le tenant. Le projet n'écrit pas `unwrap()` (interdit) mais récupère les données quand même avec `unwrap_or_else(|p| p.into_inner())`.

Le compilateur empêche aussi les courses de données : on ne peut pas envoyer à un autre thread une valeur qui n'est pas `Send`, ni la partager si elle n'est pas `Sync`. Il n'y a pas d'équivalent C++ : c'est un des gains majeurs du langage.

## 9. Fermetures

Une fermeture est une lambda : `|argument| expression`. Ses captures se déduisent de l'usage (par référence, par référence mutable, ou par déplacement avec `move`). Les types de fermetures sont décrits par les traits `Fn`, `FnMut`, `FnOnce`. Le stockage les utilise pour faire un style « transaction » : l'appelant passe une fermeture, la base ouvre la transaction, l'appelle, puis la valide ou non (≈ une lambda passée à une fonction qui fait RAII autour) :

Extrait de `engine/crates/mira-core/src/storage/database.rs:58-66` :

```rust
    /// Exécute des lectures dans une transaction cohérente, jamais validée
    pub fn read<T>(
        &self,
        work: impl FnOnce(&Tx<'_>) -> Result<T, MiraError>,
    ) -> Result<T, MiraError> {
        let mut connection = self.lock();
        let inner = connection.transaction().map_err(storage_error)?;
        work(&Tx::new(inner))
    }
```

et le service de lecture l'utilise :

Extrait de `engine/crates/mira-core/src/reading/list.rs:8-22` :

```rust
/// Liste une page de messages ; les vues standard portent sur les comptes du profil donnés
pub fn list_messages(
    db: &Database,
    accounts_of_profile: &[AccountId],
    query: &ListQuery,
) -> Result<Page<MessageSummary>, MiraError> {
    db.read(|tx| {
        tx.list_view_messages(
            accounts_of_profile,
            &query.view,
            &query.filters,
            &query.page,
        )
    })
}
```

`tx` est l'argument de la lambda (`|tx| tx.list_view_messages(...)`), la transaction rendue par la base ; la valeur de retour de la fermeture devient celle de `db.read`.

## 10. `async`/`await` et tokio

`async fn` ressemble aux coroutines C++20 (`co_await`) : la fonction rend un *futur* qui ne s'exécute que quand on l'`.await`. Contrairement à C++, il faut un **exécuteur** (*runtime*) pour faire avancer les futurs : ici `tokio`. `main` crée le runtime puis y lance la fonction asynchrone du CLI :

Extrait de `engine/crates/mira-cli/src/main.rs:10-28` :

```rust
/// Point d'entrée du CLI : dépendances réelles, flux standards, exécution asynchrone.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args_os()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    init_logging(verbosity_from_args(&args));
    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    else {
        return ExitCode::from(INTERNAL as u8);
    };
    let mut deps = CliDeps::production(Box::new(std::io::stdin()));
    let mut out = std::io::stdout();
    let mut err = std::io::stderr();
    let code = runtime.block_on(run(args, &mut CliIo::new(&mut out, &mut err), &mut deps));
    let _ = out.flush();
    ExitCode::from(u8::try_from(code).unwrap_or(INTERNAL as u8))
}
```

`run` est une `async fn` qui attend d'autres :

Extrait de `engine/crates/mira-cli/src/lib.rs:23-29` :

```rust
/// Exécute le CLI avec ses arguments (nom du programme compris) et rend le code de sortie.
pub async fn run(args: Vec<String>, io: &mut CliIo<'_>, deps: &mut CliDeps) -> i32 {
    match arguments::Cli::try_parse_from(&args) {
        Ok(cli) => commands::dispatch(cli, io, deps).await,
        Err(error) => usage::report_usage_error(&error, &args, io),
    }
}
```

Les opérations longues (synchronisation, téléchargements) sont lancées en tâche de fond avec `tokio::spawn` (≈ `std::async`/`std::jthread`, mais en tâche légère). Le mot-clé `move` donne à la tâche la propriété de ce qu'elle utilise :

Extrait de `engine/crates/mira-core/src/sync/window/spawn.rs:29-45` :

```rust
/// Démarre l'extension en arrière-plan et rend l'identifiant de l'opération (annulable via le registre).
pub fn start_extend_window(
    ctx: SyncContext,
    budget: ConnectionBudget,
    credentials: Arc<dyn CredentialProvider>,
    operations: &OperationRegistry,
    mailbox: MailboxId,
    extra_days: u32,
) -> OperationId {
    let handle = operations.start("extend_window");
    let id = handle.id().clone();
    tokio::spawn(async move {
        let outcome = run_extension(ctx, budget, credentials, &handle, mailbox, extra_days).await;
        handle.complete(outcome.as_ref().map(|_| ()));
    });
    id
}
```

Deux règles utiles : (1) on n'appelle pas de code bloquant long depuis une tâche asynchrone (le projet utilise `spawn_blocking` pour cela) ; (2) un `Mutex` standard ne doit pas être tenu à travers un `.await`. Les tests asynchrones s'écrivent avec `#[tokio::test]` (section 12).

## 11. Sérialisation : serde

`serde` génère, à la compilation et par `#[derive(Serialize, Deserialize)]`, le code de (dé)sérialisation JSON d'un type : pas de macro à écrire, pas de visiteur, pas de réflexion à l'exécution. Les attributs `#[serde(...)]` règlent le format. Le contrat d'API **est** ces types sérialisés :

Extrait de `engine/crates/mira-vocabulary/src/error/category.rs:5-9` :

```rust
/// Famille d'erreur ; le client choisit son comportement d'après elle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    InvalidInput,
```

Un enum à données se sérialise avec une étiquette (`tag`) :

Extrait de `engine/crates/mira-vocabulary/src/domain/view.rs:16-22` :

```rust
/// Référence à ce que l'on veut lister.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ViewRef {
    Mailbox { mailbox: MailboxId },
    Standard { view: StandardView },
}
```

ce qui donne `{"type":"mailbox","mailbox":"…"}` ou `{"type":"standard","view":"…"}`. C'est aussi ce qu'on voit sur la sortie `--json` du CLI : `{"data":{"data":[…],"type":"accounts"},…}`. L'enveloppe `{contract_version, ok, data|error}` est fabriquée dans `engine/crates/mira-cli/src/json_envelope.rs`. Le type `Id<K>` de la section 7 utilise `#[serde(transparent)]` : il se sérialise comme la simple chaîne qu'il contient.

## 12. Tests

Il n'y a pas de framework de test à installer : `#[test]` est dans le langage, `cargo test` compile et exécute tout.

- Les **tests d'intégration** sont des fichiers de `crate/tests/`. Chaque fichier est compilé comme un crate séparé qui n'utilise que l'API **publique** (c'est ce que la règle « tests dans des fichiers séparés » du projet exploite).
- Les tests unitaires peuvent aussi vivre dans le crate sous `#[cfg(test)]` ; dans ce projet la plupart des tests sont dans `tests/`, séparés des sources.
- `assert!`, `assert_eq!` : une assertion qui échoue **panique**, ce qui fait échouer le test. C'est la seule situation où `panic`/`unwrap` sont tolérés ; en tête de fichier de test on voit donc `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.

Extrait de `engine/crates/mira-cli/tests/exit_codes.rs:33-40` :

```rust
/// Une sous-commande inconnue est une entrée invalide
#[test]
fn unknown_subcommand_is_invalid_input() {
    let result = run_cli(&["inconnue"]);
    assert_eq!(result.code, 10);
    assert!(result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
}
```

Un test asynchrone contre les faux serveurs du testkit (sockets sur `127.0.0.1`) :

Extrait de `engine/crates/mira-cli/tests/scenario_s02_add_app_password_account.rs:14-22` :

```rust
#[tokio::test]
async fn scenario_s2_yahoo_detection_offers_only_the_app_password_via_cli() {
    let world = yahoo_world().await;
    let detected = world.run_ok(&["provider", "detect", USER]).await;
    assert_eq!(detected["provider"]["id"], "yahoo");
    assert_eq!(
        detected["provider"]["auth_methods"],
        serde_json::json!([{"type": "app_password"}])
    );
```

Le **test par propriétés** (`proptest`) génère des entrées aléatoires, vérifie un invariant, et réduit tout contre-exemple à un cas minimal (≈ RapidCheck en C++) :

Extrait de `engine/crates/mira-core/tests/proptest_invariants_flags.rs:27-38` :

```rust
proptest! {
    #![proptest_config(quick_config())]

    /// Invariant 12 : quelles que soient les opérations, le message montre les mêmes drapeaux dans toutes ses boîtes
    #[test]
    fn invariant_12_flags_are_the_same_in_every_mailbox(operations in store_ops()) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
            check_flags_live_on_message(&world);
        }
    }
```

Le nom des tests cite l'invariant ou le scénario qu'il couvre (`invariant_12_…`, `scenario_s2_…`) : `docs/ETAT/INVARIANTS.md` fait le lien.

## 13. Conventions du projet

Elles sont vérifiées mécaniquement par `tools/check-norms` (dans `./ci.sh`) ; un dépassement fait échouer la CI.

| Règle | Détail |
|---|---|
| Taille d'une fonction | **≤ 30 lignes**, de `fn` à l'accolade fermante. |
| Taille d'un fichier | **≤ 6 fonctions** avec corps par fichier (tests compris). D'où beaucoup de petits fichiers et de petits dossiers par domaine. |
| Documentation | **Exactement une ligne `///`** (en français : l'intention, pas l'implémentation) avant chaque `fn`. Un `//!` de 1 à 3 lignes en tête de fichier est permis. |
| Commentaires | **Aucun commentaire dans le corps d'une fonction** ; les justifications vont dans `docs/DECISIONS-AGENT.md`. |
| Noms | explicites, en anglais pour les identifiants ; pas de `utils.rs`, `helpers.rs`, `common.rs`, `misc.rs`. |
| Structure | arborescence métier (comptes, boîtes, messages, synchro, file d'actions, stockage, secrets, providers, api, cli) ; un handler par opération ; routage par `match` exhaustif. |
| Interdits hors tests | `unwrap`, `expect`, `panic!`, `todo!`, `dbg!`, `unsafe` (`tools/check-forbidden-patterns`). |
| Secrets | jamais dans les journaux, erreurs, événements ni réponses ; le `Debug` des types qui en portent est masqué. |

Voici une fonction conforme (une ligne `///`, corps court, aucun commentaire) :

Extrait de `engine/crates/mira-cli/src/commands/message/list.rs:19-23` :

```rust
/// Construit la requête de liste de messages.
pub fn request(args: &ListArgs) -> Result<MessageRequest, MiraError> {
    let query = args.selection.list_query(args.page.page_request())?;
    Ok(MessageRequest::ListMessages(query))
}
```

Autres conventions : `cargo fmt` fixe la mise en forme (`engine/rustfmt.toml`), `clippy -D warnings` est exigé, et `tools/check-architecture` vérifie la frontière (le cœur ne nomme aucun provider, les providers ne dépendent que de `mira-vocabulary`, aucune bibliothèque externe en dehors de son module d'encapsulation).

## 14. Lire une requête de bout en bout

Suivons `mira message list --mailbox <boîte> --limit 20`, une **lecture locale**, de la ligne de commande jusqu'à SQLite. Chaque étape cite l'extrait réel.

**1. `main` : runtime, dépendances de production, appel de `run`** — voir l'extrait `main.rs` de la section 10. `CliDeps::production` branche le vrai trousseau et la vraie horloge (le trait de la section 7) ; les tests appellent `run` directement avec de fausses dépendances.

**2. `run` : analyse des arguments.** `Cli::try_parse_from` (`clap`) transforme les arguments en structures grâce aux `#[derive(Parser/Subcommand/Args)]`. Les erreurs d'usage deviennent une `MiraError` « entrée invalide » (code 10). Les options globales et le domaine :

Extrait de `engine/crates/mira-cli/src/arguments.rs:22-41` :

```rust
/// Client en ligne de commande de MIRA.
#[derive(Debug, Parser)]
#[command(
    name = "mira",
    version,
    about = "Client mail MIRA en ligne de commande"
)]
pub struct Cli {
    /// Écrit le résultat sous forme d'enveloppe JSON versionnée.
    #[arg(long, global = true)]
    pub json: bool,
    /// Nom du profil (dossier de données) à utiliser.
    #[arg(long, global = true, default_value = "default")]
    pub profile: String,
    /// Journalisation plus bavarde sur stderr (`-v` info, `-vv` debug).
    #[arg(short = 'v', long = "verbose", global = true, action = ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Commands,
}
```

**3. `dispatch` : aiguillage par domaine.** Le catalogue de providers est local ; tout le reste passe par le moteur :

Extrait de `engine/crates/mira-cli/src/commands/mod.rs:37-44` :

```rust
/// Aiguille la commande analysée : catalogue local pour `provider`, moteur pour le reste.
pub async fn dispatch(cli: Cli, io: &mut CliIo<'_>, deps: &mut CliDeps) -> i32 {
    let mode = OutputMode::from_json_flag(cli.json);
    match cli.command {
        Commands::Provider(command) => provider::run_provider(command, &deps.registry, mode, io),
        other => execution::run_command(&other, &cli.profile, mode, io, deps).await,
    }
}
```

**4. De la commande à la requête du contrat.** `build_request` fait un `match` exhaustif sur les domaines :

Extrait de `engine/crates/mira-cli/src/commands/request_builder.rs:13-36` :

```rust
/// Traduit une commande analysée en requête du contrat (lit stdin si elle porte un secret).
pub fn build_request(
    command: &Commands,
    context: &mut ArgumentContext,
) -> Result<Request, MiraError> {
    match command {
        Commands::Engine(command) => Ok(engine::build_request(command)),
        Commands::Provider(command) => Ok(provider::build_request(command)),
        Commands::Account(command) => account::build_request(command, context),
        Commands::Identity(command) => Ok(identity::build_request(command)),
        Commands::Auth(command) => auth::build_request(command, context),
        Commands::Operation(command) => Ok(operation::build_request(command)),
        Commands::Mailbox(command) => mailbox::build_request(command, context),
        Commands::Message(command) => message::build_request(command),
        Commands::Thread(command) => Ok(thread::build_request(command)),
        Commands::Search(command) => search::build_request(command),
        Commands::Maintenance(command) => Ok(maintenance::build_request(command)),
        Commands::Action(command) => action::build_request(command, context),
        Commands::Draft(command) => draft::build_request(command, context),
        Commands::Queue(command) => Ok(queue::build_request(command)),
        Commands::Sync(command) => Ok(sync::build_request(command)),
        Commands::Events(args) => Ok(events::build_request(args)),
    }
}
```

puis le domaine `message` traduit chaque sous-commande en une variante de `MessageRequest` :

Extrait de `engine/crates/mira-cli/src/commands/message/mod.rs:46-49` :

```rust
/// Traduit une commande `message` en requête du contrat.
pub fn build_request(command: &MessageCommand) -> Result<Request, MiraError> {
    let request = match command {
        MessageCommand::List(args) => list::request(args)?,
```

et `message list` construit sa requête (vue, filtres, page) :

Extrait de `engine/crates/mira-cli/src/commands/message/list.rs:19-23` :

```rust
/// Construit la requête de liste de messages.
pub fn request(args: &ListArgs) -> Result<MessageRequest, MiraError> {
    let query = args.selection.list_query(args.page.page_request())?;
    Ok(MessageRequest::ListMessages(query))
}
```

À ce stade, il n'y a plus rien de propre au CLI : on a un `Request::Message(MessageRequest::ListMessages(…))`, une valeur sérialisable, que n'importe quel client (une interface graphique plus tard) pourrait construire.

**5. Ouverture du moteur et exécution.** `execute_request` (extrait de la section 2) ouvre le moteur sur le profil, déroule la commande avec `run_steps` (qui appelle `engine.execute`, voir `waiting::execute_and_wait`), puis referme le moteur.

Extrait de `engine/crates/mira-core/src/api/engine.rs:71-74` :

```rust
    /// Exécute une requête du contrat
    pub async fn execute(&self, request: Request) -> Result<Response, MiraError> {
        route(self, request).await
    }
```

**6. Routage vers le handler.** Un `match` par domaine (le routeur de la section 6, `api/router.rs`), puis un `match` par opération :

Extrait de `engine/crates/mira-core/src/api/handlers/message/mod.rs:27-30` :

```rust
/// Route une requête du domaine messages
pub(crate) async fn route(engine: &Engine, request: MessageRequest) -> Result<Response, MiraError> {
    match request {
        MessageRequest::ListMessages(query) => handle_list_messages(engine, &query),
```

**7. Handler** (un fichier par opération, dans `mira-core/src/api/handlers/<domaine>/`) : il traduit la requête du contrat en appel de service et encapsule le résultat dans une `Response` :

Extrait de `engine/crates/mira-core/src/api/handlers/message/list.rs:9-17` :

```rust
/// Rend une page de messages de la vue demandée, sur les comptes du profil
pub(crate) fn handle_list_messages(
    engine: &Engine,
    query: &ListQuery,
) -> Result<Response, MiraError> {
    let accounts = engine.account_ids()?;
    let page = list_messages(&engine.database, &accounts, query)?;
    Ok(Response::Messages(page))
}
```

**8. Service de lecture** (module `reading`) : une transaction de lecture sur la base, via la fermeture de la section 9 (`list_messages` de `mira-core/src/reading/list.rs`, déjà citée) : elle passe la vue, les filtres et la page à `Tx::list_view_messages`.

**9. Stockage** : la requête SQL est construite et exécutée sur le miroir local (`rusqlite`, SQLite embarquée). La vue « boîte » ou « standard » choisit le constructeur de requête, les filtres s'y ajoutent :

Extrait de `engine/crates/mira-core/src/storage/repositories/views/list.rs:15-30` :

```rust
impl Tx<'_> {
    /// Liste une page de messages d'une vue, filtrés, du plus récent au plus ancien
    pub fn list_view_messages(
        &self,
        accounts: &[AccountId],
        view: &ViewRef,
        filters: &MessageFilters,
        request: &PageRequest,
    ) -> Result<Page<MessageSummary>, MiraError> {
        let mut builder = match view {
            ViewRef::Mailbox { mailbox } => self.mailbox_builder(mailbox)?,
            ViewRef::Standard { view } => standard_builder(accounts, *view),
        };
        apply_filters(&mut builder, filters);
        self.run_message_page(builder, request)
    }
```

**10. Retour.** `Response::Messages(page)` remonte jusqu'à `report_response`, qui sérialise la réponse en JSON ou la rend en texte lisible, puis `report_success` l'écrit et `run` rend le code 0. Une erreur (`MiraError`) prend le même chemin en sens inverse jusqu'à `exit_code_for` (section 4).

Extrait de `engine/crates/mira-cli/src/commands/execution.rs:50-56` :

```rust
/// Écrit la réponse : le contrat lui-même en JSON, un texte lisible sinon.
fn report_response(response: &Response, mode: OutputMode, io: &mut CliIo) -> i32 {
    match to_json_value(response) {
        Ok(data) => report_success(io, mode, data, &human_text(response)),
        Err(error) => report_error(io, mode, &error),
    }
}
```

Aucun réseau n'a été touché : une lecture ne parle qu'à la base locale (invariant 27 : lire et lister fonctionne hors ligne). Pour une **écriture** (`mira action apply …`), le chemin est le même jusqu'au handler, qui enregistre alors une action dans la file locale (`actions`) et rend tout de suite son identifiant ; le rejeu vers le serveur est asynchrone (voir `docs/ARCHITECTURE-IMPLEMENTEE.md`).

Pour retrouver le code d'une commande quelconque : `engine/crates/mira-cli/src/commands/<domaine>/` côté CLI, `engine/crates/mira-core/src/api/handlers/<domaine>/` côté moteur, et `rg "NomDeLaVariante"` pour suivre une variante de `Request`.

## 15. Glossaire C++ ↔ Rust

| C++ | Rust |
|---|---|
| `#include` / `namespace` | `use` / `mod` |
| `CMakeLists.txt` | `Cargo.toml` |
| `const T&` / `T&` | `&T` / `&mut T` |
| `std::move` | déplacement implicite par défaut |
| copie | `.clone()` (trait `Clone`) |
| `unique_ptr<T>` | `Box<T>` |
| `shared_ptr<T>` | `Arc<T>` (ou `Rc<T>` hors threads) |
| `std::mutex` + donnée | `Mutex<T>` |
| `std::optional<T>` | `Option<T>` |
| `std::expected<T, E>` / exceptions | `Result<T, E>`, opérateur `?` |
| `std::variant` + `std::visit` | `enum` à données + `match` |
| `std::string` / `std::string_view` | `String` / `&str` |
| `std::vector<T>` | `Vec<T>` |
| `std::map` / `std::unordered_map` | `BTreeMap` / `HashMap` |
| classe abstraite, `virtual` | `trait`, `dyn Trait` |
| concept C++20, `template<…>` | génériques `<T: Trait>`, `impl Trait` |
| destructeur | `Drop` (automatique pour les champs) |
| lambda | fermeture `\|x\| …` |
| `co_await` / coroutine | `async fn` / `.await` + runtime (tokio) |
| `std::thread` / `std::async` | `std::thread::spawn` / `tokio::spawn` |
| `= default` pour copie, `==`, hachage | `#[derive(Clone, PartialEq, Hash, …)]` |
| `operator<<` | trait `Display` (affichage) / `Debug` (debug) |
| `static_assert`, tests unitaires | `#[test]`, `assert!`, `assert_eq!` |
| sanitizers, valgrind | inutiles pour la mémoire en Rust sûr ; le projet interdit `unsafe` |
| clang-tidy / clang-format | `cargo clippy` / `cargo fmt` |
| `nullptr` | pas d'équivalent : `Option::None` |
| `[[nodiscard]]` | un `Result` non utilisé est un avertissement du compilateur |
| tag dispatch / type étiquette | `PhantomData<K>` (`Id<K>`) |

Pour aller plus loin : `README.md` (construire et utiliser le CLI), `docs/ARCHITECTURE-IMPLEMENTEE.md` (organisation du moteur), `docs/DECISIONS-AGENT.md` (pourquoi les choix), `docs/specs/` (spécifications par jalon).
