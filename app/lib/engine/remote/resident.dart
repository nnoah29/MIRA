import 'dart:io';

import 'daemon_restart.dart';

/// Mode résident : le démon tourne sous l'unité `systemd --user`, fenêtre fermée comprise.
class ResidentController {
  ResidentController({CommandRunner? run}) : _run = run ?? Process.run;

  final CommandRunner _run;

  static const String unit = 'mira-daemon.service';

  /// Exécute `systemctl --user <arguments>` ; `null` si `systemctl` est absent.
  Future<ProcessResult?> _systemctl(List<String> arguments) async {
    try {
      return await _run('systemctl', ['--user', ...arguments]);
    } on ProcessException {
      return null;
    }
  }

  /// Vrai si l'unité du démon est installée (paquet deb, rpm, arch ou tarball) et `systemd --user` répond.
  Future<bool> isAvailable() async =>
      (await _systemctl(['cat', unit]))?.exitCode == 0;

  /// Vrai si l'unité est activée (le démon démarre à chaque connexion).
  Future<bool> isEnabled() async =>
      (await _systemctl(['is-enabled', '--quiet', unit]))?.exitCode == 0;

  /// Active l'unité et la démarre ; lève [ProcessException] si `systemctl` refuse.
  Future<void> enable() => _change('enable');

  /// Désactive l'unité et l'arrête ; lève [ProcessException] si `systemctl` refuse.
  Future<void> disable() => _change('disable');

  Future<void> _change(String verb) async {
    final result = await _systemctl([verb, '--now', unit]);
    if (result == null || result.exitCode != 0) {
      throw ProcessException('systemctl', ['--user', verb, '--now', unit]);
    }
  }
}
