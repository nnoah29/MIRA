import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/remote/daemon_launcher.dart';

/// Processus factice : `pid` et `exitCode` (jamais terminé sauf appel de [exit]) sont lus.
class FakeProcess implements Process {
  final Completer<int> _exit = Completer<int>();

  /// Termine le processus avec ce code.
  void exit(int code) => _exit.complete(code);

  @override
  int get pid => 4242;

  @override
  Future<int> get exitCode => _exit.future;

  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}

/// Fichier exécutable factice dans `dir`.
String fakeBinary(Directory dir, String name) {
  final file = File('${dir.path}/$name')..writeAsStringSync('#!/bin/sh\n');
  return file.path;
}

void main() {
  late Directory dir;
  setUp(() => dir = Directory.systemTemp.createTempSync('mira-launch'));
  tearDown(() => dir.deleteSync(recursive: true));

  test('binary_candidates_are_tried_in_order', () {
    final first = '${dir.path}/absent';
    final second = fakeBinary(dir, 'b');
    final third = fakeBinary(dir, 'c');
    final launcher = DaemonLauncher(binaryCandidates: [first, second, third]);
    expect(launcher.findBinary(), second);
  });

  test('the_default_candidates_follow_the_documented_order', () {
    final candidates = defaultBinaryCandidates(
      env: {'MIRA_DAEMON_BIN': '/custom/mira-daemon'},
      executableDir: '/opt/mira',
      releaseMode: true,
    );
    expect(candidates, [
      '/custom/mira-daemon',
      '/opt/mira/mira-daemon',
      '/opt/mira/lib/mira-daemon',
    ]);
  });

  test('release_builds_never_search_cwd_relative_paths', () {
    final candidates = defaultBinaryCandidates(
      env: {},
      executableDir: '/opt/mira',
      releaseMode: true,
    );
    expect(candidates.where((c) => !c.startsWith('/')), isEmpty);
    expect(candidates.any((c) => c.contains('engine/target')), isFalse);
  });

  test(
    'dev_builds_locate_the_repository_from_the_executable_not_from_the_cwd',
    () {
      final repo = Directory('${dir.path}/repo')..createSync();
      Directory('${repo.path}/engine').createSync();
      File('${repo.path}/engine/Cargo.toml').writeAsStringSync('[workspace]');
      final bundle = Directory('${repo.path}/app/build/linux/x64/debug/bundle')
        ..createSync(recursive: true);
      final candidates = defaultBinaryCandidates(
        env: {},
        executableDir: bundle.path,
        releaseMode: false,
      );
      expect(
        candidates,
        contains('${repo.path}/engine/target/release/mira-daemon'),
      );
      expect(
        candidates,
        contains('${repo.path}/engine/target/debug/mira-daemon'),
      );
      expect(candidates.where((c) => !c.startsWith('/')), isEmpty);
    },
  );

  test('no_repository_above_the_executable_yields_no_dev_candidates', () {
    final bundle = Directory('${dir.path}/nowhere/bundle')
      ..createSync(recursive: true);
    final candidates = defaultBinaryCandidates(
      env: {},
      executableDir: bundle.path,
      releaseMode: false,
    );
    expect(candidates.any((c) => c.contains('engine/target')), isFalse);
  });

  test('no_binary_found_yields_null', () {
    final launcher = DaemonLauncher(binaryCandidates: ['${dir.path}/nope']);
    expect(launcher.findBinary(), isNull);
  });

  test('start_runs_the_binary_on_demand_with_profile_home_and_exit_when_idle_and_never_the_token', () async {
    final binary = fakeBinary(dir, 'mira-daemon');
    String? exe;
    List<String>? args;
    ProcessStartMode? started;
    final process = FakeProcess();
    final launcher = DaemonLauncher(
      binaryCandidates: [binary],
      start: (e, a, {mode = ProcessStartMode.normal, environment}) async {
        exe = e;
        args = a;
        started = mode;
        return process;
      },
    );
    final launched = await launcher.start(
      profile: 'default',
      home: '/data/mira',
    );
    expect(exe, binary);
    expect(started, ProcessStartMode.inheritStdio);
    expect(args, [
      '--profile',
      'default',
      '--home',
      '/data/mira',
      '--exit-when-idle',
    ]);
    expect(args!.join(' '), isNot(contains('token')));
    process.exit(3);
    expect(await launched.exitCode, 3);
  });

  test('start_fails_with_a_clear_error_when_no_binary_exists', () async {
    final launcher = DaemonLauncher(binaryCandidates: ['${dir.path}/nope']);
    await expectLater(
      launcher.start(profile: 'default'),
      throwsA(
        isA<EngineException>().having(
          (e) => e.code,
          'code',
          'transport.daemon_missing',
        ),
      ),
    );
  });

  test(
    'start_fails_with_a_clear_error_when_the_process_cannot_be_started',
    () async {
      final binary = fakeBinary(dir, 'mira-daemon');
      final launcher = DaemonLauncher(
        binaryCandidates: [binary],
        start: (e, a, {mode = ProcessStartMode.normal, environment}) async =>
            throw const ProcessException('x', []),
      );
      await expectLater(
        launcher.start(profile: 'default'),
        throwsA(
          isA<EngineException>().having(
            (e) => e.code,
            'code',
            'transport.daemon_start_failed',
          ),
        ),
      );
    },
  );

  test(
    'an_explicit_binary_that_does_not_exist_is_an_error_not_silently_ignored',
    () async {
      final good = fakeBinary(dir, 'mira-daemon');
      final launcher = DaemonLauncher(
        env: {'MIRA_DAEMON_BIN': '${dir.path}/nope'},
        binaryCandidates: [good],
      );
      expect(
        launcher.verifyExplicitBinary,
        throwsA(
          isA<EngineException>().having(
            (e) => e.code,
            'code',
            'transport.daemon_bin_invalid',
          ),
        ),
      );
      await expectLater(
        launcher.start(profile: 'default'),
        throwsA(
          isA<EngineException>().having(
            (e) => e.code,
            'code',
            'transport.daemon_bin_invalid',
          ),
        ),
      );
      final fine = DaemonLauncher(
        env: {'MIRA_DAEMON_BIN': good},
        binaryCandidates: [good],
      );
      fine.verifyExplicitBinary();
      DaemonLauncher(
        env: const {},
        binaryCandidates: [good],
      ).verifyExplicitBinary();
    },
  );
}
