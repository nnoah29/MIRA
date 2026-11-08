import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/daemon_launcher.dart';
import 'package:mira/engine/remote/daemon_paths.dart';
import 'package:mira/engine/remote/daemon_port.dart';
import 'package:mira/engine/remote/daemon_token.dart';
import 'package:mira/engine/remote/engine_opening.dart';

void main() {
  late Directory dir;
  setUp(() => dir = Directory.systemTemp.createTempSync('mira-win'));
  tearDown(() => dir.deleteSync(recursive: true));

  test(
    'on_windows_the_profile_root_comes_from_LOCALAPPDATA_and_has_a_port_file',
    () {
      final paths = resolveDaemonPaths(
        profile: 'p',
        env: {'LOCALAPPDATA': r'C:\Users\u\AppData\Local'},
        windows: true,
      );
      expect(
        paths.port,
        r'C:\Users\u\AppData\Local/mira/profiles/p/daemon.port',
      );
      expect(
        paths.token,
        r'C:\Users\u\AppData\Local/mira/profiles/p/daemon.token',
      );
    },
  );

  test('the_port_file_sits_beside_the_token_everywhere', () {
    final paths = resolveDaemonPaths(profile: 'p', env: {'MIRA_HOME': '/m'});
    expect(paths.port, '/m/profiles/p/daemon.port');
  });

  test(
    'a_published_port_is_read_back_and_anything_else_means_absent',
    () async {
      final file = File('${dir.path}/daemon.port');
      file.writeAsStringSync('43210\n');
      expect(await readDaemonPort(file.path), 43210);
      for (final bad in ['', 'abc', '0', '70000', '-5']) {
        file.writeAsStringSync(bad);
        await expectLater(
          readDaemonPort(file.path),
          throwsA(isA<DaemonAbsent>()),
          reason: bad,
        );
      }
      await expectLater(
        readDaemonPort('${dir.path}/none'),
        throwsA(isA<DaemonAbsent>()),
      );
    },
  );

  test('without_a_port_file_tcp_attach_finds_no_daemon', () async {
    final paths = DaemonPaths(
      socket: '${dir.path}/daemon.sock',
      token: '${dir.path}/daemon.token',
      port: '${dir.path}/daemon.port',
    );
    await expectLater(
      attachDaemonAt(paths, 'client', tcp: true),
      throwsA(isA<DaemonAbsent>()),
    );
  });

  test(
    'a_token_file_is_accepted_without_unix_modes_when_they_do_not_apply',
    () async {
      final file = File('${dir.path}/daemon.token')
        ..writeAsStringSync('ab12' * 16);
      if (!Platform.isWindows) Process.runSync('chmod', ['644', file.path]);
      expect(
        await readDaemonToken(file.path, enforcePrivateMode: false),
        'ab12' * 16,
      );
    },
  );

  test('windows_looks_for_mira_daemon_exe_beside_the_app', () {
    final candidates = defaultBinaryCandidates(
      env: const {},
      executableDir: r'C:\Mira',
      releaseMode: true,
      windows: true,
    );
    expect(candidates, [
      r'C:\Mira/mira-daemon.exe',
      r'C:\Mira/lib/mira-daemon.exe',
    ]);
  });
}
