# Démon, phase 3 : client Dart — Plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** L'app Flutter peut parler au démon `mira-daemon` (phases 1–2) à la place du moteur embarqué : sur Linux elle s'attache à un démon vivant, le démarre sinon, et repasse au moteur embarqué en dernier recours ; Android, iOS et le web gardent le pont FFI.

**Architecture:** `EngineClient` (interface haute) reste inchangée. Une nouvelle implémentation `RemoteEngineClient` parle le protocole de `docs/PROTOCOLE-DEMON.md` au-dessus d'une interface basse `ByteChannel` (socket Unix ou TCP via `dart:io`). Une `RemoteConnection` porte la poignée de main, l'appariement des `result` par `id`, les flux et le maintien de la connexion. Une couche de choix (`openEngineClient`) applique la politique démon / embarqué et traduit les états de l'hôte en messages lisibles. Les trames sont vérifiées contre un fichier de vecteurs partagé avec le crate Rust `mira-protocol`.

**Tech Stack:** Dart 3 / Flutter 3.47 (`dart:io`, `dart:async`, `dart:convert`), `fake_async` et `flutter_test` (déjà en dépendances), Rust (un seul test ajouté dans `mira-protocol`), `mira-daemon` (binaire existant).

**Spec:** `docs/superpowers/specs/2026-11-04-architecture-demon-design.md` (§3.2 abstraction Dart, §4 protocole, §5 `host_state`, §7 cycle de vie, §9 tests Dart, §10 phase 3) ; format des trames : `docs/PROTOCOLE-DEMON.md`.

**Branche :** `feat/E-01-client-dart`, créée depuis `dev` (phases 1 et 2 fusionnées).

## Global Constraints

- Contrat JSON, `CONTRACT_VERSION` et interface `EngineClient` / `ChatStreamingClient` : **aucune modification incompatible** (ajouts permis). `lib/src/rust/` et `bridge/` ne changent pas.
- Android, iOS et le web (démonstration) gardent exactement leur chemin actuel (`RustEngineClient` ou démonstration) ; seul Linux de bureau peut choisir le démon. `MIRA_DEMO=1` reste prioritaire.
- Format des trames et règles du protocole : ceux de `docs/PROTOCOLE-DEMON.md`. En particulier : longueur sur 4 octets grand-boutistes, `MAX_FRAME_BYTES = 16 Mio` (taille jugée sur l'en-tête avant de lire le corps), `hello` dans les 5 s et d'au plus 8 Kio, un `id` ne se réutilise pas tant que son échange est en cours, une trame de **type inconnu reçue du démon est ignorée** (compatibilité ascendante), jamais fatale.
- Le jeton local n'est jamais écrit dans un journal, une erreur, un message d'interface ni sur stdout/stderr ; son fichier est refusé s'il est lisible par le groupe ou les autres (`mode & 0o077 != 0`).
- `client_id` stable par installation (aléa de 128 bits en hexadécimal, gardé dans `SharedPreferences`, clé `engineClientId`) ; `client_name` : `mira-app`.
- Fermer l'app **détache** le client (`EngineClient.close()` ferme la connexion) ; cela n'arrête jamais le démon (arrêt = `shutdown()`, réservé).
- Dart : `flutter analyze` sans aucun diagnostic ; docs `///` en français avant chaque déclaration publique ; pas de `print` ; tout texte affiché passe par `t()` avec son entrée anglaise dans `lib/i18n/lang/en.dart` (`tools/i18n-audit` : 0 manquant) ; les messages français non affichés portent `// i18n:ignore` comme dans le code existant.
- Tests ciblés (`flutter test test/remote/<fichier>`, `cargo test -p mira-protocol --test golden`). **Ne jamais lancer `./ci.sh`, `make check-all` ni la suite Flutter complète sans l'accord explicite de l'utilisateur.**
- Commits : un par tâche, `feat(client): …`, `test(client): …` ou `docs(client): …`, **sans trailer `Co-Authored-By`**, fichiers nommés dans `git add`.

## Review Focus

- Trames découpées en morceaux arbitraires (un octet à la fois), plusieurs trames dans un seul morceau, taille annoncée énorme ou nulle : jamais de blocage ni d'allocation avant le contrôle → tâche 1.
- Flux d'événements de plus de 256 trames : le client réapprovisionne les crédits et ne se bloque pas ; après `end lagged` ou une coupure, resouscription avec `after` = dernier `seq` reçu, sans perte ni doublon → tâche 5.
- Annuler l'abonnement d'un chat envoie `cancel` ; fermer l'app (`close()`) ne ferme pas le moteur du démon → tâches 4 et 6.
- Coupure de la connexion au milieu d'un appel : la `Future` échoue avec une `EngineException` lisible, sans blocage ; l'appel suivant se reconnecte → tâche 4.
- Démon absent, `starting`, `keyring_locked`, `update_required`, jeton refusé ou fichier de jeton trop ouvert : aucun plantage, un message clair ; repli sur le moteur embarqué uniquement quand le démon est **absent** → tâches 7 et 8.
- Le jeton n'apparaît dans aucun message, exception ni `toString` → tâches 3 et 7.

---

### Task 1: Trames : vecteurs partagés, encodeur et décodeur Dart

**Files:**
- Create: `engine/crates/mira-protocol/tests/golden/frames.json`, `engine/crates/mira-protocol/tests/golden.rs`
- Create: `app/lib/engine/remote/frames.dart`, `app/lib/engine/remote/frame_builders.dart`
- Test: `app/test/remote/frames_test.dart`

**Interfaces:**
- Produces (`frames.dart`) :
  - `const int maxFrameBytes = 16 * 1024 * 1024;`
  - `class ProtocolException implements Exception { final String reason; }` (`'too_large'`, `'empty'`, `'malformed'`)
  - `Uint8List encodeFrame(Map<String, dynamic> frame, {int max = maxFrameBytes})` : quatre octets de longueur grand-boutistes puis le JSON UTF-8 ; lève `ProtocolException('too_large')` si le corps dépasse `max`.
  - `class FrameDecoder { FrameDecoder({int max = maxFrameBytes}); List<Map<String, dynamic>> add(List<int> chunk); }` : décodeur incrémental ; rend les trames complètes contenues dans le morceau (zéro, une ou plusieurs) ; lève `ProtocolException` sur taille annoncée > `max` (dès l'en-tête complet, sans attendre le corps), taille 0, ou corps qui n'est pas un objet JSON.
- Produces (`frame_builders.dart`), un constructeur par trame cliente, champs dans l'ordre du Rust (`type` d'abord) : `helloFrame({required String clientId, required String clientName, required String token, Map<String,int> protocolVersion, Map<String,int> contractVersion})`, `callFrame(int id, Map<String,dynamic> request)`, `openEventsFrame(int id, {int? after, required int credit})`, `openChatFrame(int id, Map<String,dynamic> request, {required int credit})`, `getOriginalHtmlFrame(int id, String message)`, `hostOpShutdownFrame(int id)`, `creditFrame(int id, int n)`, `cancelFrame(int id)`, `pingFrame(int nonce)`. Constantes `clientProtocolVersion = {'major': 1, 'minor': 0}` et `appContractVersion = {'major': 1, 'minor': 6}`.

Le fichier de vecteurs `frames.json` est une liste d'objets `{name, frame, hex}` : `frame` est la trame en JSON, `hex` les octets exacts produits par `mira_protocol::encode` (longueur comprise), pour une trame de **chacun des 16 types**.

- [ ] **Step 1: Test Rust des vecteurs.** `engine/crates/mira-protocol/tests/golden.rs` : `golden_vectors_match_the_encoder` (pour chaque entrée, `serde_json::from_value::<Frame>(frame)` puis `encode(&frame, MAX_FRAME_BYTES)` donne exactement `hex`) et `golden_vectors_cover_every_frame_type` (les 16 noms de `type` y figurent). Créer d'abord `frames.json` avec des `hex` vides : lancer `cd engine && cargo test -p mira-protocol --test golden` → FAIL.
- [ ] **Step 2:** Remplir `hex` en lisant la sortie du test en échec (jamais à la main), relancer → PASS.
- [ ] **Step 3: Écrire `app/test/remote/frames_test.dart`.** Noms et assertions : `dart_encodes_every_golden_frame_to_the_rust_bytes` (pour chaque vecteur, `encodeFrame` donne les octets de `hex` après décodage de l'en-tête de longueur ; les clés sont comparées après `jsonDecode`, l'ordre des champs étant celui des constructeurs) ; `dart_decodes_every_golden_hex` ; `a_frame_split_one_byte_at_a_time_decodes_once` ; `two_frames_in_one_chunk_decode_in_order` ; `a_partial_header_yields_nothing_until_complete` ; `an_announced_size_above_the_maximum_fails_from_the_header_alone` (`FrameDecoder(max: 1024)`, en-tête de 1 Mio seul : lève sans corps) ; `a_zero_length_frame_fails` ; `a_body_that_is_not_an_object_fails` ; `encode_refuses_a_body_above_the_maximum` ; `builders_produce_the_documented_fields` (un par constructeur : `type` et champs du tableau de `PROTOCOLE-DEMON.md`, `after: null` présent pour `open_events` sans reprise). Lancer `cd app && flutter test test/remote/frames_test.dart` → FAIL (fichiers absents).
- [ ] **Step 4: Implémenter** `frames.dart` et `frame_builders.dart`. Le décodeur garde un tampon d'octets et ne copie le corps qu'une fois complet.
- [ ] **Step 5: Lancer** `flutter test test/remote/frames_test.dart` → PASS, puis `flutter analyze` → aucun diagnostic.
- [ ] **Step 6: Commit** — `feat(client): trames du protocole du démon en Dart, vérifiées contre les vecteurs Rust`.

---

### Task 2: Canal d'octets : socket Unix et TCP

**Files:**
- Create: `app/lib/engine/remote/byte_channel.dart`
- Create (support de test): `app/test/remote/support/memory_channel.dart`
- Test: `app/test/remote/byte_channel_test.dart`

**Interfaces:**
- Produces :
  - `abstract interface class ByteChannel { Stream<List<int>> get incoming; void send(List<int> bytes); Future<void> close(); Future<void> get done; }` (`done` aboutit à la fermeture, quel qu'en soit l'auteur).
  - `class SocketChannel implements ByteChannel` avec `static Future<SocketChannel> connectUnix(String path, {Duration timeout})` et `static Future<SocketChannel> connectTcp(String host, int port, {Duration timeout})` (délai par défaut 5 s ; échec = `SocketException` ou `TimeoutException` telle quelle).
  - Support de test : `MemoryChannel.pair()` rend deux `ByteChannel` reliés (ce qu'un côté envoie, l'autre le reçoit ; fermer l'un ferme l'autre ; livraison asynchrone, jamais synchrone dans `send`).

- [ ] **Step 1: Écrire `byte_channel_test.dart`.** `memory_channel_delivers_bytes_both_ways_and_close_propagates` ; `tcp_channel_exchanges_bytes_with_a_loopback_server` (`ServerSocket.bind('127.0.0.1', 0)`) ; `unix_channel_exchanges_bytes_with_a_unix_server` (socket dans `Directory.systemTemp`, `@TestOn('linux')` sur ce test) ; `connecting_to_a_missing_unix_socket_fails_with_a_socket_exception` ; `connecting_to_a_closed_tcp_port_fails` ; `closing_completes_done_and_a_second_close_is_harmless`. Lancer → FAIL.
- [ ] **Step 2: Implémenter** `byte_channel.dart` et `memory_channel.dart`. `SocketChannel` enveloppe un `Socket` (`InternetAddress(path, type: InternetAddressType.unix)` pour Unix) ; `send` fait `socket.add`.
- [ ] **Step 3: Lancer** le test, `flutter analyze` → PASS / aucun diagnostic.
- [ ] **Step 4: Commit** — `feat(client): canal d'octets (socket Unix et TCP loopback) et canal mémoire de test`.

---

### Task 3: Connexion : poignée de main, appariement, maintien

**Files:**
- Create: `app/lib/engine/remote/connection.dart`, `app/lib/engine/remote/host_state.dart`
- Create (support de test): `app/test/remote/support/fake_daemon.dart`
- Test: `app/test/remote/connection_test.dart`

**Interfaces:**
- Consumes: `ByteChannel`, `FrameDecoder`, `encodeFrame`, constructeurs de trames (tâches 1–2).
- Produces :
  - `enum HostState { ready, starting, keyringLocked, updateRequired, unknown }` avec `HostState.parse(String?)` (`'ready'` … ; toute autre valeur : `unknown`).
  - `class HandshakeRefused implements Exception { final String reason; }` (`'protocol_version'`, `'contract_version'`, `'auth'`, `'malformed'`) ; `toString` sans jeton.
  - `class ConnectionLost implements Exception { final String why; }`
  - `class RemoteConnection` :
    - `static Future<RemoteConnection> open(ByteChannel channel, {required String token, required String clientId, required String clientName, Duration helloTimeout = const Duration(seconds: 5), Duration pingInterval = const Duration(seconds: 15)})` : envoie `hello`, attend `hello_ok` ; `hello_err` lève `HandshakeRefused`, absence de réponse dans `helloTimeout` ou fermeture lève `ConnectionLost` ; ferme le canal dans ces cas.
    - `HostState get hostState`, `Map<String,dynamic> get serverVersions` (versions annoncées).
    - `int nextId()` (entiers croissants à partir de 1).
    - `Future<Map<String,dynamic>> request(Map<String,dynamic> frame)` : envoie une trame portant `id` et rend la trame `result` de même `id` ; échoue avec `ConnectionLost` si la connexion se ferme avant.
    - `Stream<Map<String,dynamic>> openStream(Map<String,dynamic> openFrame)` : envoie la trame d'ouverture (`open_events`, `open_chat`) et rend un flux brut de toutes les trames du même `id` (`event`, `chat_delta`, `end` — le flux se ferme après `end`) ; erreur `ConnectionLost` si la connexion se perd avant `end`.
    - `void send(Map<String,dynamic> frame)` (crédit, annulation) ; `Future<void> get closed` ; `Future<void> close()`.
    - Maintien : `ping` toutes les `pingInterval` ; toute trame reçue prouve la vie ; plus rien reçu pendant `3 × pingInterval` ferme la connexion (`ConnectionLost('silence')`). Les trames de type inconnu sont ignorées.
  - Support de test `FakeDaemon` : `FakeDaemon.pair({String token, HostState state, ...})` rend `(ByteChannel client, FakeDaemon server)` ; le faux démon vérifie `hello` (jeton, versions) et répond `hello_ok` / `hello_err` comme le vrai, expose `List<Map> received`, `Future<Map> next(String type)`, `void reply(Map)` / `void push(Map)`, `void close()`, `void stayMute()` (n'écrit plus rien).

- [ ] **Step 1: Écrire `connection_test.dart`.** `a_valid_handshake_reports_versions_and_host_state` ; `a_wrong_token_is_refused_with_reason_auth_and_the_channel_closes` ; `incompatible_versions_are_refused_with_their_reason` (`protocol_version`, `contract_version`) ; `a_silent_daemon_times_out_the_handshake` (`fake_async`) ; `the_token_never_appears_in_exception_text` (`toString` de `HandshakeRefused` et `ConnectionLost`) ; `results_are_matched_by_id_even_out_of_order` (trois requêtes, réponses inversées) ; `pending_requests_fail_with_connection_lost_when_the_channel_closes` ; `unknown_frame_types_are_ignored` ; `a_stream_delivers_its_frames_then_closes_after_end` ; `frames_of_another_id_never_reach_a_stream` ; `a_ping_is_sent_every_interval_and_a_pong_keeps_the_connection_alive` (`fake_async`) ; `three_silent_intervals_close_the_connection` (`fake_async`) ; `ids_grow_from_one`. Lancer → FAIL.
- [ ] **Step 2: Implémenter** `host_state.dart`, `connection.dart` et `fake_daemon.dart`. Chaque `request`/`openStream` est enregistré dans une table `id → Completer / StreamController` ; la fermeture du canal fait échouer tout ce qui reste.
- [ ] **Step 3: Lancer** `flutter test test/remote/connection_test.dart`, `flutter analyze` → PASS / aucun diagnostic.
- [ ] **Step 4: Commit** — `feat(client): connexion au démon (poignée de main, appariement par identifiant, maintien)`.

---

### Task 4: `RemoteEngineClient` : appels, HTML d'origine, détachement, reconnexion

**Files:**
- Create: `app/lib/engine/remote/remote_engine_client.dart`, `app/lib/engine/remote/endpoint.dart`
- Modify: `app/lib/engine/engine_client.dart` (ajout de `Map<String,dynamic> openEnvelopeMap(Map<String,dynamic> envelope)` ; `openEnvelope(String)` y délègue, comportement identique)
- Test: `app/test/remote/remote_calls_test.dart`

**Interfaces:**
- Consumes: `RemoteConnection` (tâche 3), `EngineException`, `openEnvelopeMap`.
- Produces :
  - `class DaemonEndpoint { final String? unixSocket; final String? tcpHost; final int? tcpPort; final String token; }` avec `Future<ByteChannel> connect()` (Unix si `unixSocket` est donné, sinon TCP). `toString` n'affiche jamais le jeton.
  - `class RemoteEngineClient implements EngineClient, ChatStreamingClient` :
    - `static Future<RemoteEngineClient> attach(DaemonEndpoint endpoint, {required String clientId, String clientName = 'mira-app', Future<ByteChannel> Function()? channelFactory, Duration pingInterval, Duration helloTimeout})` : connecte, fait la poignée de main, rend le client ; ne connaît pas encore la politique des états d'hôte (voir tâche 8) mais expose `HostState get hostState`.
    - `call(request)` : `call` + `openEnvelopeMap(result.envelope)` (mêmes `EngineException` que le pont) ; `originalHtml(id)` : `get_original_html`, valeur `html` de l'enveloppe ; `close()` : détache (ferme la connexion, jamais d'`host_op`) ; `Future<void> shutdown()` : `host_op shutdown` puis détache.
    - Coupure : tout appel en vol échoue avec `EngineException('transport.disconnected', …)` (message français, `// i18n:ignore`, sans jeton) ; l'appel suivant tente **une** reconnexion (flux monopolisé : deux appels simultanés ne rouvrent qu'une connexion) et, si elle échoue, lève `EngineException('transport.unavailable', …)`.

- [ ] **Step 1: Écrire `remote_calls_test.dart`** (faux démon). `call_returns_the_response_object_of_the_envelope` (`{type,data}`) ; `an_error_envelope_becomes_an_engine_exception_with_code_and_message` ; `concurrent_calls_are_matched_by_id` ; `original_html_returns_the_html_or_null` ; `close_detaches_without_sending_a_host_op` (le faux démon ne reçoit aucun `host_op`) ; `shutdown_sends_host_op_shutdown_then_detaches` ; `a_call_in_flight_fails_with_transport_disconnected_when_the_daemon_drops` ; `the_next_call_reconnects_once_and_succeeds` (la fabrique de canal est appelée une fois de plus) ; `two_simultaneous_calls_after_a_drop_open_a_single_connection` ; `reconnection_failure_raises_transport_unavailable` ; `a_call_after_close_fails_with_a_clear_error` (pas de reconnexion après `close()`). Lancer → FAIL.
- [ ] **Step 2: Implémenter** `endpoint.dart`, `remote_engine_client.dart`, `openEnvelopeMap`. `engine_client.dart` : `openEnvelope(String json)` devient `openEnvelopeMap(jsonDecode(json) as Map<String,dynamic>)`. Lancer `flutter test test/rust_bridge_test.dart test/engine_startup_error_test.dart` : les tests existants restent verts (ou ignorés pour le pont).
- [ ] **Step 3: Lancer** `flutter test test/remote/remote_calls_test.dart`, `flutter analyze` → PASS / aucun diagnostic.
- [ ] **Step 4: Commit** — `feat(client): RemoteEngineClient (appels, HTML d'origine, détachement, reconnexion)`.

---

### Task 5: Flux d'événements : crédits, `lagged`, reprise

**Files:**
- Create: `app/lib/engine/remote/event_stream.dart`
- Modify: `app/lib/engine/remote/remote_engine_client.dart` (méthode `events`)
- Test: `app/test/remote/remote_events_test.dart`

**Interfaces:**
- Consumes: `RemoteConnection.openStream` / `send`, trames `credit`.
- Produces: `RemoteEngineClient.events({int? after})` : flux à abonné unique d'objets `Event` (`event` de chaque trame `event`). Fenêtre de crédits : ouverture avec `credit: 256` ; chaque fois que 128 événements ont été remis à l'abonné, `credit{n: 128}`. `lastSeq` = `seq` du dernier événement remis. Fin `lagged`, fin `finished`/`cancelled` non demandée, ou coupure de connexion : réouverture avec `after: lastSeq` (ou `after` initial tant qu'aucun événement n'est arrivé), après une attente 0,5 s doublant jusqu'à 5 s, remise à 0,5 s dès qu'un événement arrive ; sans `lastSeq` ni `after`, la réouverture n'a pas de reprise (événements survenus pendant la coupure perdus : limite documentée). Annuler l'abonnement envoie `cancel` et arrête toute reconnexion. `end{error}` est remis comme erreur du flux puis le flux se ferme.

- [ ] **Step 1: Écrire `remote_events_test.dart`** (faux démon, `fake_async` pour les attentes). `events_open_with_a_credit_window_and_deliver_event_objects` ; `credit_is_replenished_every_128_events_so_a_long_stream_never_blocks` (1000 événements poussés par le faux démon, qui ne livre que dans la limite des crédits reçus : tous arrivent) ; `after_is_sent_when_given` ; `a_lagged_end_reopens_with_after_set_to_the_last_seq_without_loss_or_duplicate` (le faux démon coupe après le `seq` 40 avec `lagged`, rejoue depuis `after` : la séquence reçue par l'abonné est 1…100 sans trou ni doublon) ; `a_dropped_connection_reconnects_and_resumes_after_the_last_seq` ; `backoff_doubles_up_to_five_seconds_and_resets_after_an_event` ; `cancelling_the_subscription_sends_cancel_and_stops_reconnecting` ; `an_error_end_is_delivered_as_a_stream_error_then_closes`. Lancer → FAIL.
- [ ] **Step 2: Implémenter** `event_stream.dart` (un `StreamController` à abonné unique, `onCancel` envoie `cancel` et coupe la boucle de reconnexion) et brancher `events`.
- [ ] **Step 3: Lancer** `flutter test test/remote/remote_events_test.dart`, `flutter analyze` → PASS / aucun diagnostic.
- [ ] **Step 4: Commit** — `feat(client): flux d'événements du démon (crédits, lagged, reprise par numéro)`.

---

### Task 6: Flux de chat : crédits et annulation

**Files:**
- Create: `app/lib/engine/remote/chat_stream.dart`
- Modify: `app/lib/engine/remote/remote_engine_client.dart` (méthode `chatStream`)
- Test: `app/test/remote/remote_chat_test.dart`

**Interfaces:**
- Produces: `RemoteEngineClient.chatStream(Map start)` : flux des lignes (`line` de chaque `chat_delta`) : `started`, `text`, `usage`, `done`, `error`, … Ouverture avec `credit: 64`, `credit{n: 32}` toutes les 32 lignes remises. `end{finished}` ferme le flux ; `end{cancelled}` aussi ; `end{error}` remet une `EngineException` puis ferme. **Annuler l'abonnement envoie `cancel`** (et attend `end` sans le remettre). Une coupure de connexion en cours de route remet `EngineException('transport.disconnected', …)` ; **pas de reconnexion** pour un chat.

- [ ] **Step 1: Écrire `remote_chat_test.dart`.** `lines_arrive_in_order_then_the_stream_closes_on_end_finished` ; `credit_is_replenished_every_32_lines` ; `cancelling_the_subscription_sends_cancel` ; `no_line_is_delivered_after_cancel` ; `end_error_becomes_an_engine_exception` ; `a_dropped_connection_mid_chat_raises_transport_disconnected_and_does_not_reconnect` ; `an_unknown_start_object_is_forwarded_as_is` (la demande `ChatStart` n'est pas réinterprétée) ; `the_stream_works_through_the_ChatStreamingClient_interface` (`client is ChatStreamingClient`). Lancer → FAIL.
- [ ] **Step 2: Implémenter** `chat_stream.dart` et brancher `chatStream`.
- [ ] **Step 3: Lancer** `flutter test test/remote/remote_chat_test.dart`, `flutter analyze` → PASS / aucun diagnostic.
- [ ] **Step 4: Commit** — `feat(client): flux de chat du démon (crédits, annulation par cancel)`.

---

### Task 7: Chemins, jeton et lancement du démon

**Files:**
- Create: `app/lib/engine/remote/daemon_paths.dart`, `app/lib/engine/remote/daemon_token.dart`, `app/lib/engine/remote/daemon_launcher.dart`
- Test: `app/test/remote/daemon_paths_test.dart`, `app/test/remote/daemon_token_test.dart`, `app/test/remote/daemon_launcher_test.dart`

**Interfaces:**
- Produces :
  - `class DaemonPaths { final String socket; final String token; }` et `DaemonPaths resolveDaemonPaths({required String profile, required Map<String,String> env, String? home})` : racine du profil comme le moteur (`MIRA_HOME` → `<MIRA_HOME>/profiles/<profil>` ; sinon `XDG_DATA_HOME` → `<XDG_DATA_HOME>/mira/profiles/<profil>` ; sinon `HOME` → `<HOME>/.local/share/mira/profiles/<profil>` ; `home` explicite remplace `MIRA_HOME`) ; `socket` = `<XDG_RUNTIME_DIR>/mira-daemon/<profil>.sock` si `XDG_RUNTIME_DIR` est non vide, sinon `<racine>/daemon.sock` ; `token` = `<racine>/daemon.token`. Lève `ArgumentError` si aucune racine n'est déterminable.
  - `Future<String> readDaemonToken(String path)` : `EngineException.local('transport.token_missing', …)` si absent ; `EngineException.local('transport.token_unsafe', …)` si `FileStat.mode & 0x3F != 0` ; `EngineException.local('transport.token_invalid', …)` si ce n'est pas 64 caractères hexadécimaux ; rend le jeton sans espaces. Aucun message ne contient le jeton ni son contenu.
  - `class DaemonLauncher { DaemonLauncher({ProcessStarter? start, List<String>? binaryCandidates, Map<String,String>? env}); String? findBinary(); Future<void> start({required String profile, String? home, String? socket}); }` : candidats dans l'ordre `MIRA_DAEMON_BIN`, `<dossier de l'exécutable>/mira-daemon`, `<dossier de l'exécutable>/lib/mira-daemon`, `../engine/target/release/mira-daemon`, `../engine/target/debug/mira-daemon` (premier qui existe) ; `start` lance `mira-daemon --profile <profil> [--home <home>]` en mode détaché (`ProcessStartMode.detached`), sans jamais passer le jeton en argument. `typedef ProcessStarter = Future<Process> Function(String exe, List<String> args, {ProcessStartMode mode, Map<String,String>? environment})`.

- [ ] **Step 1: Écrire les trois tests.** `daemon_paths_test.dart` : un test par cas du tableau (`MIRA_HOME` ; `XDG_DATA_HOME` ; `HOME` seul ; `home` explicite ; `XDG_RUNTIME_DIR` présent / vide / absent ; `ArgumentError` sans racine). `daemon_token_test.dart` : `a_private_valid_token_is_read_and_trimmed` ; `a_missing_file_raises_token_missing` ; `a_group_readable_file_raises_token_unsafe` ; `a_malformed_content_raises_token_invalid_without_echoing_it`. `daemon_launcher_test.dart` : `binary_candidates_are_tried_in_order` (fichiers factices dans un dossier jetable) ; `no_binary_found_yields_null` ; `start_runs_the_binary_detached_with_profile_and_home_and_never_the_token` (starter factice qui enregistre les arguments) ; `start_fails_with_a_clear_error_when_the_process_cannot_be_started`. Lancer → FAIL.
- [ ] **Step 2: Implémenter** les trois fichiers.
- [ ] **Step 3: Lancer** les trois tests, `flutter analyze` → PASS / aucun diagnostic.
- [ ] **Step 4: Commit** — `feat(client): chemins du démon, lecture du jeton, lancement détaché`.

---

### Task 8: Choix démon / embarqué, états de l'hôte, branchement dans l'app

**Files:**
- Create: `app/lib/engine/remote/engine_opening.dart`
- Modify: `app/lib/main.dart` (`openEngineSource` passe par `openEngineClient`), `app/lib/i18n/lang/en.dart` (textes anglais des nouveaux messages)
- Test: `app/test/remote/engine_opening_test.dart`, `app/test/engine_startup_error_test.dart` (un cas ajouté)

**Interfaces:**
- Consumes: `RemoteEngineClient.attach`, `resolveDaemonPaths`, `readDaemonToken`, `DaemonLauncher`, `RustEngineClient.open`.
- Produces :
  - `enum EngineMode { auto, embedded, daemon }` et `EngineMode parseEngineMode(String? value)` (`MIRA_ENGINE` : `embedded` | `daemon` | tout autre ou absent : `auto`).
  - `class EngineOpeningDeps` (injection : `attachDaemon`, `launchDaemon`, `openEmbedded`, `isLinuxDesktop`, `env`, `clientId`, `wait`/horloge pour les tests).
  - `Future<EngineClient> openEngineClient({required EngineOpeningDeps deps, EngineMode mode, String profile = 'default', String? home, Uint8List? secretsKey})` avec la politique :
    - hors Linux de bureau (Android, iOS, web, autres) : toujours `openEmbedded` ;
    - `embedded` : `openEmbedded` ;
    - `auto` : s'attacher à un démon vivant ; sinon, si un binaire est trouvé, le lancer puis s'attacher (attente 20 s au plus, essais toutes les 250 ms) ; sinon `openEmbedded` ;
    - `daemon` : comme `auto` sans le repli : l'échec est remonté (`EngineException`) ;
    - état d'hôte après `hello_ok` : `ready` → utiliser ; `starting` → refaire la poignée de main toutes les 250 ms jusqu'à `ready` (30 s au plus, puis `transport.unavailable`) ; `keyringLocked` → `EngineException.local('host.keyring_locked', t(…))` ; `updateRequired` → `EngineException.local('host.update_required', t(…))` ; **ces deux états et un jeton refusé (`auth`, `protocol_version`, `contract_version`) ne déclenchent jamais le repli embarqué** (le repli n'a lieu que si le démon est **absent** ou injoignable) ;
    - le client rendu par le démon expose son mode : `EngineClient` reste inchangé, la fonction rend `({EngineClient client, EngineMode used})`.
  - Textes (français, avec entrée anglaise) : « Le trousseau du système est verrouillé : déverrouillez-le puis relancez MIRA » ; « Le moteur en arrière-plan n'est pas compatible avec cette version de MIRA : redémarrez-le » ; « Accès refusé par le moteur en arrière-plan » ; « Moteur en arrière-plan injoignable ».
  - `client_id` : lu dans `SharedPreferences` (`engineClientId`), généré (128 bits aléatoires sûrs, hexadécimal) et enregistré s'il manque.

- [ ] **Step 1: Écrire `engine_opening_test.dart`** (dépendances factices, `fake_async`). `embedded_mode_never_touches_the_daemon` ; `non_desktop_platforms_always_use_the_embedded_engine` ; `auto_attaches_to_a_live_daemon` ; `auto_launches_the_daemon_when_absent_and_a_binary_exists_then_attaches` ; `auto_falls_back_to_embedded_when_no_daemon_and_no_binary` ; `auto_falls_back_to_embedded_when_the_launched_daemon_never_answers_within_20_seconds` ; `daemon_mode_reports_the_failure_instead_of_falling_back` ; `a_starting_daemon_is_polled_until_ready` ; `a_starting_daemon_that_never_gets_ready_fails_after_30_seconds` ; `keyring_locked_raises_a_localized_exception_and_never_falls_back` ; `update_required_raises_a_localized_exception_and_never_falls_back` ; `a_refused_token_never_falls_back_to_embedded` ; `a_missing_token_file_with_a_live_socket_is_reported_not_hidden` ; `the_client_id_is_generated_once_and_kept` ; `parse_engine_mode_defaults_to_auto`. Ajouter à `engine_startup_error_test.dart` : `une indisponibilité de l'hôte donne un message traduit` (le message de `host.keyring_locked` est affiché par `engineErrorText`). Lancer → FAIL.
- [ ] **Step 2: Implémenter** `engine_opening.dart`, brancher `main.dart` (`openEngineSource` appelle `openEngineClient` avec les dépendances réelles ; `MIRA_DEMO` et le web inchangés ; Android passe `home` et `secretsKey` au chemin embarqué comme avant), ajouter les textes dans `en.dart`.
- [ ] **Step 3: Lancer** `flutter test test/remote/engine_opening_test.dart test/engine_startup_error_test.dart`, `flutter analyze`, puis `tools/i18n-audit` (depuis la racine) → PASS / aucun diagnostic / 0 manquant.
- [ ] **Step 4: Commit** — `feat(client): choix démon ou embarqué, états de l'hôte, branchement dans l'app`.

---

### Task 9: Contre le vrai démon, cible Makefile, documentation

**Files:**
- Create: `app/test/remote/remote_daemon_test.dart`
- Modify: `Makefile` (cible `daemon`), `docs/PROTOCOLE-DEMON.md` (section « Client Dart »), `docs/ARCHITECTURE-IMPLEMENTEE.md`
- Modify (non suivis par git, `docs/dev/` est ignoré) : `docs/dev/DECISIONS-AGENT.md` (AG-90), `docs/dev/REFLEXION-ARCHITECTURE-DEMON.md` (statut)

**Interfaces:** aucune nouvelle.

- [ ] **Step 1: Écrire `remote_daemon_test.dart`** (`@TestOn('linux')`, ignoré avec un message clair si `engine/target/{release,debug}/mira-daemon` est absent ; lance le **vrai binaire** avec `--home <dossier jetable> --secrets-key-file <32 octets 0600>` **sans** `--socket` ni `--token-file`, `XDG_RUNTIME_DIR` retiré de son environnement, et résout les chemins avec `resolveDaemonPaths`). Tests : `dart_and_rust_agree_on_the_default_paths` (le client trouve la socket et le jeton que le démon vient de créer) ; `the_real_daemon_answers_get_contract_info_through_RemoteEngineClient` (`version` = `{major: 1, minor: 6}`) ; `a_second_client_sees_the_same_engine` ; `events_deliver_a_local_label_change` (créer un libellé local par `call`, recevoir l'événement) ; `closing_the_client_leaves_the_daemon_running` (un second `attach` réussit après `close()`) ; `shutdown_stops_the_daemon_process_with_code_0` ; `openEngineClient_in_auto_mode_attaches_to_that_daemon` (politique réelle, `auto`, démon déjà vivant). Lancer sans binaire : tous ignorés ; avec binaire : PASS.
- [ ] **Step 2: Makefile** : cible `daemon` (`cd engine && cargo build --release -p mira-daemon`), listée dans l'aide comme les autres. Vérifier `make -n daemon`.
- [ ] **Step 3: Documentation.** `PROTOCOLE-DEMON.md` : section « Client Dart » (fichiers, fenêtres de crédits 256/128 et 64/32, reconnexion et reprise, ce que fait `close()`, politique de choix, variables `MIRA_ENGINE`, `MIRA_DAEMON_BIN`) ; `ARCHITECTURE-IMPLEMENTEE.md` : une ligne pour `app/lib/engine/remote/`. `docs/dev/DECISIONS-AGENT.md` : AG-90 (politique de choix, repli embarqué seulement si le démon est absent, `client_id`, limites : événements perdus pendant une coupure sans `lastSeq`, `retry_unlock` absent, démon lancé détaché sans supervision avant la phase 4).
- [ ] **Step 4: Vérifier** : `cd app && flutter analyze && flutter test test/remote` (avec `engine/target/debug/mira-daemon` construit : `cd engine && cargo build -p mira-daemon`), `tools/i18n-audit`, `tools/check-norms engine/crates` (le test Rust ajouté en tâche 1 respecte les normes), `cd engine && cargo test -p mira-protocol`. Expected: tout vert.
- [ ] **Step 5: Commit** — `docs(client): client Dart du démon et test contre le vrai binaire`.
