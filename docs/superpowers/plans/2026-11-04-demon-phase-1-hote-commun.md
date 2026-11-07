# Démon, phase 1 : hôte commun — Plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extraire la logique du pont FFI dans un crate `mira-host` du workspace moteur, sans aucun changement de comportement visible par Flutter, et poser les écrits de la phase (inventaire des chemins locaux, clôture de D-30).

**Architecture:** Un objet `Host` possède le moteur (fini le `static` du pont) et porte `open` / `execute` / `close` / flux d'événements / flux de chat / HTML d'origine. Le pont garde une seule instance globale de `Host` et ses fonctions FRB ; ses modules deviennent des ré-exportations, de sorte que `bridge/tests` ne change pas. Le futur démon (phase 2) créera sa propre instance.

**Tech Stack:** Rust 1.99 (édition 2024), tokio, serde_json, mail-parser, ring, base64 ; `mira-assembly` (façade `Engine`) ; `mira-testkit` en dev.

**Spec:** `docs/superpowers/specs/2026-11-04-architecture-demon-design.md` (§2 découpage, §8 chemins locaux, §10 phase 1).

## Global Constraints

- Contrat JSON, enveloppes (`success`/`failure`/`internal_failure`) et signatures FRB **strictement inchangés** ; `bridge/src/frb_generated.rs` et `app/lib/src/rust/` ne doivent pas changer d'un octet.
- Normes (`tools/check-norms`) : fonction ≤ 30 lignes, fichier ≤ 6 fonctions avec corps, aucun commentaire dans un corps de fonction, une seule ligne `///` en français avant chaque `fn`, `//!` de 1 à 3 lignes en tête de fichier, tests dans des fichiers séparés.
- Pas de `unsafe`, pas de `unwrap`/`expect`/`panic!` hors tests, aucun secret dans journaux, erreurs ou enveloppes.
- `mira-host` ne nomme aucun provider ; il ne dépend que de `mira-assembly` (et de bibliothèques génériques), jamais directement d'un crate `mira-provider-*` ou `mira-ai-*`.
- Les tests de `bridge/tests` ne sont ni modifiés ni affaiblis (aucun changement, même d'import, n'est prévu).
- Tests ciblés (`cargo test -p mira-host --test <nom>`, `cargo test --test <nom>` dans `bridge/`). **Ne jamais lancer `./ci.sh` ni la suite moteur complète sans l'accord explicite de l'utilisateur** (consigne en mémoire).
- Commits : un par tâche, `feat(host): …` ou `docs(host): …`, **sans trailer `Co-Authored-By`** (consigne de l'utilisateur en mémoire), fichiers nommés dans `git add` (jamais un dossier entier). Branche : `feat/E-01-host-commun` créée depuis `dev`.

## Review Focus

- Deux `Host` dans un même processus ne partagent aucun état (le démon et les tests en dépendent) → test en tâche 1.
- `execute` / `close` / flux sur un hôte jamais ouvert : enveloppe d'échec fixe `moteur non ouvert`, jamais de panique → tâches 1 et 3.
- `close` pendant qu'un appel tient encore le moteur : échec `moteur encore utilisé`, puis hôte vide (comportement actuel à conserver) → tâche 1.
- Seconde ouverture du même profil sur le même hôte : refusée (verrou D-27), le premier moteur continue de répondre → tâche 1.
- Surface visible par Dart inchangée (FRB généré, tests du pont) → tâche 4.

---

### Task 1: Crate `mira-host` et objet `Host`

**Files:**
- Create: `engine/crates/mira-host/Cargo.toml`, `src/lib.rs`, `src/host.rs`, `src/lifecycle.rs`, `src/envelope.rs` (copie provisoire, déplacée en tâche 2)
- Create: `engine/crates/mira-host/tests/support/mod.rs`, `tests/host_lifecycle.rs`
- Modify: `tools/check-architecture` (règle A10), `tools/test-check-architecture`

**Interfaces:**
- Produces (`mira_host::Host`) :
  - `Host::new() -> Host` (`const fn`) et `impl Default`
  - `Host::engine(&self) -> Option<Arc<Engine>>`
  - `Host::open(&self, config: EngineConfig) -> String` (enveloppe)
  - `async Host::execute(&self, request_json: &str) -> String`
  - `async Host::close(&self) -> String`
  - `mira_host::envelope::{success, failure, internal_failure}` (mêmes signatures qu'aujourd'hui)
- `Cargo.toml` : dépend de `mira-assembly`, `serde_json`, `tokio` (workspace) ; dev : `mira-testkit`, `tempfile`, `tokio`.

- [ ] **Step 1: Règle A10 et son auto-test.** Dans `tools/check-architecture`, ajouter A10 : `mira-host` ne dépend (normal et dev) d'aucun crate `mira-provider-*` ni `mira-ai-*`. Dans `tools/test-check-architecture`, ajouter `sabotage; echo 'mira-provider-gmail.workspace = true' >> "$tmp/engine/crates/mira-host/Cargo.toml"; expect A10`. Lancer `tools/test-check-architecture` : attendu ÉCHEC (crate absent), c'est le rouge.
- [ ] **Step 2: Écrire `tests/host_lifecycle.rs`** (config de test dans `tests/support/mod.rs` : `fn config(home: &Path) -> EngineConfig` bâtie sur `mira_assembly::production_config_with_oauth`, avec `secrets = Arc::new(MemorySecretStore::default())`, `environment.mira_home = Some(home)`, `network.background_sync = BackgroundSync::Off` ; modèle : `config()` de `bridge/tests/engine_bridge.rs:56`). Tests, noms et assertions :
  - `a_host_never_opened_answers_with_a_fixed_failure` : `Host::new().execute("{}")` et `.close()` ; le premier donne `ok == false` et `error.message == "moteur non ouvert"`, le second `ok == true`.
  - `two_hosts_do_not_share_their_engine` : ouvrir `a` sur le profil `a` ; `b.engine().is_none()` et `b.execute(…)` répond « moteur non ouvert ».
  - `a_second_open_of_the_same_profile_is_refused_and_the_first_keeps_working` : deux `Host` sur le même `home` et le même profil ; le second `open` renvoie `ok == false` ; une requête `{"domain":"account","request":{"op":"list_accounts"}}` sur le premier renvoie `ok == true`.
  - `close_while_the_engine_is_in_use_fails_then_leaves_the_host_empty` : garder `host.engine()` ; `close()` donne `ok == false` avec `moteur encore utilisé` ; ensuite `execute` donne « moteur non ouvert ».
- [ ] **Step 3: Lancer** `cd engine && cargo test -p mira-host --test host_lifecycle` : attendu FAIL (crate absent).
- [ ] **Step 4: Implémenter.** `host.rs` : `struct Host { slot: Mutex<Option<Arc<Engine>>> }`, `new`, `engine`, et deux méthodes `pub(crate)` `install(&self, Arc<Engine>) -> Option<Arc<Engine>>` et `take(&self) -> Option<Arc<Engine>>`, tolérant un mutex empoisonné comme `bridge/src/session.rs`. `lifecycle.rs` : `impl Host { open, execute, close }` reprenant mot pour mot la logique de `bridge/src/runtime.rs` (fermeture forcée de l'ancien moteur au remplacement, `shutdown_within(5 s)`). `envelope.rs` : copie de `bridge/src/envelope.rs`.
- [ ] **Step 5: Lancer** les tests de l'étape 3 et `tools/test-check-architecture` : attendu PASS. Puis `cargo clippy -p mira-host --all-targets -- -D warnings`, `cargo fmt --all --check`, `tools/check-norms engine/crates/mira-host`.
- [ ] **Step 6: Commit** — `feat(host): objet Host qui possède le moteur (D-30)`.

### Task 2: Modules sans état déplacés tels quels

**Files:**
- Move (`git mv`) : `bridge/src/{polling,platform,file_secrets}.rs` → `engine/crates/mira-host/src/` ; supprimer `bridge/src/envelope.rs` (déjà copié)
- Move : `bridge/tests/{file_secrets,polling}.rs` **restent** dans le pont (inchangés)
- Modify: `engine/crates/mira-host/{Cargo.toml,src/lib.rs}`, `bridge/{Cargo.toml,src/lib.rs}`

**Interfaces:**
- Produces : `mira_host::{polling::{supervisor_for, MIN_POLL_SECS, MAX_POLL_SECS}, platform::adapt, file_secrets::FileSecretStore}` aux signatures actuelles ; `mira_bridge::{envelope, polling, platform, file_secrets}` redeviennent des ré-exportations (`pub use mira_host::{…};` dans `bridge/src/lib.rs`) pour que `crate::envelope::…` et les imports des tests restent valides.
- Consumes : tâche 1.

- [ ] **Step 1: Ajouter à `mira-host`** les dépendances `ring` et `base64`/`mail-parser` (workspace) selon ce que les fichiers déplacés utilisent ; déclarer les modules dans `lib.rs`.
- [ ] **Step 2: `git mv`** des trois fichiers, remplacer les imports `crate::…` qui visaient `session` ; retirer les modules déplacés de `bridge/src/lib.rs` et ajouter la ré-exportation ; retirer `ring` du `Cargo.toml` du pont.
- [ ] **Step 3: Vérifier** `cd bridge && cargo test --test file_secrets --test polling` : PASS sans modifier ces fichiers de test ; `cd engine && cargo clippy -p mira-host --all-targets -- -D warnings` ; `tools/check-norms engine/crates/mira-host` (les fichiers satisfaisaient déjà les normes via `make norms`).
- [ ] **Step 4: Commit** — `feat(host): trousseau chiffré, réglages de relevé et adaptation de plateforme dans mira-host`.

### Task 3: Flux et HTML d'origine rattachés à `Host`

**Files:**
- Move : `bridge/src/{chat_feed,events_feed,original_html}.rs` → `engine/crates/mira-host/src/`
- Modify: `engine/crates/mira-host/src/{lib.rs,lifecycle.rs}` ; `Cargo.toml` (mail-parser, base64)
- Create: `engine/crates/mira-host/src/streams.rs` (méthodes de `Host` pour les flux)
- Test: `engine/crates/mira-host/tests/host_streams.rs`

**Interfaces:**
- Produces (fonctions libres, moteur passé en paramètre) :
  - `chat_feed::pump_chat(engine: Option<Arc<Engine>>, request_json: &str, emit: impl FnMut(String) -> bool)` (async)
  - `chat_feed::closing_line(finished: bool, open: bool) -> Option<String>` (inchangé)
  - `events_feed::pump_events(engine: Option<Arc<Engine>>, after: Option<u64>, emit: impl FnMut(String) -> bool)` (async)
  - `original_html::html_of_raw(raw: &[u8]) -> Option<String>` (inchangé) et `original_html::fetch(engine: Option<Arc<Engine>>, message: &str) -> String` (async)
  - Méthodes `Host::pump_events`, `Host::pump_chat`, `Host::original_html` qui passent `self.engine()`.
- Comportement à conserver tel quel : sans moteur, `pump_events` retourne sans rien émettre ; `pump_chat` émet **une seule** ligne `{"type":"error","error":{"code":"internal.bug","message":"moteur non ouvert"}}` ; `fetch` renvoie l'échec interne `moteur non ouvert`.

- [ ] **Step 1: Écrire `tests/host_streams.rs`** — `events_on_a_closed_host_emit_nothing`, `chat_on_a_closed_host_emits_one_fixed_error_line` (vérifie aussi que la ligne ne contient pas le texte de la requête envoyée), `original_html_on_a_closed_host_fails_with_a_fixed_message`.
- [ ] **Step 2: Lancer** `cargo test -p mira-host --test host_streams` : FAIL.
- [ ] **Step 3: Déplacer** les trois fichiers (`git mv`), remplacer `current()` par le paramètre `engine`, ajouter les méthodes de `Host` dans `streams.rs`.
- [ ] **Step 4: Lancer** `host_streams` : PASS ; clippy, fmt, `tools/check-norms engine/crates/mira-host` (attention à `chat_feed.rs` : déjà à 6 fonctions, ne rien y ajouter).
- [ ] **Step 5: Commit** — `feat(host): flux d'événements, flux de chat et HTML d'origine portés par Host`.

### Task 4: Le pont ne garde que ses enveloppes FRB

**Files:**
- Modify: `bridge/src/{session.rs,runtime.rs,lib.rs,api/engine.rs}`, `bridge/Cargo.toml` ; remplacer `bridge/src/{chat_feed,events_feed,original_html}.rs` par des fichiers d'une ligne de ré-exportation (`pub use mira_host::chat_feed::closing_line;`, etc.) ou des fonctions fines sur l'instance globale
- Test: `bridge/tests/*` (inchangés)

**Interfaces:**
- Consumes : `Host` et les flux des tâches 1 à 3.
- Produces (à conserver pour les tests du pont) : `mira_bridge::runtime::{open, execute, close, pump_events, pump_chat}` aux signatures actuelles ; `mira_bridge::original_html::fetch(message: &str) -> String` ; `mira_bridge::chat_feed::closing_line` ; `mira_bridge::{polling, file_secrets, envelope, platform}`.
- `session.rs` devient `static HOST: Host = Host::new();` et `pub fn host() -> &'static Host`.

- [ ] **Step 1: Réécrire** `session.rs` et `runtime.rs` (fonctions fines : `host().open(config)`, etc.), ajuster `api/engine.rs` si un chemin d'import change, retirer de `bridge/Cargo.toml` `mail-parser`, `base64` et `ring` s'ils ne servent plus ; ajouter `mira-host = { path = "../engine/crates/mira-host" }`.
- [ ] **Step 2: Tests du pont**, un fichier à la fois : `cd bridge && cargo test --test engine_bridge`, puis `chat_stream`, `chat_stream_draft`, `chat_stream_ending`, `chat_stream_errors`, `chat_stream_masked`, `chat_stream_tools`, `file_secrets`, `local_labels_bridge`, `polling`, `rules_bridge` : tous PASS, sans modification de ces fichiers (`git diff --stat bridge/tests` vide).
- [ ] **Step 3: Surface Dart** : `git diff --stat bridge/src/frb_generated.rs app/lib/src/rust` vide ; `cd bridge && cargo clippy --all-targets -- -D warnings` ; `tools/check-norms $(find bridge/src -name '*.rs' ! -name frb_generated.rs)`.
- [ ] **Step 4: Construction** : `cd bridge && cargo build` réussit (la bibliothèque `cdylib` se lie toujours).
- [ ] **Step 5: Commit** — `feat(bridge): le pont délègue à mira-host (aucun changement de contrat)`.

### Task 5: Inventaire des chemins locaux du contrat

**Files:**
- Create: `docs/dev/INVENTAIRE-CHEMINS-LOCAUX.md`

**Interfaces:** produit la liste que la spec v4 consommera (§8) ; aucun code.

- [ ] **Step 1: Relever** les requêtes et réponses du contrat qui portent un chemin ou une destination de fichier : `grep -rnE "path|destination|PathBuf" engine/crates/mira-core/src/api engine/crates/mira-vocabulary/src`. Points déjà repérés : `ExportReport.path` (`api/contract/read_responses.rs:29`), `ComposeRequest::AttachFile { path }` (`write/compose_requests.rs:35`), `handlers/message/save_attachment.rs` (`destination`), `handlers/maintenance/diagnostic.rs`, `handlers/maintenance/profile_export.rs`. Vérifier aussi les imports et exports de règles et de libellés (côté app : `app/lib/ui/rules_file_flow.dart`, `labels_file_bar.dart`, `download_all_button.dart`, `data/attachment_save.dart`) pour savoir si le fichier passe par le moteur ou par l'app seule.
- [ ] **Step 2: Écrire** `INVENTAIRE-CHEMINS-LOCAUX.md` : un tableau (opération du contrat · champ · sens · qui touche le disque, moteur ou app · remplacement proposé pour un moteur distant : envoi ou téléchargement par flux, ou contenu en ligne). Chaque entrée du `grep` y figure, ou y est écartée avec la raison.
- [ ] **Step 3: Vérifier** qu'aucun chemin trouvé par le `grep` de l'étape 1 n'est absent du tableau, puis commit — `docs(host): inventaire des chemins de fichiers du contrat (préparation du moteur distant)`.

### Task 6: Clôture de D-30 et documentation

**Files:**
- Modify: `docs/dev/PROPOSITIONS-REFERENCE.md` (ajouter PR-21), `docs/dev/DECISIONS-AGENT.md` (ajouter AG-88), `docs/ARCHITECTURE-IMPLEMENTEE.md`
- Modify: `docs/dev/REFLEXION-ARCHITECTURE-DEMON.md` (ligne de statut : remplacée par la spec)

- [ ] **Step 1: PR-21** au format des entrées existantes (Contexte / Proposition / Raison / Renvoi) : D-30 devient « les deux » — moteur embarqué (Android, iOS, repli) et processus séparé (bureau) ; contrat identique ; moteur distant visé pour la version 4, sur le serveur choisi par l'utilisateur. Renvoi : la spec du 2026-11-04. Ne **jamais** modifier `docs/MIRA-architecture-reference.md`.
- [ ] **Step 2: AG-88** au format des décisions (Contexte / Choix / Raison / Retour arrière / Limites) : choix de `Host` à la place du `static`, ré-exportations du pont pour garder `bridge/tests` intacts, règle A10, abandon du spike, TCP loopback + jeton sur Windows. Retour arrière : supprimer `mira-host` et rétablir les fichiers du pont (`git revert` des commits de la phase).
- [ ] **Step 3: `ARCHITECTURE-IMPLEMENTEE.md`** : ajouter `mira-host` au schéma des crates et à la description du pont (une dizaine de lignes).
- [ ] **Step 4: Commit** — `docs(host): clôture de D-30 (PR-21, AG-88) et description de mira-host`.

### Task 7: Vérification de la phase

- [ ] **Step 1:** `cd engine && cargo fmt --all --check && cargo clippy -p mira-host --all-targets -- -D warnings && cargo test -p mira-host`.
- [ ] **Step 2:** `tools/check-architecture engine && tools/test-check-architecture && tools/check-norms engine/crates` ; `make norms` si les cibles locales le permettent.
- [ ] **Step 3:** la suite du pont (tâche 4, étape 2) repasse en entier.
- [ ] **Step 4:** demander à l'utilisateur s'il autorise `./ci.sh` (et la suite moteur complète) avant de proposer la fusion de `feat/E-01-host-commun` dans `dev`. Ne pas le lancer sans accord.
