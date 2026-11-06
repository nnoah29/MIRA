import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/remote/daemon_launcher.dart';

/// Processus factice : seul `pid` est lu.
class FakeProcess implements Process {
  @override
  int get pid => 4242;

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
    );
    expect(candidates, [
      '/custom/mira-daemon',
      '/opt/mira/mira-daemon',
      '/opt/mira/lib/mira-daemon',
      '../engine/target/release/mira-daemon',
      '../engine/target/debug/mira-daemon',
    ]);
    expect(
      defaultBinaryCandidates(env: {}, executableDir: '/o').first,
      '/o/mira-daemon',
    );
  });

  test('no_binary_found_yields_null', () {
    final launcher = DaemonLauncher(binaryCandidates: ['${dir.path}/nope']);
    expect(launcher.findBinary(), isNull);
  });

  test(
    'start_runs_the_binary_detached_with_profile_and_home_and_never_the_token',
    () async {
      final binary = fakeBinary(dir, 'mira-daemon');
      String? exe;
      List<String>? args;
      ProcessStartMode? started;
      final launcher = DaemonLauncher(
        binaryCandidates: [binary],
        start: (e, a, {mode = ProcessStartMode.normal, environment}) async {
          exe = e;
          args = a;
          started = mode;
          return FakeProcess();
        },
      );
      await launcher.start(profile: 'default', home: '/data/mira');
      expect(exe, binary);
      expect(started, ProcessStartMode.detached);
      expect(args, ['--profile', 'default', '--home', '/data/mira']);
      expect(args!.join(' '), isNot(contains('token')));
    },
  );

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
}
