import 'dart:io';

import '../../i18n/i18n.dart';
import '../engine_client.dart';

/// Lance un processus ; remplaçable dans les tests.
typedef ProcessStarter = Future<Process> Function(
  String executable,
  List<String> arguments, {
  ProcessStartMode mode,
  Map<String, String>? environment,
});

/// Emplacements où chercher `mira-daemon`, du plus explicite au plus courant.
List<String> defaultBinaryCandidates({
  required Map<String, String> env,
  required String executableDir,
}) => [
  if ((env['MIRA_DAEMON_BIN'] ?? '').isNotEmpty) env['MIRA_DAEMON_BIN']!,
  '$executableDir/mira-daemon',
  '$executableDir/lib/mira-daemon',
  '../engine/target/release/mira-daemon',
  '../engine/target/debug/mira-daemon',
];

/// Démarre le démon quand aucun ne tourne : processus détaché, jamais de jeton en argument.
class DaemonLauncher {
  DaemonLauncher({
    ProcessStarter? start,
    List<String>? binaryCandidates,
    Map<String, String>? env,
  }) : _start = start ?? Process.start,
       _candidates =
           binaryCandidates ??
           defaultBinaryCandidates(
             env: env ?? Platform.environment,
             executableDir: File(Platform.resolvedExecutable).parent.path,
           );

  final ProcessStarter _start;
  final List<String> _candidates;

  /// Premier exécutable existant parmi les candidats, ou `null`.
  String? findBinary() {
    for (final candidate in _candidates) {
      if (File(candidate).existsSync()) return candidate;
    }
    return null;
  }

  /// Lance `mira-daemon --profile <profil> [--home <dossier>]` en mode détaché.
  Future<void> start({required String profile, String? home}) async {
    final binary = findBinary();
    if (binary == null) {
      throw EngineException.local(
        'transport.daemon_missing',
        t('Moteur en arrière-plan introuvable'),
      );
    }
    try {
      await _start(binary, [
        '--profile',
        profile,
        if (home != null) ...['--home', home],
      ], mode: ProcessStartMode.detached);
    } on ProcessException {
      throw EngineException.local(
        'transport.daemon_start_failed',
        t('Impossible de démarrer le moteur en arrière-plan'),
      );
    }
  }
}
