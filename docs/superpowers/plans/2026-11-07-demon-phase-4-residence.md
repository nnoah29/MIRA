# Démon, phase 4 : démon résident, notifications et icône — plan d'implémentation

> **Pour l'exécutant :** exécution native (`superpowers:executing-plans`), TDD strict : chaque correctif ou fonctionnalité est vu échouer avant d'être écrit. Les cases `- [ ]` suivent l'avancement.

**Objectif :** fermer l'app, et la synchro et les notifications continuent.

**Architecture :** le démon (`mira-daemon`) lit un `daemon.toml`, tourne sous `systemd --user` ou à la demande, produit les quatre états d'hôte, et héberge un notifieur (`mira-host`) qui remet des notifications toutes prêtes à un récepteur D-Bus ; une icône StatusNotifierItem est portée par le démon. La présence (focus de l'app) est un mécanisme d'hôte à part.

**Tech stack :** Rust 2024 (toolchain 1.99), `zbus` 5 (déjà dans l'arbre), `toml` 1 (déjà au workspace), `chrono` (heure locale, `clock` seulement), `clap`, tokio ; Dart/Flutter côté app ; `flutter_rust_bridge_codegen` (installé) pour le pont.

**Spec :** `docs/superpowers/specs/2026-11-07-demon-phase-4-residence-design.md` (autorité) ; protocole : `docs/PROTOCOLE-DEMON.md`.

## Contraintes globales

- Règles de `CLAUDE.md` : pas de `unsafe`, pas d'`unwrap`/`expect`/`panic!` hors tests, aucun secret dans trames/journaux/erreurs, fonction ≤ 30 lignes, fichier ≤ 6 fonctions (tests compris, tests dans des fichiers séparés), aucun commentaire dans un corps de fonction, une ligne `///` en français avant chaque `fn`, `//!` de 1 à 3 lignes en tête de fichier, interdits `utils.rs` `helpers.rs` `common.rs` `misc.rs`.
- Le cœur ne nomme aucun provider ; `mira-host` et `mira-protocol` non plus.
- **Jamais** `./ci.sh`, `make check-all` ni suite complète sans accord explicite. Tests ciblés : `cd engine && CARGO_BUILD_JOBS=4 cargo test -q -p <crate> --test <fichier>`. Côté Dart : `flutter analyze` propre, `tools/i18n-audit` à 0 manquant, tests visés par fichier.
- Commits : un par tâche, fichiers nommés (jamais `git add -A`), **sans trailer `Co-Authored-By`**, message `type(domaine): …` avec les D-xx et tests ajoutés dans le corps. Pas de push, pas de fusion.
- Jamais supprimer ni affaiblir un test pour passer au vert ; pas de `#[ignore]` hors procédure de blocage.
- Contrat JSON : seul ajout compatible 1.6 vers 1.7 (tâche 6). Protocole de trames : 1.0 vers 1.1 (tâche 3), ajouts seulement.
- Valeurs fixées par la spec : délai de grâce 30 s ; réessai du trousseau 5 s doublant jusqu'à 60 s ; menu de l'icône = « Ouvrir MIRA », ligne d'état, « Quitter MIRA ».

## Décisions prises en lisant le code (à consigner dans `AG-92`)

1. **`set_poll_secs` n'est pas ajouté.** `EngineSettings.poll_secs` est déjà appliqué en direct (`SyncService::apply_settings` → `supervisor.set_poll`) et relu au démarrage ; l'app l'écrit déjà par `SetSettings`. La dette « non produit par le démon » se solde par un test, pas par du code (tâche 1).
2. **`resident` n'est pas dans `daemon.toml`.** Le démon lancé par `systemd` ne reçoit aucun drapeau (il reste actif) ; celui lancé par l'app reçoit `--exit-when-idle`. L'état résident est « unité activée » (`systemctl --user is-enabled`) ; la réponse à la question du premier lancement est gardée dans les préférences de l'app. L'app n'écrit donc jamais `daemon.toml`. Pas de clé de niveau de journal non plus : le démon n'a pas de journalisation à régler.
3. **Heures calmes.** `NewMessageReceived.silenced` couvre règles, étiquettes et comptes en sourdine, **pas** les heures calmes (`silenced_by_settings` ne regarde que `mutes`). Le notifieur applique les heures calmes avec l'heure locale (`chrono`, derrière un trait pour les tests).
4. **Clic sur une notification.** Il relance l'app (qui devient à instance unique : `G_APPLICATION_DEFAULT_FLAGS`) ; il n'ouvre pas encore le message précis. Le message reste dans `Notification.message` pour Android.
5. **Regroupement.** Comme le code Dart actuel : plus de 3 messages d'un compte dans la fenêtre de regroupement (2 s) → une seule notification « N nouveaux messages ».

## Revue ciblée (cas que la spec ne dit pas)

- `daemon.toml` illisible, vide ou aux clés inconnues → message fixe, code 2, jamais de contenu du fichier dans la sortie. (tâche 1)
- Le dernier client part puis un autre arrive avant 30 s → le démon ne s'arrête pas. (tâche 2)
- Trousseau déverrouillé pendant l'attente, ou `retry_unlock` pendant un réessai → un seul passage à `ready`. (tâche 3)
- Client plus ancien que le démon (contrat) → refus `contract_version` comme avant, pas `update_required`. (tâche 4)
- Deux clients, un seul au premier plan ; le premier se déconnecte → `is_active()` repasse à faux. (tâche 5)
- Heures calmes passant minuit (22-7) ; réglages absents d'un moteur 1.6 → défauts. (tâches 6, 7)
- Notification avec `preview` faux → ni expéditeur ni objet. (tâche 7)
- Bus D-Bus absent (pas de service de notifications) → le démon continue, une seule ligne dans sa sortie. (tâche 8)

---

### Tâche 1 : fichier `daemon.toml` et solde de `set_poll_secs`

**Files :** Create `engine/crates/mira-daemon/src/config/file.rs`, `engine/crates/mira-daemon/tests/config_file.rs`, `engine/crates/mira-daemon/tests/poll_secs_live.rs` ; Modify `config/mod.rs` (déclaration + fusion), `setup.rs` (lecture du fichier avant `resolve_paths`), `Cargo.toml` du démon (`toml.workspace = true`, `serde` avec `derive`).

**Interfaces — produit :**
- `pub struct FileConfig { pub profile: Option<String>, pub home: Option<PathBuf>, pub socket: Option<PathBuf>, pub token_file: Option<PathBuf> }` (`#[serde(deny_unknown_fields, default)]`).
- `pub fn config_file_path(environment: &ProfileEnvironment) -> Result<PathBuf, MiraError>` : `<dossier MIRA>/daemon.toml`, où le dossier MIRA est le parent de `profiles/` (obtenu avec `ProfilePaths::resolve(&ProfileId::from_raw("default"), environment)` : `root.parent().parent()`).
- `pub fn read_file_config(path: &Path) -> Result<FileConfig, MiraError>` : fichier absent → `FileConfig::default()` ; illisible, non-UTF-8, clé inconnue ou plus de 64 Kio → `MiraError` à message fixe (`invalid_input::PARAMETER`, « fichier de configuration du démon invalide »).
- `DaemonConfig::merged_with(self, file: FileConfig) -> DaemonConfig` : une option de la ligne de commande l'emporte, sinon le fichier. `profile` a le défaut `"default"` de clap : il n'est remplacé par le fichier que si la ligne de commande ne l'a pas donné (passer `profile` de `String` à `Option<String>` n'est pas nécessaire : utiliser `ArgMatches::value_source`, ou changer le champ en `Option<String>` avec accesseur `profile()` — choisir la plus simple).

- [ ] **Étape 1 : tests rouges** dans `config_file.rs` : `absent_file_gives_defaults`, `file_values_fill_missing_options`, `command_line_wins_over_file`, `unknown_key_is_refused_with_fixed_message` (le message ne contient ni le chemin ni le contenu), `oversized_file_is_refused`, `unreadable_file_is_refused_without_leaking_path`.
- [ ] **Étape 2 :** lancer `cargo test -q -p mira-daemon --test config_file` → échec de compilation (module absent).
- [ ] **Étape 3 :** implémenter `file.rs` et la fusion ; brancher dans `setup::prepare` (message fixe « configuration invalide », code 2).
- [ ] **Étape 4 :** test vert ; `cargo test -q -p mira-daemon --test config` (existant) toujours vert.
- [ ] **Étape 5 : test `poll_secs_live.rs`** (pas de code) : `poll_secs_set_through_the_socket_is_stored_and_announced` — démon de test (support existant `tests/support/daemon.rs`), `SetSettings{poll_secs: Some(60)}` par `Call`, puis `GetSettings` rend 60 et un événement `settings_changed` contient `poll_secs`. Rouge seulement si le contrat est cassé ; il doit être vert d'emblée : c'est la preuve de la décision 1.
- [ ] **Étape 6 : commit** `feat(daemon): fichier de configuration daemon.toml (démarrage seulement), relevé réglable en direct vérifié` — fichiers nommés.

### Tâche 2 : sortie du démon quand le dernier client part

**Files :** Create `engine/crates/mira-daemon/src/idle.rs`, `tests/idle_exit.rs` ; Modify `config/mod.rs` (`--exit-when-idle`, `--idle-grace-secs` défaut 30), `server/context.rs` (compteur de clients), `session/mod.rs` (inscrire/désinscrire après `handshake` réussi), `run.rs` (attendre l'arrêt **ou** l'inactivité).

**Interfaces — produit :**
- `ServerContext::client_attached(&self) -> ClientGuard` (le garde décrémente au `Drop`) et `ServerContext::clients(&self) -> watch::Receiver<usize>`.
- `pub async fn wait_until_idle_for(ctx: &ServerContext, grace: Duration)` : rend quand le nombre de clients est resté à 0 pendant toute la durée `grace` ; un client qui arrive remet le compte à zéro. Le démon sans client depuis le début n'attend pas : la grâce court depuis le démarrage.

- [ ] **Étape 1 : tests rouges** (`idle_exit.rs`, `--idle-grace-secs 1` pour la vitesse via le processus de test existant `tests/support/process.rs`) : `exits_after_last_client_leaves_and_grace`, `new_client_within_grace_keeps_daemon_alive`, `without_flag_daemon_never_exits_on_idle`, `exit_code_is_success_and_socket_removed`.
- [ ] **Étape 2 :** exécuter → échec (option inconnue).
- [ ] **Étape 3 :** implémenter ; `run.rs` : `tokio::select!` entre `signals::wait_for_stop` et `idle::wait_until_idle_for` quand `exit_when_idle`.
- [ ] **Étape 4 :** vert, plus `--test binary_lifecycle` et `--test binary_shutdown` (existants).
- [ ] **Étape 5 : commit** `feat(daemon): arrêt après le départ du dernier client (--exit-when-idle, délai de grâce)`.

### Tâche 3 : `keyring_locked` et `retry_unlock`

**Files :** Create `mira-daemon/src/unlock.rs`, `mira-daemon/tests/keyring_locked.rs` ; Modify `mira-protocol/src/auth.rs` (`HostOp::RetryUnlock`), `mira-protocol/src/version.rs` (minor 1), `mira-protocol/tests/frames.rs` + vecteurs `tests/golden/` (voir le format des fichiers existants), `session/host_ops.rs`, `server/context.rs` (`Notify` de réessai), `run.rs`, `docs/PROTOCOLE-DEMON.md`.

**Interfaces — produit :**
- `HostOp::RetryUnlock` (JSON `{"op":"retry_unlock"}` comme `Shutdown`).
- `pub struct UnlockBackoff { pub first: Duration, pub max: Duration }` (défaut 5 s / 60 s ; options cachées `--unlock-first-secs`, `--unlock-max-secs` pour les tests ; ou champ de `ServerContext`).
- `pub async fn open_when_unlocked(ctx: &Arc<ServerContext>, engine: EngineConfig, backoff: UnlockBackoff) -> Result<(), ExitCode>` : essaie `ctx.host.open(engine.clone())` ; si l'erreur est `secrets::KEYRING_LOCKED`, passe `HostState::KeyringLocked`, attend `min(délai, max)` **ou** `ctx.retry_unlock.notified()`, double le délai, recommence ; toute autre erreur garde le traitement actuel de `serve_until_stopped` (verrou de profil = code 3, sinon 1). Sort aussi sur arrêt demandé.
- `ServerContext::request_unlock_retry(&self)`.
- `EngineConfig` doit être `Clone` (sinon construire la configuration à chaque essai via `setup`).

- [ ] **Étape 1 : tests rouges** : `mira-protocol/tests/frames.rs::retry_unlock_round_trips` + golden ; `keyring_locked.rs` : `locked_keyring_is_announced_then_ready_after_unlock` (faux trousseau du testkit ou configuration `secrets_key_file` pointant vers un fichier créé après coup — choisir ce que `mira-testkit`/`engine_config.rs` permet ; si aucun moyen de simuler un trousseau verrouillé n'existe, ajouter un trousseau factice dans `tests/support/` qui renvoie `KEYRING_LOCKED` N fois), `retry_unlock_skips_the_wait`, `only_one_ready_transition`, `shutdown_during_wait_exits_cleanly`.
- [ ] **Étape 2 :** exécuter → rouge.
- [ ] **Étape 3 :** implémenter. **Une seule notification** « en attente du trousseau » est émise par la tâche 8 (point d'accroche : `ServerContext::set_state(KeyringLocked)` déclenchera le notifieur) ; ici, seulement l'état.
- [ ] **Étape 4 :** vert ; relancer `cargo test -q -p mira-protocol` et `-p mira-daemon --test session_hello --test host_ops`.
- [ ] **Étape 5 : commit** `feat(daemon): trousseau verrouillé annoncé, réessai périodique et retry_unlock (protocole 1.1)`.

### Tâche 4 : `update_required` avec seule la sortie `shutdown`

**Files :** Create `mira-daemon/tests/update_required.rs` ; Modify `mira-protocol/src/handshake.rs` (`pub fn client_is_newer(server: ContractVersion, client: ContractVersion) -> bool`), `session/hello.rs`, `session/mod.rs` (mode restreint), `session/host_ops.rs`.

**Interfaces — produit :**
- `client_is_newer` : `client.major > server.major || (client.major == server.major && client.minor > server.minor)`.
- `hello.rs` : jeton valide, protocole de même majeure, et `client_is_newer` → `Frame::HelloOk { host_state: UpdateRequired, .. }` et la session passe en mode restreint ; toute trame autre que `HostOp::Shutdown`, `Ping`, `Pong` ou `Unknown` ferme la connexion (violation). Un client plus ancien reste refusé par `HelloErr::ContractVersion`.

- [ ] **Étape 1 : tests rouges** : `newer_client_gets_hello_ok_update_required`, `restricted_session_refuses_calls`, `restricted_session_accepts_shutdown`, `older_client_still_refused_contract_version`, `bad_token_wins_over_version` (le jeton est vérifié avant, aucune fuite).
- [ ] **Étape 2–4 :** rouge, implémenter, vert (+ `--test session_compat`, `--test session_hello`).
- [ ] **Étape 5 : commit** `feat(daemon): update_required annoncé à un client plus récent, seul shutdown permis`.

### Tâche 5 : présence

**Files :** Create `mira-host/src/presence.rs`, `mira-host/tests/presence.rs`, `mira-daemon/tests/presence_ops.rs` ; Modify `mira-protocol/src/auth.rs` (`HostOp::SetPresence { focused: bool }`, golden), `mira-host/src/host.rs` + `lib.rs` (`Host::presence()`), `mira-daemon/src/session/hello.rs`/`mod.rs` (garder `client_id` de `Hello`, inscrire au `hello_ok`, retirer à la fin de la session), `session/host_ops.rs`, `bridge/src/api/engine.rs` (`set_presence(focused: bool)` synchrone) et régénération FRB, `docs/PROTOCOLE-DEMON.md`.

**Interfaces — produit :**
- `pub struct Presence` (`Default`, interne `Mutex<HashMap<String, bool>>`) avec `set(&self, client: &str, focused: bool)`, `remove(&self, client: &str)`, `is_active(&self) -> bool`, `subscribe(&self) -> watch::Receiver<bool>`.
- `Host::presence(&self) -> &Presence`.
- Pont : `#[frb(sync)] pub fn set_presence(focused: bool)` utilise le client `"embedded"`.

- [ ] **Étape 1 : tests rouges** : `presence.rs` : `no_client_is_inactive`, `one_focused_client_activates`, `two_clients_one_leaves`, `remove_unknown_is_harmless`, `subscribe_sees_changes` ; `presence_ops.rs` : `set_presence_over_socket_marks_active`, `connection_drop_removes_presence`, `unknown_client_ids_are_isolated`.
- [ ] **Étape 2–4 :** rouge, implémenter, vert.
- [ ] **Étape 5 :** `cd bridge && flutter_rust_bridge_codegen generate` (voir `bridge/README.md` / `flutter_rust_bridge.yaml` pour la commande exacte) ; `cargo build` du pont ; `app` : `flutter analyze` sur les fichiers générés.
- [ ] **Étape 6 : commit** `feat(host): présence des clients (set_presence) pour le démon et le pont`.

### Tâche 6 : réglages de notification, contrat 1.7

**Files :** Modify `mira-vocabulary/src/domain/engine_settings.rs` (struct `NotificationSettings`), `contract/version.rs` (1.7), `mira-core/src/storage/repositories/engine_settings/{read,write}.rs` (clés `notify_enabled`, `notify_preview`, `notify_sound`, `notify_when_focused`), `mira-core/src/api/handlers/settings/set.rs` (rien à valider ; `changed_keys` couvre `notifications`), tests `mira-vocabulary/tests/engine_settings.rs` et `mira-core/tests/api_settings_notifications.rs`, `contract_examples/settings_domain.rs`, documentation du contrat (`docs/ENGINE.md` et chap. 12 non modifié : écrire dans `PROPOSITIONS-REFERENCE.md` si le chapitre doit changer).

**Interfaces — produit :**
- `pub struct NotificationSettings { pub enabled: bool, pub preview: bool, pub sound: bool, pub when_focused: bool }`, `Default` = `{ true, true, true, false }`, `#[serde(default)]`.
- `EngineSettings.notifications: NotificationSettings` (`#[serde(default)]`) ; `changed_keys` ajoute `"notifications"`.
- `CONTRACT_VERSION = 1.7`.

- [ ] **Étape 1 : tests rouges** : vocabulaire `notification_defaults_match_previous_app_defaults`, `old_json_without_notifications_loads_defaults`, `changed_keys_reports_notifications` ; moteur `notification_settings_round_trip_and_event_keys` ; contrat `contract_examples` mis à jour ; `a_1_6_client_can_talk_to_1_7_engine` (règle `is_compatible_with`).
- [ ] **Étape 2–4 :** rouge, implémenter, vert ; relancer `-p mira-vocabulary`, et dans `mira-core` les fichiers `--test api_settings --test api_settings_notifications --test storage_ai_history_setting`.
- [ ] **Étape 5 : commit** `feat(contract): réglages de notification du moteur (contrat 1.7)`.

### Tâche 7 : le notifieur (`mira-host`)

**Files :** Create `mira-host/src/notifier/{mod,notification,decision,batching,clock,run}.rs`, `mira-host/tests/notifier_decision.rs`, `notifier_batching.rs`, `notifier_run.rs`, `tests/support/fake_sink.rs` ; Modify `mira-host/Cargo.toml` (`chrono = { version = "0.4", default-features = false, features = ["clock"] }`), `lib.rs`. Vérifier `cargo deny check` en local (hors `ci.sh`) pour `chrono`.

**Interfaces — produit :**
- `pub struct Notification { pub id: u32, pub title: String, pub body: String, pub sound: bool, pub message: Option<String> }`.
- `pub trait NotificationSink: Send + Sync { fn show(&self, notification: Notification); }`.
- `pub trait LocalClock: Send + Sync { fn hour(&self) -> u8; }` + `SystemClock`.
- `pub fn decide(settings: &EngineSettings, event_silenced: bool, account_muted: bool, active: bool, hour: u8) -> Decision` avec `Decision::{Show, Skip}` : `Skip` si `!notifications.enabled`, si `silenced`, si compte en sourdine, si `active && !when_focused`, si `hour` dans `quiet_hours` (plage passant minuit prise en charge).
- `pub fn compose(arrivals: &[Arrival], settings: &NotificationSettings) -> Vec<Notification>` : au plus 3 → une notification par message (`preview` : expéditeur + objet ; sinon « Nouveau message » + compte) ; plus de 3 → une seule « N nouveaux messages ».
- `pub async fn run_notifier(host: Arc<Host>, sink: Arc<dyn NotificationSink>, clock: Arc<dyn LocalClock>, window: Duration)` : s'abonne aux événements (`engine.subscribe`, sans rejeu), sur `NewMessageReceived` lit `GetSettings` et `GetMessage` (en-tête seulement, jamais le corps), regroupe pendant `window` (2 s en production), appelle `decide`/`compose`, remet à `sink`. Se termine à la fermeture du moteur.

- [ ] **Étape 1 : tests rouges** : `notifier_decision.rs` — table `decide` : `disabled_skips`, `silenced_skips`, `muted_account_skips`, `focused_app_skips_unless_when_focused`, `quiet_hours_same_day`, `quiet_hours_across_midnight_22_to_7`, `no_quiet_hours_never_skips` ; `notifier_batching.rs` — `three_messages_three_notifications`, `four_messages_one_summary`, `preview_off_hides_sender_and_subject`, `body_never_contains_secret_like_tokens` (corpus d'en-têtes contenant `Bearer`, clé factice) ; `notifier_run.rs` — avec le moteur de test du testkit et `FakeSink` : `new_message_event_reaches_sink`, `silenced_event_does_not`, `presence_active_blocks_then_allows_after_drop`.
- [ ] **Étape 2–4 :** rouge, implémenter par fichier (une responsabilité chacun), vert.
- [ ] **Étape 5 : commit** `feat(host): notifieur du moteur (décision, regroupement, récepteur abstrait)`.

### Tâche 8 : récepteur D-Bus des notifications dans le démon

**Files :** Create `mira-daemon/src/desktop/{mod,notification_sink,launch_app}.rs`, `mira-daemon/tests/dbus_notifications.rs`, `tests/support/fake_notification_server.rs` ; Modify `mira-daemon/Cargo.toml` (`zbus` avec `tokio` ; vérifier `cargo deny`), `run.rs` (lancer `run_notifier` après le passage à `ready`, et la notification unique « trousseau » à l'entrée en `KeyringLocked`), `docs/PROTOCOLE-DEMON.md`.

**Interfaces — produit :**
- `pub struct DbusNotificationSink` : `connect() -> Result<Self, DesktopError>` (bus de session), implémente `NotificationSink::show` en appelant `org.freedesktop.Notifications.Notify` (catégorie `email.arrived`, action `default` « Ouvrir »), et écoute `ActionInvoked` pour appeler `launch_app`.
- `pub fn launch_app(command: &[String])` : lance le binaire de l'app (chemin absolu résolu comme `defaultBinaryCandidates` côté Dart : à côté du démon, `…/mira`, sinon `gtk-launch io.github.nnoah29.Mira`), jamais d'argument contenant des données d'un message.
- Bus absent ou sans service de notifications → `show` ne fait rien ; une seule ligne `say("notifications du bureau indisponibles")` au démarrage.

- [ ] **Étape 1 : tests rouges** (`dbus-run-session` lancé par le test pour avoir un bus privé ; si `dbus-run-session` est introuvable, le test **échoue** avec un message clair) : `notify_reaches_a_fake_notification_server`, `action_invoked_launches_the_app` (commande de lancement injectée), `missing_service_is_silent_and_daemon_continues`, `notification_text_matches_notification_struct`.
- [ ] **Étape 2–4 :** rouge, implémenter, vert ; relancer `--test binary_lifecycle`.
- [ ] **Étape 5 : commit** `feat(daemon): notifications du bureau par D-Bus, clic qui relance l'app`.

### Tâche 9 : icône de zone de notification

**Files :** Create `mira-daemon/src/desktop/tray/{mod,menu,status_line}.rs`, `mira-daemon/tests/dbus_tray.rs`, `assets/` si une icône est nécessaire (réutiliser `packaging/common/io.github.nnoah29.Mira.png`, nom d'icône du thème `io.github.nnoah29.Mira`) ; Modify `Cargo.toml`, `run.rs`.

**Interfaces — produit :**
- Trait `Tray { fn set_status(&self, status: TrayStatus); }` avec `TrayStatus::{UpToDate, KeyringLocked, UpdateRequired}` ; `status_line(TrayStatus) -> &'static str` (« À jour », « Trousseau verrouillé », « Mise à jour requise »).
- Implémentation StatusNotifierItem : essayer la crate `ksni` ; **si `cargo deny check licenses` la refuse ou si sa version ne s'appuie pas sur `zbus` 5/tokio, s'arrêter, consigner dans `BLOCAGES.md`/AG-92 et demander à l'utilisateur** (pas d'écriture à la main de `dbusmenu`).
- Menu : « Ouvrir MIRA » (aussi `Activate`) → `launch_app` ; ligne d'état désactivée ; « Quitter MIRA » → `shutdown::run`.
- Aucun hôte StatusNotifierWatcher → une ligne dans la sortie, le démon continue.

- [ ] **Étape 1 : tests rouges** : `tray_registers_with_watcher`, `menu_has_exactly_three_entries`, `quit_entry_stops_the_daemon`, `open_entry_launches_app`, `no_watcher_is_silent`, `status_follows_host_state`.
- [ ] **Étape 2–4 :** rouge, implémenter, vert.
- [ ] **Étape 5 : commit** `feat(daemon): icône de zone de notification (Ouvrir, état, Quitter)`.

### Tâche 10 : client Dart du démon (présence, déverrouillage, mise à jour, sortie immédiate)

**Files :** Modify `app/lib/engine/remote/{frame_builders,frames,remote_engine_client,connection,daemon_launcher,engine_opening,host_errors,host_state}.dart` ; tests `app/test/engine/remote/…` (suivre les noms existants : `frames_test`, `daemon_launcher_test`, `engine_opening_test`, vecteurs partagés `engine/crates/mira-protocol/tests/golden`).

**Interfaces — produit :**
- `Frame` Dart : `hostOpSetPresence(bool focused)`, `hostOpRetryUnlock()`.
- `RemoteEngineClient.setPresence(bool focused)`, `retryUnlock()`, `restartDaemon()`.
- `DaemonLauncher.start({bool onDemand = true})` passe `--exit-when-idle` ; `watchExit()` : `Future<int>` terminé quand le processus lancé sort, avec son code ; `waitReady` abandonne aussitôt à la sortie (au lieu des 20 s) et rend l'erreur lisible (code 3 → « profil déjà utilisé », 2 → « configuration invalide »).
- `update_required` : `engine_opening` rend un `HostUpdateRequired` ; `UpdateFlow.run()` envoie `shutdown`, relance (`systemctl --user restart mira-daemon.service` si l'unité est active, sinon relance directe), sans question si `clients == 1`, sinon demande (texte i18n).
- `keyring_locked` : message i18n + bouton « Réessayer » → `retryUnlock()`.

- [ ] **Étape 1 : tests rouges** (un par comportement, fichiers existants prolongés, pas de nouveaux types de test) : trames golden, `launcher_passes_exit_when_idle`, `early_exit_is_reported_with_code_not_after_20s`, `update_required_restarts_without_asking_when_alone`, `update_required_asks_when_others_attached`, `retry_unlock_button_sends_op`.
- [ ] **Étape 2–4 :** rouge, implémenter, vert (`flutter test` des fichiers touchés), `flutter analyze`, `tools/i18n-audit`.
- [ ] **Étape 5 : commit** `feat(client): présence, retry_unlock, redémarrage sur update_required, sortie immédiate du démon détectée`.

### Tâche 11 : réglages de notification côté app, migration et suppression du code Dart de décision

**Files :** Modify `app/lib/engine/engine_settings_admin.dart` (`notifications`), `app/lib/state/store.dart` (migration unique, `setNotify` écrit dans le moteur, suppression de `_announce`, `appFocused` ne sert plus qu'à `setPresence`), `app/lib/state/mail_notifier.dart` (ne garde que `SystemNotifications`/`PluginNotifications` comme **afficheur**), `app/lib/state/arrival_gate.dart` (supprimé si plus utilisé), `app/lib/ui/notification_settings.dart`, `app/lib/main.dart` ; Create `app/lib/engine/notification_feed.dart` ; tests `app/test/state/…`.
Côté pont embarqué : Create `bridge/src/notifications.rs` + `#[api] watch_notifications(sink)` (flux de `Notification` en JSON) avec un `NotificationSink` qui pousse dans le flux, démarré par `open_engine` ; régénérer FRB.

**Interfaces — produit :**
- `EngineSettings.notifications : NotificationSettings {enabled, preview, sound, whenFocused}` (JSON `notifications`, défauts du moteur).
- `Store.migrateNotificationPrefs()` : si le moteur est 1.7 et la préférence `notificationsMigrated` absente, copier `notify`, `notifyPreview`, `notifySound`, `notifyFocused` dans le moteur puis poser le drapeau et ne plus lire ces clés.
- Mode démon : aucun afficheur Dart (le démon affiche). Mode embarqué (Android, repli Linux) : `notification_feed.dart` écoute `watch_notifications` et appelle `PluginNotifications.show` sans aucune règle.
- Écran des réglages inchangé visuellement.

- [ ] **Étape 1 : tests rouges** : `migrates_prefs_once`, `settings_screen_writes_engine_not_prefs`, `daemon_mode_installs_no_dart_notifier`, `embedded_mode_displays_what_the_engine_sends`, `appfocus_reports_presence`, `no_arrival_gate_left` (le fichier supprimé, plus aucune référence — `flutter analyze` le prouve) ; Rust : `bridge/tests` — `notification_stream_emits_notification_json`.
- [ ] **Étape 2–4 :** rouge, implémenter, vert ; `flutter analyze` ; `tools/i18n-audit`.
- [ ] **Étape 5 : commit** `feat(app): notifications portées par le moteur, l'app n'affiche plus que les réglages (migration des préférences)`.

### Tâche 12 : mode résident côté app et instance unique

**Files :** Create `app/lib/engine/remote/resident.dart` (`ResidentController`), `app/lib/ui/resident_prompt.dart` (question du premier lancement), tests ; Modify `app/lib/ui/settings_panel.dart` (interrupteur « Garder MIRA actif en arrière-plan »), `app/lib/main.dart` (question une fois), `app/linux/runner/my_application.cc` (`G_APPLICATION_DEFAULT_FLAGS`, activation de la fenêtre existante), `app/lib/i18n/…` (fr + en, `tools/i18n-audit`).

**Interfaces — produit :**
- `ResidentController({required ProcessRunner run})` : `Future<bool> isEnabled()` (`systemctl --user is-enabled mira-daemon.service`), `enable()` (`systemctl --user enable --now …`), `disable()` (`systemctl --user disable --now …`), `isAvailable()` (systemd utilisateur présent **et** unité installée, sinon le choix n'est pas proposé : AppImage, flatpak, snap).
- Préférence `residentAnswered` ; la question ne s'affiche qu'une fois, seulement si `isAvailable()`.
- Une seconde instance de l'app active la première au lieu d'ouvrir une deuxième fenêtre.

- [ ] **Étape 1 : tests rouges** : `prompt_shown_once_only_when_available`, `yes_enables_unit_no_does_not`, `toggle_in_settings_enables_and_disables`, `unavailable_hides_prompt_and_toggle`.
- [ ] **Étape 2–4 :** rouge, implémenter, vert ; `flutter analyze`.
- [ ] **Étape 5 :** vérification manuelle décrite au journal de bord (pas de test automatisé possible pour GTK) : lancer l'app deux fois, une seule fenêtre.
- [ ] **Étape 6 : commit** `feat(app): mode résident (question au premier lancement, interrupteur) et instance unique sous Linux`.

### Tâche 13 : paquets, unité `systemd`, documentation

**Files :** Create `packaging/common/mira-daemon.service`, `packaging/tests/units_and_payload.sh` ; Modify `packaging/build-bundle.sh` (compiler `mira-daemon` en release, le copier dans le bundle), `packaging/common/install-tree.sh` (unité vers `usr/lib/systemd/user/`, démon à côté de l'app), `packaging/deb/*`, `packaging/rpm/mira-mail.spec`, `packaging/arch/PKGBUILD`, `packaging/tarball.sh` (+ `install.sh` : unité vers `~/.config/systemd/user/`), `docs/PROTOCOLE-DEMON.md`, `docs/ARCHITECTURE-IMPLEMENTEE.md`, `README.md` si le démon y est cité, `docs/dev/DECISIONS-AGENT.md` (AG-92 avec les cinq décisions ci-dessus, les limites et le retour arrière).

**Unité :** `[Service] Type=simple ExecStart=%LIBDIR%/mira/mira-daemon Restart=on-failure RestartSec=5` ; `[Install] WantedBy=default.target`. AppImage, flatpak, snap : aucune modification (démon à la demande).

- [ ] **Étape 1 : test rouge** `packaging/tests/units_and_payload.sh` : `systemd-analyze --user verify` sur l'unité ; l'arbre installé par `install-tree.sh` (dans un dossier temporaire) contient `mira-daemon` exécutable et l'unité ; l'unité ne référence aucun chemin hors `%LIBDIR%`.
- [ ] **Étape 2–4 :** rouge, implémenter, vert.
- [ ] **Étape 5 : commit** `feat(packaging): mira-daemon et unité systemd --user dans deb, rpm, arch et tarball` puis `docs(daemon): phase 4 (protocole, architecture implémentée, AG-92)`.

### Tâche 14 : jetons OAuth absents des trames

**Files :** Modify `mira-daemon/tests/secrets_in_frames.rs` (ou Create `tests/secrets_in_frames_oauth.rs`), `tests/support/` si le scénario OAuth de bout en bout contre le testkit demande un pilote.

- [ ] **Étape 1 : test rouge** : `oauth_tokens_never_appear_in_any_frame` — scénario complet contre le serveur OAuth du testkit (ajout de compte, rafraîchissement du jeton), toutes les trames capturées (aller et retour, journal rejoué compris) ne contiennent ni jeton d'accès, ni jeton de rafraîchissement, ni code d'autorisation, ni `client_secret`.
- [ ] **Étape 2–4 :** rouge si une fuite existe (la corriger à la racine, avec test de non-régression d'abord), sinon vert d'emblée.
- [ ] **Étape 5 : commit** `test(daemon): aucun jeton OAuth dans les trames`.

---

## Autocontrôle

- **Couverture de la spec :** §3 → tâches 1, 2, 3, 4, 12 ; §4 présence → 5 ; réglages → 6, 11 ; notifieur → 7, 8, 11 ; §5 icône → 9 ; §6 paquets → 13 ; §7 tests → intégrés à chaque tâche ; dette OAuth → 14 ; dette `set_poll_secs` → tâche 1 (décision 1).
- **Écarts à la spec (consignés ci-dessus) :** pas de `set_poll_secs`, pas de clé `resident` ni de niveau de journal dans `daemon.toml`, clic = relance de l'app et non ouverture du message.
- **Cohérence des noms :** `HostOp::{Shutdown, RetryUnlock, SetPresence}`, `Presence::{set, remove, is_active, subscribe}`, `NotificationSettings`, `NotificationSink::show`, `LocalClock::hour`, `Tray::set_status` sont les mêmes d'une tâche à l'autre.
- **Ordre :** 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12 → 13, 14 libre. Les tâches 5 et 6 sont indépendantes mais touchent le pont : exécution séquentielle (pas de parallélisme, exécution native).
