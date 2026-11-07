import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/resident.dart';

/// Lanceur de commandes factice : garde les commandes et répond avec ce code (ou échoue).
class Commands {
  final List<List<String>> ran = [];
  int exitCode = 0;
  bool missing = false;

  Future<ProcessResult> run(String executable, List<String> arguments) async {
    if (missing) throw const ProcessException('systemctl', []);
    ran.add([executable, ...arguments]);
    return ProcessResult(1, exitCode, '', '');
  }
}

void main() {
  test('availability_means_the_user_unit_exists', () async {
    final commands = Commands();
    final controller = ResidentController(run: commands.run);
    expect(await controller.isAvailable(), isTrue);
    expect(commands.ran.single, [
      'systemctl',
      '--user',
      'cat',
      'mira-daemon.service',
    ]);
    commands.exitCode = 1;
    expect(await controller.isAvailable(), isFalse);
  });

  test('without_systemctl_nothing_is_available_or_enabled', () async {
    final commands = Commands()..missing = true;
    final controller = ResidentController(run: commands.run);
    expect(await controller.isAvailable(), isFalse);
    expect(await controller.isEnabled(), isFalse);
  });

  test('enable_and_disable_use_the_user_unit_now', () async {
    final commands = Commands();
    final controller = ResidentController(run: commands.run);
    await controller.enable();
    await controller.disable();
    expect(commands.ran, [
      ['systemctl', '--user', 'enable', '--now', 'mira-daemon.service'],
      ['systemctl', '--user', 'disable', '--now', 'mira-daemon.service'],
    ]);
  });

  test('is_enabled_reads_the_unit_state', () async {
    final commands = Commands();
    final controller = ResidentController(run: commands.run);
    expect(await controller.isEnabled(), isTrue);
    expect(commands.ran.single, [
      'systemctl',
      '--user',
      'is-enabled',
      '--quiet',
      'mira-daemon.service',
    ]);
    commands.exitCode = 1;
    expect(await controller.isEnabled(), isFalse);
  });

  test('a_failing_enable_is_reported_to_the_caller', () async {
    final commands = Commands()..exitCode = 1;
    final controller = ResidentController(run: commands.run);
    await expectLater(controller.enable(), throwsA(isA<ProcessException>()));
  });
}
