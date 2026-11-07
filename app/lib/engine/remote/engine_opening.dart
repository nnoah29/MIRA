import 'dart:async';
import 'dart:io';
import 'dart:math';

import 'package:shared_preferences/shared_preferences.dart';

import '../../platform_info.dart';
import '../engine_client.dart';
import 'connection.dart';
import 'daemon_launcher.dart';
import 'daemon_restart.dart';
import 'daemon_paths.dart';
import 'daemon_token.dart';
import 'endpoint.dart';
import 'host_errors.dart';
import 'host_state.dart';
import 'remote_engine_client.dart';

/// Quel moteur l'app ouvre : le démon s'il tourne ou peut démarrer, sinon l'embarqué.
enum EngineMode {
  /// Démon si possible, moteur embarqué en dernier recours.
  auto,

  /// Toujours le moteur embarqué (pont FFI).
  embedded,

  /// Toujours le démon ; son absence est une erreur.
  daemon,
}

/// Lit `MIRA_ENGINE` : `embedded`, `daemon`, tout autre valeur donne [EngineMode.auto].
EngineMode parseEngineMode(String? value) => switch (value) {
  'embedded' => EngineMode.embedded,
  'daemon' => EngineMode.daemon,
  _ => EngineMode.auto,
};

/// Aucun démon n'écoute (pas de socket, socket périmée ou connexion refusée).
class DaemonAbsent implements Exception {
  const DaemonAbsent();

  @override
  String toString() => 'DaemonAbsent';
}

/// Tout ce que la politique de choix touche : remplaçable dans les tests.
class EngineOpeningDeps {
  EngineOpeningDeps({
    required this.env,
    required this.isLinuxDesktop,
    required this.attachDaemon,
    required this.launchDaemon,
    required this.restartDaemon,
    required this.openEmbedded,
    Future<void> Function(Duration)? pause,
    Duration Function()? clock,
  }) : pause = pause ?? Future<void>.delayed,
       clock = clock ?? _stopwatchClock();

  /// Variables d'environnement (chemins du profil).
  final Map<String, String> env;

  /// Vrai sous Linux de bureau, la seule plateforme qui peut choisir le démon.
  final bool isLinuxDesktop;

  /// S'attache au démon de ces chemins ; lève [DaemonAbsent] ou [SocketException] s'il n'y en a pas.
  final Future<RemoteEngineClient> Function(DaemonPaths paths) attachDaemon;

  /// Démarre le démon ; `null` si aucun binaire n'est disponible.
  final Future<LaunchedDaemon?> Function(String profile, String? home)
  launchDaemon;

  /// Redémarre un démon plus ancien que l'app (unité `systemd --user`, ou arrêt puis relance).
  final Future<void> Function(RemoteEngineClient stale) restartDaemon;

  /// Ouvre le moteur embarqué.
  final Future<EngineClient> Function() openEmbedded;

  /// Attente entre deux essais.
  final Future<void> Function(Duration) pause;

  /// Temps écoulé (monotone) : les délais de démarrage se mesurent dessus, pas sur la somme des pauses.
  final Duration Function() clock;

  /// Dépendances réelles : environnement du processus, socket Unix, lanceur de `mira-daemon`.
  factory EngineOpeningDeps.real({
    required SharedPreferences prefs,
    required Future<EngineClient> Function() openEmbedded,
  }) {
    final clientId = loadClientId(prefs);
    return EngineOpeningDeps(
      env: Platform.environment,
      isLinuxDesktop: runsOnLinux,
      openEmbedded: openEmbedded,
      attachDaemon: (paths) => attachDaemonAt(paths, clientId),
      launchDaemon: (profile, home) async {
        final launcher = DaemonLauncher();
        launcher.verifyExplicitBinary();
        if (launcher.findBinary() == null) return null;
        return launcher.start(profile: profile, home: home);
      },
      restartDaemon: DaemonRestarter().restart,
    );
  }
}

/// S'attache au démon de ces chemins : [DaemonAbsent] sans socket ou si la socket est périmée ;
/// le jeton est relu à chaque (re)connexion, le démon ayant pu redémarrer avec un autre.
Future<RemoteEngineClient> attachDaemonAt(
  DaemonPaths paths,
  String clientId,
) async {
  if (!File(paths.socket).existsSync()) throw const DaemonAbsent();
  final token = await readDaemonToken(paths.token);
  try {
    return await RemoteEngineClient.attach(
      DaemonEndpoint(unixSocket: paths.socket, token: token),
      clientId: clientId,
      tokenProvider: () => readDaemonToken(paths.token),
    );
  } on SocketException {
    throw const DaemonAbsent();
  }
}

/// Horloge monotone qui démarre à l'appel.
Duration Function() _stopwatchClock() {
  final watch = Stopwatch()..start();
  return () => watch.elapsed;
}

/// Identifiant stable de cette installation, créé au premier besoin (128 bits aléatoires, hexadécimal).
String loadClientId(SharedPreferences prefs) {
  final known = prefs.getString('engineClientId');
  if (known != null && known.isNotEmpty) return known;
  final random = Random.secure();
  final id = [
    for (var i = 0; i < 16; i++)
      random.nextInt(256).toRadixString(16).padLeft(2, '0'),
  ].join();
  prefs.setString('engineClientId', id);
  return id;
}

const Duration _poll = Duration(milliseconds: 250);
const Duration _launchWait = Duration(seconds: 20);
const Duration _startingWait = Duration(seconds: 30);
const Duration _keyringWait = Duration(seconds: 20);
const Duration _restartWait = Duration(seconds: 10);

/// Une tentative d'attache ; `null` si aucun démon ne répond.
Future<RemoteEngineClient?> _attachOnce(
  EngineOpeningDeps deps,
  DaemonPaths paths,
) async {
  try {
    return await deps.attachDaemon(paths);
  } on DaemonAbsent {
    return null;
  } on SocketException {
    return null;
  } on ConnectionLost {
    return null;
  } on HandshakeRefused catch (refusal) {
    throw refusalError(refusal);
  }
}

/// S'attache et attend que l'hôte soit prêt ; `null` si aucun démon ne répond.
/// Un démon périmé est redémarré une fois ; un trousseau verrouillé reçoit `retry_unlock` à chaque essai.
Future<RemoteEngineClient?> _attachReady(
  EngineOpeningDeps deps,
  DaemonPaths paths,
) async {
  final started = deps.clock();
  Duration? restartedAt;
  while (true) {
    final client = await _attachOnce(deps, paths);
    if (client == null) return null;
    switch (client.hostState) {
      case HostState.ready || HostState.unknown:
        return client;
      case HostState.keyringLocked:
        await client.retryUnlock();
        await client.close();
        if (deps.clock() - started >= _keyringWait) throw keyringLockedError();
        await deps.pause(_poll);
      case HostState.updateRequired:
        final since = restartedAt;
        if (since == null) {
          restartedAt = deps.clock();
          await deps.restartDaemon(client);
        } else {
          await client.close();
          if (deps.clock() - since >= _restartWait) throw updateRequiredError();
        }
        await deps.pause(_poll);
      case HostState.starting:
        await client.close();
        if (deps.clock() - started >= _startingWait) throw unavailableError();
        await deps.pause(_poll);
    }
  }
}

/// Démarre le démon puis l'attend jusqu'à 20 s ; `null` sans binaire ou sans réponse.
Future<RemoteEngineClient?> _launchAndAttach(
  EngineOpeningDeps deps,
  DaemonPaths paths,
  String profile,
  String? home,
) async {
  final launched = await deps.launchDaemon(profile, home);
  if (launched == null) return null;
  int? exitCode;
  unawaited(launched.exitCode.then((code) => exitCode = code));
  final started = deps.clock();
  while (deps.clock() - started < _launchWait) {
    final client = await _attachReady(deps, paths);
    if (client != null) return client;
    final exited = exitCode;
    if (exited != null) throw daemonExitedError(exited);
    await deps.pause(_poll);
  }
  return null;
}

/// Vrai si l'app affiche elle-même les notifications : seulement avec le moteur embarqué, le démon décide et affiche les siennes.
bool appShowsNotifications(EngineMode used) => used == EngineMode.embedded;

/// Ouvre le moteur selon la politique : embarqué hors Linux de bureau, sinon démon vivant, démon lancé, puis embarqué.
/// Les états `keyring_locked` et `update_required` et un jeton refusé ne déclenchent jamais le repli.
Future<({EngineClient client, EngineMode used})> openEngineClient({
  required EngineOpeningDeps deps,
  EngineMode mode = EngineMode.auto,
  String profile = 'default',
  String? home,
}) async {
  Future<({EngineClient client, EngineMode used})> embedded() async =>
      (client: await deps.openEmbedded(), used: EngineMode.embedded);
  if (mode == EngineMode.embedded || !deps.isLinuxDesktop) return embedded();
  RemoteEngineClient? client;
  try {
    final paths = resolveDaemonPaths(
      profile: profile,
      env: deps.env,
      home: home,
    );
    client =
        await _attachReady(deps, paths) ??
        await _launchAndAttach(deps, paths, profile, home);
  } on ArgumentError {
    client = null;
  } on EngineException catch (error) {
    final launchFailure =
        error.code == 'transport.daemon_start_failed' ||
        error.code == 'transport.daemon_missing';
    if (mode == EngineMode.daemon || !launchFailure) rethrow;
  }
  if (client != null) return (client: client, used: EngineMode.daemon);
  if (mode == EngineMode.daemon) throw unavailableError();
  return embedded();
}
