import 'dart:io';

import 'package:flutter/foundation.dart' show kReleaseMode;

import '../../i18n/i18n.dart';
import '../engine_client.dart';

/// Lance un processus ; remplaçable dans les tests.
typedef ProcessStarter = Future<Process> Function(
  String executable,
  List<String> arguments, {
  ProcessStartMode mode,
  Map<String, String>? environment,
});

/// Racine du dépôt (dossier qui contient `engine/Cargo.toml`) au-dessus du dossier de l'exécutable ; `null` si aucune.
String? _repositoryRoot(String executableDir) {
  var dir = Directory(executableDir);
  for (var i = 0; i < 8; i++) {
    if (File('${dir.path}/engine/Cargo.toml').existsSync()) return dir.path;
    final parent = dir.parent;
    if (parent.path == dir.path) return null;
    dir = parent;
  }
  return null;
}

/// Emplacements où chercher `mira-daemon`, du plus explicite au plus courant. Tous sont absolus :
/// jamais relatifs au répertoire courant, où n'importe qui peut avoir déposé un exécutable.
/// Les binaires de compilation du dépôt ne sont cherchés que hors version de production.
List<String> defaultBinaryCandidates({
  required Map<String, String> env,
  required String executableDir,
  bool releaseMode = kReleaseMode,
}) {
  final root = releaseMode ? null : _repositoryRoot(executableDir);
  return [
    if ((env['MIRA_DAEMON_BIN'] ?? '').isNotEmpty) env['MIRA_DAEMON_BIN']!,
    '$executableDir/mira-daemon',
    '$executableDir/lib/mira-daemon',
    if (root != null) ...[
      '$root/engine/target/release/mira-daemon',
      '$root/engine/target/debug/mira-daemon',
    ],
  ];
}

/// Démon lancé par l'app : permet de savoir s'il s'arrête avant d'être prêt.
class LaunchedDaemon {
  const LaunchedDaemon({required this.exitCode});

  /// Aboutit avec le code de sortie du processus.
  final Future<int> exitCode;
}

/// Démarre le démon quand aucun ne tourne : à la demande (il s'arrête quand l'app est partie), jamais de jeton en argument.
class DaemonLauncher {
  DaemonLauncher({
    ProcessStarter? start,
    List<String>? binaryCandidates,
    Map<String, String>? env,
  }) : _start = start ?? Process.start,
       _env = env ?? Platform.environment,
       _candidates =
           binaryCandidates ??
           defaultBinaryCandidates(
             env: env ?? Platform.environment,
             executableDir: File(Platform.resolvedExecutable).parent.path,
           );

  final ProcessStarter _start;
  final Map<String, String> _env;
  final List<String> _candidates;

  /// Premier exécutable existant parmi les candidats, ou `null`.
  String? findBinary() {
    for (final candidate in _candidates) {
      if (File(candidate).existsSync()) return candidate;
    }
    return null;
  }

  /// Refuse une variable `MIRA_DAEMON_BIN` qui désigne un fichier absent : un choix explicite ne s'ignore pas en silence.
  void verifyExplicitBinary() {
    final explicit = _env['MIRA_DAEMON_BIN'];
    if (explicit == null || explicit.isEmpty) return;
    if (File(explicit).existsSync()) return;
    throw EngineException.local(
      'transport.daemon_bin_invalid',
      t('La variable MIRA_DAEMON_BIN désigne un fichier introuvable'),
    );
  }

  /// Lance `mira-daemon --profile <profil> [--home <dossier>] --exit-when-idle` et rend de quoi surveiller sa sortie.
  Future<LaunchedDaemon> start({required String profile, String? home}) async {
    verifyExplicitBinary();
    final binary = findBinary();
    if (binary == null) {
      throw EngineException.local(
        'transport.daemon_missing',
        t('Moteur en arrière-plan introuvable'),
      );
    }
    try {
      final process = await _start(binary, [
        '--profile',
        profile,
        if (home != null) ...['--home', home],
        '--exit-when-idle',
      ], mode: ProcessStartMode.inheritStdio);
      return LaunchedDaemon(exitCode: process.exitCode);
    } on ProcessException {
      throw EngineException.local(
        'transport.daemon_start_failed',
        t('Impossible de démarrer le moteur en arrière-plan'),
      );
    }
  }
}
