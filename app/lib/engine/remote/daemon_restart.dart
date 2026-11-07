import 'dart:io';

import 'remote_engine_client.dart';

/// Lance une commande ; remplaçable dans les tests.
typedef CommandRunner =
    Future<ProcessResult> Function(String executable, List<String> arguments);

/// Redémarre un démon périmé : par `systemd --user` s'il est géré par l'unité, sinon en lui demandant de s'arrêter.
class DaemonRestarter {
  DaemonRestarter({CommandRunner? run}) : _run = run ?? Process.run;

  final CommandRunner _run;

  static const String unit = 'mira-daemon.service';

  /// Vrai si l'unité `systemd --user` du démon est active.
  Future<bool> unitActive() async {
    try {
      final result = await _run('systemctl', [
        '--user',
        'is-active',
        '--quiet',
        unit,
      ]);
      return result.exitCode == 0;
    } on ProcessException {
      return false;
    }
  }

  /// Arrête le démon périmé ; l'unité le relance, sinon l'app en lance un neuf.
  Future<void> restart(RemoteEngineClient stale) async {
    if (!await unitActive()) return stale.shutdown();
    try {
      await _run('systemctl', ['--user', 'restart', unit]);
    } on ProcessException {
      await stale.shutdown();
      return;
    }
    await stale.close();
  }
}
