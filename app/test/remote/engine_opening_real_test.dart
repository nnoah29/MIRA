import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/daemon_paths.dart';
import 'package:mira/engine/remote/engine_opening.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  late Directory dir;
  setUp(() => dir = Directory.systemTemp.createTempSync('mira-real'));
  tearDown(() => dir.deleteSync(recursive: true));

  DaemonPaths pathsIn(Directory d) => DaemonPaths(
    socket: '${d.path}/daemon.sock',
    token: '${d.path}/daemon.token',
  );

  test('without_a_socket_file_the_daemon_is_absent', () async {
    await expectLater(
      attachDaemonAt(pathsIn(dir), 'client'),
      throwsA(isA<DaemonAbsent>()),
    );
  });

  test('a_stale_socket_file_counts_as_absent', () async {
    final paths = pathsIn(dir);
    // Un processus lie la socket puis meurt : le fichier reste, personne n'écoute.
    final made = Process.runSync('python3', [
      '-c',
      'import socket,sys; socket.socket(socket.AF_UNIX).bind(sys.argv[1])',
      paths.socket,
    ]);
    expect(made.exitCode, 0, reason: '${made.stderr}');
    expect(
      File(paths.socket).existsSync(),
      isTrue,
      reason: 'la socket reste sur le disque',
    );
    File(paths.token).writeAsStringSync('ab12' * 16);
    Process.runSync('chmod', ['600', paths.token]);
    await expectLater(
      attachDaemonAt(paths, 'client'),
      throwsA(isA<DaemonAbsent>()),
    );
  }, testOn: 'linux');

  test(
    'real_deps_use_the_process_environment_and_keep_a_stable_client_id',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final deps = EngineOpeningDeps.real(
        prefs: prefs,
        openEmbedded: () async => throw StateError('non utilisé'),
      );
      expect(deps.env, Platform.environment);
      expect(deps.isLinuxDesktop, Platform.isLinux);
      expect(
        prefs.getString('engineClientId'),
        matches(RegExp(r'^[0-9a-f]{32}$')),
      );
      await expectLater(
        deps.attachDaemon(pathsIn(dir)),
        throwsA(isA<DaemonAbsent>()),
      );
    },
    testOn: 'linux',
  );
}
