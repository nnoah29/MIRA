import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/resident.dart';
import 'package:mira/engine/remote/resident_windows.dart';

/// Registre factice : garde les commandes `reg` et répond avec ce code.
class Registry {
  final List<List<String>> ran = [];
  final List<String> started = [];
  int exitCode = 0;

  Future<ProcessResult> run(String executable, List<String> arguments) async {
    ran.add([executable, ...arguments]);
    return ProcessResult(1, exitCode, '', '');
  }

  Future<void> start(String executable) async => started.add(executable);
}

void main() {
  const key = r'HKCU\Software\Microsoft\Windows\CurrentVersion\Run';

  test('availability_means_the_daemon_exe_sits_beside_the_app', () async {
    final dir = Directory.systemTemp.createTempSync('mira-res');
    addTearDown(() => dir.deleteSync(recursive: true));
    final exe = File('${dir.path}/mira-daemon.exe');
    final controller = WindowsResidentController(
      daemonExe: exe.path,
      run: Registry().run,
    );
    expect(await controller.isAvailable(), isFalse);
    exe.writeAsStringSync('x');
    expect(await controller.isAvailable(), isTrue);
  });

  test('enabled_means_the_run_value_exists', () async {
    final registry = Registry();
    final controller = WindowsResidentController(
      daemonExe: r'C:\M\mira-daemon.exe',
      run: registry.run,
    );
    expect(await controller.isEnabled(), isTrue);
    expect(registry.ran.single, ['reg', 'query', key, '/v', 'MIRA']);
    registry.exitCode = 1;
    expect(await controller.isEnabled(), isFalse);
  });

  test(
    'enable_writes_the_run_value_with_the_quoted_path_and_starts_the_daemon',
    () async {
      final registry = Registry();
      final controller = WindowsResidentController(
        daemonExe: r'C:\Program Files\M\mira-daemon.exe',
        run: registry.run,
        start: registry.start,
      );
      await controller.enable();
      expect(registry.ran.single, [
        'reg',
        'add',
        key,
        '/v',
        'MIRA',
        '/t',
        'REG_SZ',
        '/d',
        r'"C:\Program Files\M\mira-daemon.exe"',
        '/f',
      ]);
      expect(registry.started, [r'C:\Program Files\M\mira-daemon.exe']);
    },
  );

  test('disable_deletes_the_run_value_and_a_failure_is_reported', () async {
    final registry = Registry();
    final controller = WindowsResidentController(
      daemonExe: r'C:\M\mira-daemon.exe',
      run: registry.run,
    );
    await controller.disable();
    expect(registry.ran.single, ['reg', 'delete', key, '/v', 'MIRA', '/f']);
    registry.exitCode = 1;
    await expectLater(controller.disable(), throwsA(isA<ProcessException>()));
  });

  test('the_platform_factory_gives_the_systemd_controller_off_windows', () {
    expect(
      ResidentController.forPlatform(windows: false),
      isNot(isA<WindowsResidentController>()),
    );
    expect(
      ResidentController.forPlatform(windows: true),
      isA<WindowsResidentController>(),
    );
  });
}
