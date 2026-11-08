import 'dart:io';

import 'daemon_restart.dart';
import 'resident.dart';

/// Lance le démon résident (sans arrêt à l'inactivité) ; remplaçable dans les tests.
typedef DaemonStarter = Future<void> Function(String executable);

/// Démarre le programme à part, sans attendre sa fin.
Future<void> _startDetached(String executable) async {
  await Process.start(executable, const [], mode: ProcessStartMode.detached);
}

/// Mode résident sous Windows : le démon est lancé à l'ouverture de session par la clé `Run` de l'utilisateur.
class WindowsResidentController extends ResidentController {
  // ignore: use_super_parameters
  WindowsResidentController({
    String? daemonExe,
    CommandRunner? run,
    DaemonStarter? start,
  }) : _exe =
           daemonExe ??
           '${File(Platform.resolvedExecutable).parent.path}\\mira-daemon.exe',
       _reg = run ?? Process.run,
       _start = start ?? _startDetached,
       super(run: run);

  final String _exe;
  final CommandRunner _reg;
  final DaemonStarter _start;

  static const String _key =
      r'HKCU\Software\Microsoft\Windows\CurrentVersion\Run';
  static const String _name = 'MIRA';

  /// Exécute `reg` ; `null` si la commande est introuvable.
  Future<ProcessResult?> _registry(List<String> arguments) async {
    try {
      return await _reg('reg', arguments);
    } on ProcessException {
      return null;
    }
  }

  @override
  Future<bool> isAvailable() async => File(_exe).existsSync();

  @override
  Future<bool> isEnabled() async =>
      (await _registry(['query', _key, '/v', _name]))?.exitCode == 0;

  @override
  Future<void> enable() async {
    final result = await _registry([
      'add',
      _key,
      '/v',
      _name,
      '/t',
      'REG_SZ',
      '/d',
      '"$_exe"',
      '/f',
    ]);
    if (result == null || result.exitCode != 0) {
      throw ProcessException('reg', ['add', _key]);
    }
    try {
      await _start(_exe);
    } on ProcessException {
      return;
    }
  }

  @override
  Future<void> disable() async {
    final result = await _registry(['delete', _key, '/v', _name, '/f']);
    if (result == null || result.exitCode != 0) {
      throw ProcessException('reg', ['delete', _key]);
    }
  }
}
