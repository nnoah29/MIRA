# Démon, phase 2 : protocole et démon — Plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Un binaire `mira-daemon` qui héberge le moteur (via `mira-host`) et expose le contrat JSON sur une socket Unix ou un TCP loopback, avec un protocole de trames défini dans `mira-protocol`, et un seul jeu de tests de contrat qui passe sur le chemin embarqué et sur le chemin socket.

**Architecture:** `mira-protocol` est pur (types de trames, codec, règles de poignée de main). `mira-daemon` (bibliothèque + binaire) ajoute les transports (`Listener` → flux d'octets + `Trust`), le jeton local, une session par connexion (lecture des trames, une tâche par appel, un registre de flux avec crédits et annulation) et l'arrêt propre. `mira-host` gagne des variantes à émission asynchrone pour que la contre-pression d'un flux de chat suspende vraiment la génération. Les scénarios de contrat sont écrits une fois contre un trait `Driver` à trois implémentations : hôte direct, socket, pont FFI.

**Tech Stack:** Rust 1.99 (édition 2024), tokio (net, io-util, sync, time, + `signal` pour le binaire), serde/serde_json, thiserror, clap, ring (jeton, comparaison en temps constant), `mira-host`, `mira-assembly`, `mira-testkit` (dev).

**Spec:** `docs/superpowers/specs/2026-11-04-architecture-demon-design.md` (§2 crates, §3.1 transports Rust, §4 protocole, §5 possession du moteur, §6 sécurité, §9 tests, §10 phase 2).

**Branche :** `feat/E-01-protocole-demon`, créée depuis `feat/E-01-host-commun` (la phase 1 n'est pas encore intégrée dans `dev` ; la phase 2 en dépend).

## Global Constraints

- Le contrat JSON, les enveloppes et `CONTRACT_VERSION` ne changent pas. Seul le transport est nouveau. `bridge/src/frb_generated.rs`, `bridge/src/api/*` et `app/` ne changent pas.
- Trames : JSON préfixé par la longueur sur 4 octets grand-boutiste ; la taille annoncée est comparée au maximum du serveur **avant** toute lecture ou allocation du corps ; `MAX_FRAME_BYTES = 16 * 1024 * 1024` par défaut. Chaque trame a un champ `type` ; les échanges ont un `id` u64 choisi par le client.
- Poignée de main : refus si la version majeure du protocole diffère ou si `CONTRACT_VERSION.is_compatible_with(client)` est faux (règle de `mira-vocabulary`, `major` égal et `minor` client ≤ serveur ; aucune autre règle). Le refus est une trame `hello_err` à raison fixe, sans texte libre, puis fermeture.
- Authentification : le jeton local (256 bits, hex, fichier `0600`, comparé en temps constant, jamais journalisé ni renvoyé) est exigé dans `hello` sur **tous** les transports locaux ; la socket Unix y ajoute `0600`, dossier `0700` et vérification de l'uid du pair. Le démon n'écoute que sur `127.0.0.1` pour TCP.
- Une déconnexion n'annule jamais un envoi ni une synchronisation : les tâches d'appel (`call`) ne sont jamais interrompues par la fermeture de la connexion ; seuls les flux (`open_events`, `open_chat`) le sont.
- Aucun secret dans les trames **émises par le démon**, ni dans les journaux et erreurs (messages fixes). Les entrées secrètes que le contrat définit déjà (mot de passe de `AddAccount`, clé de `SetAiApiKey`) restent permises dans les `call` entrants sur ces transports locaux.
- Normes (`tools/check-norms`, tests compris) : fonction ≤ 30 lignes, fichier ≤ 6 fonctions avec corps, aucun commentaire dans un corps, une seule ligne `///` en français avant chaque `fn`, `//!` de 1 à 3 lignes en tête de fichier, pas de fichier `utils`/`helpers`/`common`/`misc`. Une macro de test compte une seule fois ses `fn` écrites.
- Pas de `unsafe`, pas de `unwrap`/`expect`/`panic!` hors tests. Pas de named pipe ni de code propre à Windows (phase 5) ; le code Unix est derrière `cfg(unix)` et le crate doit rester compilable sous `cfg(not(unix))` sans les transports Unix.
- Tests ciblés (`cargo test -p <crate> --test <nom>`). **Ne jamais lancer `./ci.sh` ni la suite moteur complète sans l'accord explicite de l'utilisateur.**
- Commits : un par tâche, `feat(protocol): …`, `feat(daemon): …`, `test(contract): …` ou `docs(daemon): …`, **sans trailer `Co-Authored-By`**, fichiers nommés dans `git add`.

## Review Focus

- Trame annoncée à 4 Gio ou plus grande que le maximum : rejetée sans lire ni allouer le corps → tâche 4.
- Client lent sur un flux d'événements : `end{lagged}` puis réabonnement avec `after` sans perte ni doublon → tâche 7.
- Flux de chat sans crédit : la génération est suspendue (le fournisseur n'est plus lu), pas abandonnée ; reprise au crédit suivant → tâche 7.
- Connexion fermée pendant un envoi de message : l'envoi aboutit quand même → tâche 8.
- Deux démons sur le même profil : le second échoue proprement, le premier répond toujours ; socket laissée par un démon tué (`kill -9`) remplacée au redémarrage → tâches 5 et 9.
- Connexion muette ou qui envoie des octets à vitesse d'escargot avant `hello` : fermée au délai ; aucun jeton ni secret dans les trames sortantes d'un scénario complet → tâches 6 et 11.

---

### Task 1: Les tests de `file_secrets` et `polling` rejoignent `mira-host`

Solde le mineur différé de la phase 1 : ces tests testent du code de `mira-host` mais vivent dans `bridge/tests`, que `./ci.sh` ne lance pas.

**Files:**
- Move: `bridge/tests/file_secrets.rs` → `engine/crates/mira-host/tests/file_secrets.rs` ; `bridge/tests/polling.rs` → `engine/crates/mira-host/tests/polling.rs`
- Modify: les imports (`mira_bridge::file_secrets` → `mira_host::file_secrets`, `mira_bridge::polling` → `mira_host::polling`), `engine/crates/mira-host/Cargo.toml` (dev-dependencies utiles)

**Interfaces:** aucune nouvelle.

- [ ] **Step 1:** `git mv` des deux fichiers, imports mis à jour, aucune assertion modifiée. Lancer `cd engine && cargo test -p mira-host --test file_secrets --test polling`. Expected: tous les tests passent (les mêmes noms qu'avant).
- [ ] **Step 2:** `tools/check-norms engine/crates/mira-host`. Si `file_secrets.rs` dépasse 6 fonctions, le scinder par thème (`file_secrets_roundtrip.rs`, `file_secrets_opacity.rs`) sans toucher aux assertions. Expected: 0 violation.
- [ ] **Step 3:** Dans `bridge/` : `cargo test --no-run` doit encore compiler les tests restants. Expected: succès.
- [ ] **Step 4: Commit** — `test(host): les tests du trousseau chiffré et du relevé rejoignent mira-host`.

---

### Task 2: Crate `mira-protocol`

**Files:**
- Create: `engine/crates/mira-protocol/Cargo.toml`, `src/lib.rs`, `src/version.rs`, `src/auth.rs`, `src/frame.rs`, `src/codec.rs`, `src/handshake.rs`
- Create (tests): `tests/frames.rs`, `tests/codec.rs`, `tests/handshake.rs`
- Modify: `engine/Cargo.toml` (`mira-protocol` dans `[workspace.dependencies]`), `tools/check-architecture` (règle A11), `tools/test-check-architecture`

**Interfaces:**
- Produces :
  - `ProtocolVersion { major: u16, minor: u16 }` et `PROTOCOL_VERSION` (1.0)
  - `Auth` : `#[serde(tag = "type")] enum { LocalToken { token: String } }` (serde `local_token`)
  - `HostState` : `Ready | Starting | KeyringLocked | UpdateRequired` (serde `snake_case`)
  - `HelloRefusal` : `ProtocolVersion | ContractVersion | Auth | Malformed` (serde `snake_case`, aucun texte)
  - `EndReason` : `#[serde(tag = "type")] Finished | Cancelled | Lagged | Error { code: String, message: String }`
  - `HostOp` : `#[serde(rename_all = "snake_case")] Shutdown`
  - `Frame` : `#[serde(tag = "type", rename_all = "snake_case")]` avec les variantes ci-dessous
  - `MAX_FRAME_BYTES: u32`, `CodecError { TooLarge{announced: u32, max: u32}, Malformed, Empty }` (thiserror)
  - `encode(frame: &Frame, max: u32) -> Result<Vec<u8>, CodecError>` (préfixe + JSON) ; `body_len(header: [u8; 4], max: u32) -> Result<usize, CodecError>` ; `decode(body: &[u8]) -> Result<Frame, CodecError>`
  - `check_hello(protocol: ProtocolVersion, contract: ContractVersion, server_contract: ContractVersion) -> Result<(), HelloRefusal>`
- Dépend de `mira-vocabulary` (pour `ContractVersion`), `serde`, `serde_json`, `thiserror` : rien d'autre (règle A11).

Forme des trames (le client Dart réimplémentera exactement ceci) :

| `type` | Champs en plus de `type` |
|---|---|
| `hello` | `protocol_version`, `contract_version`, `client_id` (String), `client_name` (String), `auth` |
| `hello_ok` | `protocol_version`, `contract_version`, `host_state` |
| `hello_err` | `reason` (`HelloRefusal`) |
| `call` | `id`, `request` (JSON du contrat, objet) |
| `result` | `id`, `envelope` (enveloppe du contrat `{contract_version, ok, data \| error}`) |
| `open_events` | `id`, `after` (u64 ou null), `credit` (u32 : fenêtre initiale) |
| `open_chat` | `id`, `request` (`ChatStart`), `credit` |
| `get_original_html` | `id`, `message` (String) |
| `host_op` | `id`, `op` (`HostOp`) ; la réponse est un `result` |
| `event` / `chat_delta` | `id`, `event` / `line` (JSON) |
| `credit` | `id`, `n` (u32, s'ajoute à la fenêtre) |
| `cancel` | `id` |
| `end` | `id`, `reason` (`EndReason`) |
| `ping` / `pong` | `nonce` (u64) |

- [ ] **Step 1: Règle A11 et son auto-test.** `tools/check-architecture` : `mira-protocol` ne dépend (normal) que de `mira-vocabulary`, `serde`, `serde_json`, `thiserror`, et d'aucun autre `mira-*` même en dev. Auto-test : `sabotage; echo 'tokio.workspace = true' >> .../mira-protocol/Cargo.toml; expect A11`. Lancer `tools/test-check-architecture`. Expected: ÉCHEC (crate absent), c'est le rouge.
- [ ] **Step 2: Tests `frames.rs`.** `every_frame_round_trips_through_json` (une instance de chaque variante : `serde_json::to_value` puis retour égal) ; `the_type_tag_uses_the_documented_names` (les 15 noms du tableau, dont `hello_ok`, `hello_err`, `get_original_html`, `host_op`) ; `hello_err_carries_only_a_fixed_reason` (le JSON n'a que `type` et `reason`) ; `an_unknown_type_is_rejected`.
- [ ] **Step 3: Tests `codec.rs`.** `encode_prefixes_the_body_length_big_endian` ; `decode_returns_the_encoded_frame` ; `an_announced_size_above_the_maximum_is_refused_from_the_header_alone` (`body_len([0xFF; 4], 1024)` → `TooLarge`, sans corps) ; `encode_refuses_a_frame_above_the_maximum` ; `a_zero_length_body_is_empty_error` ; `garbage_and_truncated_json_are_malformed`.
- [ ] **Step 4: Tests `handshake.rs`.** `same_major_and_lower_or_equal_minor_is_accepted` ; `a_newer_client_minor_is_refused_as_contract_version` ; `a_different_contract_major_is_refused_as_contract_version` ; `a_different_protocol_major_is_refused_as_protocol_version` (le protocole est testé avant le contrat) ; `a_newer_protocol_minor_is_accepted`.
- [ ] **Step 5: Lancer** `cd engine && cargo test -p mira-protocol` : FAIL (crate absent).
- [ ] **Step 6: Implémenter** selon les Interfaces. `check_hello` réutilise `ContractVersion::is_compatible_with`. Un fichier par responsabilité (≤ 6 fonctions chacun).
- [ ] **Step 7: Lancer** `cargo test -p mira-protocol`, `tools/test-check-architecture`, `tools/check-architecture engine`, `cargo clippy -p mira-protocol --all-targets -- -D warnings`, `cargo fmt --all --check`, `tools/check-norms engine/crates/mira-protocol`. Expected: tout PASS.
- [ ] **Step 8: Commit** — `feat(protocol): trames, codec et règles de poignée de main (mira-protocol)`.

---

### Task 3: `mira-host` : émission asynchrone des flux

La contre-pression du chat suspend la lecture du fournisseur ; l'émetteur synchrone actuel ne peut pas attendre un crédit.

**Files:**
- Modify: `engine/crates/mira-host/src/chat_feed.rs`, `events_feed.rs`, `streams.rs`
- Test: `engine/crates/mira-host/tests/host_streams_async.rs`

**Interfaces:**
- Consumes: `Host::engine()`, `chat_feed::pump_chat`, `events_feed::pump_events` (phase 1).
- Produces :
  - `Host::pump_events_async(&self, after: Option<u64>, emit: impl AsyncFnMut(String) -> bool)`
  - `Host::pump_chat_async(&self, request_json: &str, emit: impl AsyncFnMut(String) -> bool)`
  - `Host::pump_events` et `Host::pump_chat` (synchrones) gardent leur signature et délèguent aux variantes asynchrones (`async |line| emit(line)`).

- [ ] **Step 1: Écrire `host_streams_async.rs`.** `async_events_on_a_closed_host_emit_nothing` ; `async_chat_on_a_closed_host_emits_one_fixed_error_line` (mêmes assertions que `host_streams.rs`, texte de la demande absent de la ligne). Lancer : FAIL (méthodes absentes).
- [ ] **Step 2: Implémenter** les variantes asynchrones (le corps des fonctions actuelles, avec `emit(line).await`) ; les fonctions synchrones deviennent des enveloppes. Garder `closing_line` et le moment où le moteur est pris (`&Host`, jamais un `Arc` tenu pendant le flux).
- [ ] **Step 3: Lancer** `cargo test -p mira-host` (les 7 tests de la phase 1 + les 2 nouveaux), puis dans `bridge/` : `cargo test --test chat_stream --test chat_stream_ending --test chat_stream_errors --test chat_stream_masked --test chat_stream_tools --test chat_stream_draft`. Expected: tout PASS sans modifier ces tests. Puis clippy, fmt, `tools/check-norms engine/crates/mira-host`.
- [ ] **Step 4: Commit** — `feat(host): variantes asynchrones des flux d'événements et de chat`.

---

### Task 4: `mira-daemon` : squelette, lecture/écriture de trames, jeton

**Files:**
- Create: `engine/crates/mira-daemon/Cargo.toml` (lib `mira_daemon` + bin `mira-daemon`), `src/lib.rs`, `src/main.rs` (provisoire : `fn main() {}` jusqu'à la tâche 9), `src/wire.rs`, `src/token.rs`
- Create (tests): `tests/wire.rs`, `tests/token.rs`
- Modify: `engine/Cargo.toml` (`mira-daemon` dans `[workspace.dependencies]`), `tools/check-architecture` (règle A12 : `mira-daemon` ne dépend d'aucun provider ni `mira-ai-*`, même en dev) et son auto-test

**Interfaces:**
- Consumes: `mira_protocol::{Frame, encode, body_len, decode, CodecError}`.
- Produces :
  - `WireError { TooLarge{announced: u32, max: u32}, Malformed, Truncated, Io(std::io::Error) }`
  - `async read_frame<R: AsyncRead + Unpin>(reader: &mut R, max: u32) -> Result<Option<Frame>, WireError>` : `None` sur fin propre à une frontière de trame ; l'erreur de taille survient après les 4 octets d'en-tête, **sans lire ni allouer le corps**.
  - `async write_frame<W: AsyncWrite + Unpin>(writer: &mut W, frame: &Frame, max: u32) -> Result<(), WireError>`
  - `Token::load_or_create(path: &Path) -> std::io::Result<Token>` : 32 octets aléatoires (`ring::rand::SystemRandom`) en hexadécimal (64 caractères) ; crée le dossier parent ; fichier créé en `0600` (`OpenOptionsExt::mode`) ; un fichier existant lisible par le groupe ou les autres est refusé (`PermissionDenied`) ; `Token::matches(&self, given: &str) -> bool` en temps constant (`ring::constant_time::verify_slices_are_equal`) ; `Token::as_str(&self) -> &str` ; `Debug` n'affiche pas le jeton.

- [ ] **Step 1: Règle A12 et son auto-test**, comme en tâche 2 (`sabotage` ajoute `mira-provider-gmail.workspace = true` à `mira-daemon` ; `expect A12`). Lancer `tools/test-check-architecture`. Expected: ÉCHEC avant le crate.
- [ ] **Step 2: Tests `wire.rs`** (sur `tokio::io::duplex`) : `a_frame_written_is_read_back` ; `a_clean_end_before_a_frame_reads_as_none` ; `an_announced_4_gib_is_refused_without_reading_the_body` (n'écrire que `FF FF FF FF` puis garder le flux ouvert : `read_frame` rend `TooLarge` sans attendre d'autres octets, sous un `timeout` de 1 s) ; `a_size_above_the_server_maximum_is_refused` (max 1024) ; `a_body_cut_short_is_truncated` ; `a_zero_length_or_garbage_body_is_malformed` ; `write_refuses_a_frame_above_the_maximum`.
- [ ] **Step 3: Tests `token.rs`** : `a_new_token_is_64_hex_characters_and_two_files_differ` ; `the_token_file_is_mode_0600` ; `loading_twice_returns_the_same_token` ; `a_file_readable_by_others_is_refused` ; `matches_accepts_only_the_exact_token` (vide, préfixe, jeton plus long, casse différente) ; `debug_never_shows_the_token`.
- [ ] **Step 4: Lancer** `cd engine && cargo test -p mira-daemon --test wire --test token` : FAIL.
- [ ] **Step 5: Implémenter** `wire.rs` et `token.rs`. `Cargo.toml` : `mira-protocol`, `mira-host`, `mira-assembly`, `tokio`, `serde_json`, `ring`, `thiserror`, `clap` ; dev : `mira-testkit`, `tempfile`.
- [ ] **Step 6: Lancer** les tests de l'étape 4, `tools/test-check-architecture`, clippy, fmt, `tools/check-norms engine/crates/mira-daemon`. Expected: PASS.
- [ ] **Step 7: Commit** — `feat(daemon): trames sur flux d'octets et jeton local (mira-daemon)`.

---

### Task 5: Transports : socket Unix et TCP loopback

**Files:**
- Create: `engine/crates/mira-daemon/src/transport/mod.rs`, `transport/unix.rs`, `transport/tcp.rs`
- Test: `tests/transport_unix.rs`, `tests/transport_tcp.rs`

**Interfaces:**
- Produces :
  - `trait ByteStream: AsyncRead + AsyncWrite + Send + Unpin {}` avec implémentation générale
  - `enum Trust { LocalUser(Option<u32>), Remote }` ; aucun écouteur de la phase 2 ne produit `Remote` (réservé à la v4) ; pas de politique d'accès par `Trust` avant la v4
  - `struct Accepted { pub stream: Box<dyn ByteStream>, pub trust: Trust }`
  - `trait Listener: Send + Sync { fn accept(&self) -> impl Future<Output = std::io::Result<Accepted>> + Send; fn describe(&self) -> String; }`
  - `UnixSocketListener::bind(path: &Path) -> std::io::Result<Self>` (`cfg(unix)`) : crée le dossier parent en `0700` ; refuse un dossier existant lisible par le groupe ou les autres ; si le fichier de socket existe, tente de s'y connecter : si une connexion aboutit → `AddrInUse`, sinon le fichier est périmé et supprimé ; bind ; fixe `0600` ; `accept` ne rend que les pairs dont `peer_cred().uid()` égale l'uid propriétaire du fichier de socket (les autres sont fermés, l'écoute continue) ; `Trust::LocalUser(Some(uid))` ; `Drop` supprime le fichier.
  - `fn peer_is_owner(peer_uid: u32, owner_uid: u32) -> bool` (pure, testée)
  - `LoopbackTcpListener::bind(port: u16) -> std::io::Result<Self>` : écoute sur `127.0.0.1` uniquement ; `local_port(&self) -> u16` ; `Trust::LocalUser(None)`.

- [ ] **Step 1: Tests `transport_unix.rs`** : `the_socket_file_is_0600_and_its_directory_0700` ; `a_client_of_the_same_user_is_accepted_with_its_uid` ; `a_stale_socket_file_is_replaced` (fichier laissé par un écouteur dont le processus n'existe plus : créer l'écouteur, `std::mem::forget` son fichier en le copiant d'un `UnixListener` abandonné sans `Drop`) ; `a_live_socket_is_refused_with_addr_in_use` ; `a_directory_open_to_others_is_refused` ; `dropping_the_listener_removes_the_socket_file` ; `only_the_owner_uid_passes_peer_check` (`peer_is_owner`). Lancer : FAIL.
- [ ] **Step 2: Tests `transport_tcp.rs`** : `the_listener_is_bound_to_loopback_only` ; `a_client_connects_and_is_local_user_without_uid` ; `binding_port_0_picks_a_free_port`.
- [ ] **Step 3: Implémenter** (aucun `unsafe` : l'uid propriétaire vient de `MetadataExt::uid` du fichier de socket, l'uid du pair de `tokio::net::UnixStream::peer_cred`).
- [ ] **Step 4: Lancer** les deux fichiers de tests, clippy, fmt, norms. Expected: PASS.
- [ ] **Step 5: Commit** — `feat(daemon): écouteurs socket Unix (0600, uid du pair) et TCP loopback`.

---

### Task 6: Session : poignée de main et appels

**Files:**
- Create: `engine/crates/mira-daemon/src/server.rs` (`ServerContext`, `serve`), `src/session/mod.rs` (boucle), `src/session/hello.rs`, `src/session/calls.rs`
- Create (support de test): `tests/support/mod.rs`, `tests/support/daemon.rs` (`TestDaemon`), `tests/support/client.rs` (`TestClient`)
- Test: `tests/session_hello.rs`, `tests/session_calls.rs`

**Interfaces:**
- Consumes: `Host` (`execute`), `wire`, `Token`, `Listener`, `mira_protocol::*`.
- Produces :
  - `ServerContext { host: Arc<Host>, token: Token, max_frame: u32, hello_timeout: Duration, event_queue: usize, state: tokio::sync::watch::Sender<HostState>, shutdown: tokio::sync::Notify }` avec `ServerContext::new(host: Arc<Host>, token: Token) -> Self` (valeurs par défaut : `MAX_FRAME_BYTES`, 5 s, 256, `Starting`)
  - `async serve<L: Listener>(listener: L, ctx: Arc<ServerContext>)` : accepte jusqu'à `ctx.shutdown`, une tâche de session par connexion
  - Session : lit `hello` dans `hello_timeout` ; vérifie dans cet ordre `check_hello` puis le jeton ; répond `hello_ok{protocol_version, contract_version, host_state}` ou `hello_err` puis ferme. Ensuite : `call` → tâche détachée qui exécute `host.execute(request.to_string())` et répond `result{id, envelope}` ; `ping` → `pong` (même `nonce`). Un écrivain unique (canal borné de 64 trames) sérialise les écritures.
  - Violations de protocole = fermeture de la connexion sans réponse : trame avant `hello`, second `hello`, trame de réponse (`hello_ok`, `result`, `event`, …) envoyée par le client, `id` réutilisé alors que l'échange est en cours, trame illisible ou trop grande. `credit` et `cancel` sur un `id` inconnu sont ignorés (course avec `end`).
  - Support de test : `TestDaemon::start(config: EngineConfig) -> TestDaemon` (hôte ouvert, `UnixSocketListener` dans un dossier jetable, jeton, tâche `serve`), `TestDaemon::start_closed()` (hôte jamais ouvert), `.socket_path()`, `.token()`, `.context()` ; `TestClient::connect(&TestDaemon) -> TestClient` (connexion brute), `.hello_ok()` (envoie un `hello` valide et attend `hello_ok`), `.send(Frame)`, `.recv() -> Option<Frame>` (avec délai), `.call(Value) -> Value`, et un journal des octets reçus (`received_bytes()`) pour la tâche 11.

- [ ] **Step 1: Tests `session_hello.rs`** : `a_valid_hello_gets_hello_ok_with_versions_and_ready_state` ; `a_wrong_token_gets_hello_err_auth_and_the_connection_closes` (la trame `hello_err` ne contient que `reason`, et les octets reçus ne contiennent ni le jeton du serveur ni celui fourni) ; `a_different_protocol_major_gets_hello_err_protocol_version` ; `a_newer_contract_minor_gets_hello_err_contract_version` ; `a_frame_before_hello_closes_the_connection` ; `a_second_hello_closes_the_connection` ; `a_silent_connection_is_closed_after_the_hello_timeout` (200 ms) ; `bytes_trickled_slower_than_the_timeout_are_closed` (en-tête d'un octet toutes les 100 ms, délai de 300 ms : la lecture de `hello` est bornée dans son ensemble, pas octet par octet).
- [ ] **Step 2: Tests `session_calls.rs`** : `a_call_gets_the_contract_envelope_matched_by_id` (`list_accounts` sur un moteur vide : `envelope.ok == true`, `contract_version.major == 1`) ; `concurrent_calls_are_matched_by_id` (trois `call` envoyés d'affilée avec des requêtes aux réponses distinctes, réponses lues dans n'importe quel ordre, chacune appariée à son `id`) ; `a_malformed_request_gets_a_failure_envelope_not_a_closed_connection` (`request: {}` → `envelope.ok == false`) ; `a_call_on_a_never_opened_host_answers_moteur_non_ouvert` ; `an_oversize_frame_closes_the_connection_and_the_daemon_keeps_serving` (max 1024 : en-tête de 1 Mio → fermeture ; un second client fait son `hello_ok`) ; `a_truncated_frame_closes_only_that_connection` ; `a_reused_in_flight_id_closes_the_connection` ; `ping_is_answered_with_pong_and_the_same_nonce`.
- [ ] **Step 3: Lancer** `cargo test -p mira-daemon --test session_hello --test session_calls` : FAIL.
- [ ] **Step 4: Implémenter.** Le support de test construit sa configuration de moteur comme `engine/crates/mira-host/tests/support/mod.rs` (trousseau en mémoire, horloge fixe, relevé en arrière-plan coupé).
- [ ] **Step 5: Lancer** les tests de l'étape 3, clippy, fmt, norms. Expected: PASS.
- [ ] **Step 6: Commit** — `feat(daemon): sessions, poignée de main authentifiée et appels concurrents`.

---

### Task 7: Flux : événements, chat, HTML d'origine, crédits, annulation

**Files:**
- Create: `engine/crates/mira-daemon/src/session/streams.rs` (registre `id → StreamControl`), `src/session/events.rs`, `src/session/chat.rs`, `src/session/html.rs`
- Modify: `src/session/mod.rs` (aiguillage des trames `open_events`, `open_chat`, `get_original_html`, `credit`, `cancel`)
- Create (support de test): `tests/support/mail_fixture.rs` (faux IMAP/SMTP du testkit + compte ajouté, port de `start()` et `add_account()` de `bridge/tests/engine_bridge.rs`), `tests/support/ai_fixture.rs` (moteur avec le registre d'IA et un `FakeAiServer`, port de `bridge/tests/support/chat_fixture.rs`)
- Test: `tests/stream_events.rs`, `tests/stream_chat.rs`, `tests/stream_ids.rs`

**Interfaces:**
- Consumes: `Host::pump_events`, `Host::pump_chat_async`, `Host::original_html`, `ServerContext::event_queue`.
- Produces (comportement) :
  - `StreamControl { credits: Arc<Semaphore>, cancel: watch::Sender<bool> }` ; un flux démarre avec `credit` permis ; `credit{n}` en ajoute ; chaque trame `event`/`chat_delta` consomme un permis.
  - Événements : `pump_events` écrit dans une file bornée de `ctx.event_queue` éléments (`try_send`) ; un expéditeur envoie une trame par permis. File pleine → `emit` rend faux et le flux se termine par `end{lagged}`. `cancel` → `end{cancelled}`.
  - Chat : `pump_chat_async` avec un `emit` qui attend un permis **ou** l'annulation (`select!`) ; annulation ou écriture impossible → rend faux (l'hôte annule l'opération par le contrat) ; fin normale → `end{finished}` ; annulation → `end{cancelled}`. La fermeture de la connexion déclenche l'annulation de tous les flux de la connexion.
  - `get_original_html{id, message}` → `result{id, envelope}`.

- [ ] **Step 1: Tests `stream_events.rs`** (moteur sur le faux IMAP ; un `call` local qui émet un événement sert de producteur, par exemple `create_local_label`, à vérifier dans `mira-core/src/events` et à noter dans le ledger) : `events_without_after_do_not_replay_the_journal` ; `events_with_after_zero_replay_the_journal` ; `the_credit_window_is_respected` (`credit: 1` → une trame `event`, puis rien pendant 300 ms ; `credit{n: 2}` → exactement deux de plus) ; `a_client_that_stops_reading_gets_end_lagged_and_resubscribes_with_after` (`event_queue = 2`, `credit: 1`, le client lit le premier événement puis cesse de lire pendant que cinq autres sont produits : `end{lagged}` ; réabonnement avec `after` = `seq` du dernier événement reçu : les événements manquants arrivent dans l'ordre, sans doublon ni trou) ; `cancel_ends_the_event_stream_with_cancelled`.
- [ ] **Step 2: Tests `stream_chat.rs`** (`ai_fixture`, flux `openai_stream` du testkit) : `chat_lines_arrive_as_started_text_usage_done_then_end_finished` ; `a_chat_stream_without_credit_is_suspended_not_dropped` (`credit: 1` : seule la ligne `started` arrive ; pendant 300 ms le faux serveur n'a ni coupé la connexion ni vu la génération terminée ; `credit{n: 8}` → le reste arrive, `done` compris) ; `cancel_mid_stream_cancels_the_operation` (flux lent de 40 morceaux à 100 ms, `cancel` après 2 trames : `end{cancelled}`, connexion au fournisseur fermée par le client, statut de l'opération `cancelled`) ; `closing_the_connection_mid_stream_cancels_the_operation` (même assertions sur le faux serveur et le statut, vérifiées par un second client) ; `a_chat_on_a_closed_host_gets_one_error_line_then_end`.
- [ ] **Step 3: Tests `stream_ids.rs`** : `get_original_html_answers_with_a_result` (message HTML du faux IMAP, `html` contient `<table>` ; message texte seul : `html` nul) ; `a_stream_id_reused_while_open_closes_the_connection` ; `credit_and_cancel_on_an_unknown_id_are_ignored`.
- [ ] **Step 4: Lancer** `cargo test -p mira-daemon --test stream_events --test stream_chat --test stream_ids` : FAIL.
- [ ] **Step 5: Implémenter.** Les tâches de flux sont interrompues à la fermeture de la connexion (fermeture du `watch`) ; les tâches d'appel ne le sont pas.
- [ ] **Step 6: Lancer** les tests de l'étape 4, clippy, fmt, norms. Expected: PASS.
- [ ] **Step 7: Commit** — `feat(daemon): flux d'événements et de chat avec crédits, annulation et HTML d'origine`.

---

### Task 8: Opérations d'hôte, état, arrêt propre, envoi qui survit à la déconnexion

**Files:**
- Create: `engine/crates/mira-daemon/src/session/host_ops.rs`, `src/shutdown.rs`
- Modify: `src/session/mod.rs`, `src/server.rs`
- Test: `tests/host_ops.rs`, `tests/disconnect_send.rs`

**Interfaces:**
- Produces :
  - `host_op{id, op: shutdown}` : répond `result{id, envelope: success(null)}` après avoir lancé l'arrêt ; toute connexion authentifiée peut l'envoyer (tous les transports de la phase 2 sont locaux).
  - `shutdown::run(ctx: &ServerContext)` : arrête l'acceptation, ferme les sessions, appelle `host.close()` (délai de 5 s de `mira-host`), puis réveille `serve`.
  - `ServerContext::set_state(HostState)` ; `hello_ok` annonce l'état courant. `KeyringLocked` et `UpdateRequired` existent dans le protocole mais le démon de la phase 2 ne les produit pas.
  - Hors périmètre, notés dans le ledger : `set_poll_secs` et `retry_unlock` (phases 3–4, ils demandent de rouvrir le moteur ou de détecter le trousseau verrouillé).

- [ ] **Step 1: Tests `host_ops.rs`** : `shutdown_answers_ok_then_serve_returns_and_the_engine_is_closed` (`host.engine()` vide, nouveau `Host` peut rouvrir le même profil : verrou libéré) ; `shutdown_closes_the_other_connections` ; `hello_ok_reports_starting_then_ready` (changer l'état par `set_state` entre deux `hello`) ; `a_new_connection_after_shutdown_is_refused`.
- [ ] **Step 2: Test `disconnect_send.rs`** (`mail_fixture`, faux SMTP) : `a_message_send_survives_the_client_disconnecting` (créer un brouillon et envoyer par `call` ; fermer la socket **sans lire** le `result` ; le faux SMTP reçoit le message ; un second client voit le message envoyé) et `a_sync_started_by_a_client_continues_after_it_disconnects` (démarrer la synchro puis couper ; la boîte de réception se remplit, vue par un second client). Les requêtes exactes viennent de `bridge/tests/engine_bridge.rs` et des tests d'envoi de `mira-core`.
- [ ] **Step 3: Lancer** `cargo test -p mira-daemon --test host_ops --test disconnect_send` : FAIL.
- [ ] **Step 4: Implémenter.**
- [ ] **Step 5: Lancer** les tests de l'étape 3, clippy, fmt, norms. Expected: PASS.
- [ ] **Step 6: Commit** — `feat(daemon): opération d'hôte shutdown, état annoncé, envois indépendants de la connexion`.

---

### Task 9: Le binaire `mira-daemon`

**Files:**
- Create: `engine/crates/mira-daemon/src/config.rs` (`DaemonConfig`, options clap), `src/paths.rs`, `src/run.rs`
- Modify: `src/main.rs`, `src/lib.rs`, `Cargo.toml` (`tokio` avec la fonctionnalité `signal`)
- Test: `tests/binary_lifecycle.rs`, `tests/config.rs`

**Interfaces:**
- Produces :
  - `DaemonConfig` (clap) : `--profile` (défaut `default`), `--home <dir>` (comme `MIRA_HOME`), `--poll-secs <n>`, `--secrets-key-file <path>` (32 octets, fichier `0600`, remplace le trousseau du système comme le `secrets_key` du pont), `--socket <path>` (défaut `$XDG_RUNTIME_DIR/mira/<profile>.sock`, repli `<racine du profil>/daemon.sock`), `--tcp-port <n>` (absent = pas de TCP ; `0` = port libre, affiché), `--token-file <path>` (défaut `<racine du profil>/daemon.token`, racine = `ProfilePaths::resolve(...).root`). Pas de fichier de configuration en phase 2 (phase 4).
  - `async run(config: DaemonConfig) -> ExitCode` : état `Starting` → `production_config` + `supervisor_for` + `platform::adapt` → `Host::open` → état `Ready` ; écoute sur la socket Unix (et TCP si demandé) ; un seul message fixe sur stdout `mira-daemon prêt` une fois les écouteurs prêts (plus `tcp_port=<n>` si TCP), journal `tracing` sans secret ; SIGTERM et SIGINT déclenchent `shutdown::run`.
  - Codes de sortie : 0 arrêt normal ; 3 verrou de profil déjà pris (message fixe sur stderr, rien d'autre) ; 2 configuration invalide.

- [ ] **Step 1: Tests `config.rs`** : `defaults_resolve_socket_and_token_under_the_profile` ; `socket_defaults_to_xdg_runtime_dir_when_set` ; `a_secrets_key_file_of_the_wrong_length_is_refused` ; `a_secrets_key_file_readable_by_others_is_refused`.
- [ ] **Step 2: Tests `binary_lifecycle.rs`** (processus réel via `env!("CARGO_BIN_EXE_mira-daemon")`, `--home` jetable, `--secrets-key-file` : aucun trousseau système) : `the_daemon_starts_and_answers_contract_info_over_its_socket` ; `a_second_daemon_on_the_same_profile_exits_with_code_3_and_the_first_keeps_answering` ; `sigterm_stops_the_daemon_with_code_0_and_removes_the_socket` (signal envoyé par `kill -TERM <pid>` via `Command`, sans libc) ; `a_socket_left_by_a_killed_daemon_is_replaced_at_restart` (`Child::kill`, puis relance, puis `contract_info`) ; `tcp_port_0_listens_on_loopback_and_requires_the_token` (le port lu sur stdout, `hello` avec un faux jeton → `hello_err`, avec le vrai → `hello_ok`) ; `stdout_and_stderr_never_contain_the_token`.
- [ ] **Step 3: Lancer** `cargo test -p mira-daemon --test config --test binary_lifecycle` : FAIL.
- [ ] **Step 4: Implémenter.**
- [ ] **Step 5: Lancer** les tests de l'étape 3 puis `cargo build -p mira-daemon`, clippy, fmt, norms, `tools/check-architecture engine`. Expected: PASS.
- [ ] **Step 6: Commit** — `feat(daemon): binaire mira-daemon (configuration, écoute, arrêt par signal, verrou de profil)`.

---

### Task 10: Jeu de contrat, partie 1 : `Driver` et scénario courrier

Les scénarios sont **déplacés** depuis `bridge/tests` (assertions identiques) vers un jeu unique, exécuté par chaque `Driver`.

**Files:**
- Create: `engine/crates/mira-daemon/tests/contract/mod.rs`, `contract/driver.rs` (trait), `contract/mail_fixture.rs`, `contract/mail_values.rs`, `contract/mail_html.rs`, `contract/mail_events.rs`, `contract/errors.rs`
- Create: `tests/support/direct_driver.rs` (`DirectDriver`), `tests/support/socket_driver.rs` (`SocketDriver`)
- Create: `tests/contract_direct.rs`, `tests/contract_socket.rs`
- Source des scénarios : `bridge/tests/engine_bridge.rs` (deux tests et leurs `check_*`)

**Interfaces:**
- Produces :
  - `trait Driver: Sized { async fn start(config: EngineConfig) -> Self; async fn start_closed() -> Self; async fn execute(&self, request_json: &str) -> String; async fn pump_events(&self, after: Option<u64>, emit: impl FnMut(String) -> bool); async fn pump_chat(&self, request_json: &str, emit: impl FnMut(String) -> bool); async fn original_html(&self, message: &str) -> String; async fn close(&self) -> String; }`
  - `DirectDriver` : un `Host` ; `close` = `Host::close`.
  - `SocketDriver` : un `TestDaemon` plus un `TestClient` attaché ; `execute` = `call` ; `pump_events`/`pump_chat` = `open_*` avec une grande fenêtre de crédit, lecture des trames, `cancel` quand `emit` rend faux, puis attente du `end` ; `original_html` = `get_original_html` ; `close` = `host_op shutdown` (le `result` porte l'enveloppe de succès).
  - Chaque scénario est une `async fn <nom><D: Driver>()` ; `macro_rules! contract_suite { ($driver:ty) => { … } }` (un fichier par thème) génère les `#[tokio::test(flavor = "multi_thread")]` de chaque entrée, pour que `contract_direct.rs` et `contract_socket.rs` tiennent en quelques lignes.

- [ ] **Step 1: Écrire le trait, `DirectDriver` et `contract_direct.rs`** avec les scénarios `the_bridge_serves_real_engine_values_as_json` (renommé `the_engine_serves_real_values_as_json`) et `the_bridge_reports_errors_as_envelopes` (renommé `errors_are_reported_as_envelopes`), portés mot pour mot, sauf `runtime::execute`/`pump_events`/`close` → méthodes du `Driver`. Découper en fichiers pour respecter les normes (`start`/`config`/`raw_mail`/`html_mail` en `mail_fixture.rs`, les `check_*` répartis dans `mail_html.rs` et `mail_events.rs`). Lancer `cargo test -p mira-daemon --test contract_direct` : FAIL tant que le trait n'est pas complet, puis PASS.
- [ ] **Step 2: Écrire `SocketDriver` et `contract_socket.rs`** (mêmes scénarios, mêmes assertions). Lancer `cargo test -p mira-daemon --test contract_socket` : FAIL d'abord (driver incomplet), puis PASS. Si un scénario ne passe que sur un chemin, c'est une divergence de comportement : la corriger dans le démon ou l'hôte, jamais en affaiblissant le test.
- [ ] **Step 3: Vérifier la parité.** Les deux fichiers d'entrée exécutent exactement les mêmes noms de test (`cargo test -p mira-daemon --test contract_direct -- --list` et `--test contract_socket -- --list` : mêmes noms). Clippy, fmt, `tools/check-norms engine/crates/mira-daemon`.
- [ ] **Step 4: Commit** — `test(contract): scénario courrier exécuté sur l'hôte direct et sur la socket`.

---

### Task 11: Jeu de contrat, partie 2 : chat, libellés, règles, aucun secret dans les trames

**Files:**
- Create: `contract/chat_fixture.rs`, `contract/chat_lines.rs`, `contract/chat_cancel.rs`, `contract/chat_errors.rs`, `contract/chat_masked.rs`, `contract/chat_tools.rs`, `contract/chat_draft.rs`, `contract/labels.rs`, `contract/rules.rs`, `contract/no_secrets.rs`
- Modify: `contract/mod.rs`, `tests/contract_direct.rs`, `tests/contract_socket.rs`
- Sources : `bridge/tests/chat_stream*.rs`, `bridge/tests/support/chat_fixture.rs`, `bridge/tests/local_labels_bridge.rs`, `bridge/tests/rules_bridge.rs`

**Interfaces:** `Driver` de la tâche 10 ; aucune nouvelle.

- [ ] **Step 1: Porter** chaque test de ces sources (assertions identiques) vers `contract/`, en remplaçant `runtime::*` par le `Driver` ; la fixture n'a plus besoin du mutex global `SESSION` (un hôte par test). `closed_sink_cancels_the_operation` devient `a_closed_consumer_cancels_the_operation` : sur la socket, « fermer le flux » = `cancel` envoyé quand `emit` rend faux. Brancher les scénarios dans la macro et dans les deux fichiers d'entrée. Lancer `cargo test -p mira-daemon --test contract_direct` puis `--test contract_socket` : PASS sur les deux.
- [ ] **Step 2: Test de sécurité `no_secrets.rs`** : `no_frame_from_the_daemon_contains_a_secret` (socket seulement, entrée propre dans `contract_socket.rs`) : jouer le scénario courrier (mot de passe `pa55-SECRET-xyz`) et un chat (clé `sk-BRIDGE-SECRET-KEY`), puis vérifier que `TestClient::received_bytes()` — toutes les trames sorties du démon, `hello_ok` compris — ne contient ni ces secrets ni le jeton local. Variante `the_local_token_is_never_echoed_in_a_refusal`.
- [ ] **Step 3: Parité** : mêmes listes de noms sur les deux entrées (`-- --list`). Clippy, fmt, norms.
- [ ] **Step 4: Commit** — `test(contract): chat, libellés, règles et absence de secrets sur les deux chemins`.

---

### Task 12: Le pont rejoue le même jeu

Le pont FFI est le troisième chemin ; il garde la couverture de ses enveloppes (`runtime`, `original_html`, `chat_feed`) sans dupliquer les scénarios.

**Files:**
- Create: `bridge/tests/contract_bridge.rs`, `bridge/tests/support/bridge_driver.rs` (`BridgeDriver`)
- Delete (après vérification): `bridge/tests/engine_bridge.rs`, `chat_stream*.rs`, `local_labels_bridge.rs`, `rules_bridge.rs`, `bridge/tests/support/chat_fixture.rs`
- Modify: `bridge/tests/support/mod.rs` (ne garde que ce que `BridgeDriver` utilise)

**Interfaces:** `BridgeDriver : Driver` fait passer chaque appel par `mira_bridge::runtime` et `mira_bridge::original_html` ; l'hôte du pont étant global, `BridgeDriver::start` prend un verrou asynchrone global gardé jusqu'à `close`/`Drop`.

- [ ] **Step 1:** `contract_bridge.rs` inclut les scénarios par `#[path = "../../engine/crates/mira-daemon/tests/contract/mod.rs"] mod contract;` et instancie la macro avec `BridgeDriver`. Lancer `cd bridge && cargo test --test contract_bridge` : PASS.
- [ ] **Step 2: Parité** : la liste des tests de `contract_bridge` égale celle de `contract_direct`, et chaque ancien test du pont a un équivalent par son nom ou son thème (tableau dans le ledger : ancien nom → nouveau nom). Si l'inclusion `#[path]` ne se compile pas, **ne rien supprimer**, ledgériser un ruling et garder les anciens fichiers.
- [ ] **Step 3:** Supprimer les anciens fichiers de test uniquement si l'étape 2 est complète ; `cargo test --no-run` dans `bridge/` ; `git diff --stat bridge/src bridge/Cargo.toml app/` doit être vide, sauf dev-dependencies strictement nécessaires.
- [ ] **Step 4: Commit** — `test(contract): le pont FFI rejoue le jeu de contrat commun`.

---

### Task 13: Documentation et garde-fous

**Files:**
- Create: `docs/PROTOCOLE-DEMON.md` (format des trames, ordre des échanges, crédits, annulation, codes de sortie et options du démon ; source de vérité écrite que le client Dart réimplémentera en phase 3)
- Modify: `docs/ARCHITECTURE-IMPLEMENTEE.md` (lignes `mira-protocol` et `mira-daemon`, sens des dépendances, corriger « Neuf crates » et la phrase « pont → mira-host → mira-assembly »)
- Modify (non suivis par git, `docs/dev/` est ignoré) : `docs/dev/DECISIONS-AGENT.md` (AG-89 : décisions de la phase 2 — jeton exigé sur tous les transports locaux, trame `host_op`, crédit initial dans `open_*`, variantes asynchrones de l'hôte, périmètre reporté de `set_poll_secs`/`retry_unlock`/`keyring_locked`, fichier de configuration en phase 4), `docs/dev/REFLEXION-ARCHITECTURE-DEMON.md` (statut)

- [ ] **Step 1:** Rédiger les trois documents à partir du code livré (aucune affirmation sans le test qui la prouve).
- [ ] **Step 2: Vérifier** : `tools/check-architecture engine` (A10, A11, A12), `tools/test-check-architecture`, `tools/check-norms engine/crates`, `cargo fmt --all --check`, `cargo clippy -p mira-protocol -p mira-host -p mira-daemon --all-targets -- -D warnings`. Expected: 0 violation.
- [ ] **Step 3: Commit** — `docs(daemon): protocole des trames et architecture implémentée`.
