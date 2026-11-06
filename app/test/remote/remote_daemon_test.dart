@TestOn('linux')
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/remote/daemon_paths.dart';
import 'package:mira/engine/remote/daemon_token.dart';
import 'package:mira/engine/remote/endpoint.dart';
import 'package:mira/engine/remote/engine_opening.dart';
import 'package:mira/engine/remote/remote_engine_client.dart';

/// Binaire `mira-daemon` construit, le plus récent des deux profils de compilation.
String? daemonBinary() {
  final found = [
    '../engine/target/release/mira-daemon',
    '../engine/target/debug/mira-daemon',
  ].where((p) => File(p).existsSync()).toList();
  if (found.isEmpty) return null;
  found.sort(
    (a, b) => File(b).lastModifiedSync().compareTo(File(a).lastModifiedSync()),
  );
  return found.first;
}

/// Un vrai démon sur un profil jetable, lancé sans `--socket` ni `--token-file` et sans `XDG_RUNTIME_DIR`.
class RunningDaemon {
  RunningDaemon(this.dir, this.process);

  final Directory dir;
  final Process process;

  String get home => '${dir.path}/home';

  /// Chemins que le client calcule pour ce démon.
  DaemonPaths get paths =>
      resolveDaemonPaths(profile: 'default', env: const {}, home: home);

  /// Démarre le binaire et attend sa ligne « prêt ».
  static Future<RunningDaemon> start(String binary) async {
    final dir = Directory.systemTemp.createTempSync('mira-dart-daemon');
    final key = File('${dir.path}/secrets.key')
      ..writeAsBytesSync(List.filled(32, 7));
    Process.runSync('chmod', ['600', key.path]);
    final environment = Map<String, String>.of(Platform.environment)
      ..remove('XDG_RUNTIME_DIR')
      ..remove('MIRA_HOME');
    final process = await Process.start(
      binary,
      ['--home', '${dir.path}/home', '--secrets-key-file', key.path],
      environment: environment,
      includeParentEnvironment: false,
    );
    final ready = Completer<void>();
    process.stdout
        .transform(utf8.decoder)
        .transform(const LineSplitter())
        .listen((line) {
          if (line == 'mira-daemon prêt' && !ready.isCompleted) {
            ready.complete();
          }
        });
    process.stderr.drain<void>();
    await ready.future.timeout(const Duration(seconds: 60));
    return RunningDaemon(dir, process);
  }

  /// S'attache comme le fera l'app.
  Future<RemoteEngineClient> attach() async => RemoteEngineClient.attach(
    DaemonEndpoint(
      unixSocket: paths.socket,
      token: await readDaemonToken(paths.token),
    ),
    clientId: 'dart-test',
  );

  /// Arrête le processus et supprime le profil.
  Future<void> dispose() async {
    process.kill();
    await process.exitCode;
    dir.deleteSync(recursive: true);
  }
}

void main() {
  final binary = daemonBinary();
  final skip = binary == null
      ? 'binaire absent (cd engine && cargo build -p mira-daemon)'
      : null;
  late RunningDaemon daemon;

  setUpAll(() async {
    if (binary != null) daemon = await RunningDaemon.start(binary);
  });
  tearDownAll(() async {
    if (binary != null) await daemon.dispose();
  });

  test('dart_and_rust_agree_on_the_default_paths', () {
    expect(File(daemon.paths.socket).existsSync(), isTrue);
    expect(File(daemon.paths.token).existsSync(), isTrue);
  }, skip: skip);

  test(
    'the_real_daemon_answers_get_contract_info_through_RemoteEngineClient',
    () async {
      final client = await daemon.attach();
      final info = await client.call(req('engine', 'get_contract_info'));
      expect(info['type'], 'contract_info');
      expect((info['data'] as Map)['version'], {'major': 1, 'minor': 6});
      await client.close();
    },
    skip: skip,
  );

  test('a_second_client_sees_the_same_engine', () async {
    final first = await daemon.attach();
    final second = await daemon.attach();
    final created = await first.call(
      req('local_label', 'create_local_label', {
        'name': 'Partagé',
        'idempotency_key': 'dart-shared',
      }),
    );
    expect(created['type'], isNotNull);
    final listed = await second.call(req('local_label', 'list_local_labels'));
    expect(jsonEncode(listed), contains('Partagé'));
    await first.close();
    await second.close();
  }, skip: skip);

  test('events_deliver_a_local_label_change', () async {
    final watcher = await daemon.attach();
    final actor = await daemon.attach();
    final seen = Completer<Map<String, dynamic>>();
    final sub = watcher.events().listen((event) {
      if (jsonEncode(event).contains('local_labels_changed') &&
          !seen.isCompleted) {
        seen.complete(event);
      }
    });
    await Future<void>.delayed(const Duration(milliseconds: 200));
    await actor.call(
      req('local_label', 'create_local_label', {
        'name': 'Événement',
        'idempotency_key': 'dart-event',
      }),
    );
    final event = await seen.future.timeout(const Duration(seconds: 10));
    expect(event['seq'], isA<int>());
    await sub.cancel();
    await watcher.close();
    await actor.close();
  }, skip: skip);

  test('closing_the_client_leaves_the_daemon_running', () async {
    final client = await daemon.attach();
    await client.close();
    final again = await daemon.attach();
    final info = await again.call(req('engine', 'get_contract_info'));
    expect(info['type'], 'contract_info');
    await again.close();
  }, skip: skip);

  test('shutdown_stops_the_daemon_process_with_code_0', () async {
    final own = await RunningDaemon.start(binary!);
    final client = await own.attach();
    await client.shutdown();
    expect(await own.process.exitCode.timeout(const Duration(seconds: 20)), 0);
    own.dir.deleteSync(recursive: true);
  }, skip: skip);

  test('openEngineClient_in_auto_mode_attaches_to_that_daemon', () async {
    final deps = EngineOpeningDeps(
      env: const {},
      isLinuxDesktop: true,
      attachDaemon: (paths) async => RemoteEngineClient.attach(
        DaemonEndpoint(
          unixSocket: paths.socket,
          token: await readDaemonToken(paths.token),
        ),
        clientId: 'dart-test',
      ),
      launchDaemon: (profile, home) async => false,
      openEmbedded: () async =>
          throw StateError('le moteur embarqué ne doit pas servir'),
    );
    final opened = await openEngineClient(deps: deps, home: daemon.home);
    expect(opened.used, EngineMode.daemon);
    final info = await opened.client.call(req('engine', 'get_contract_info'));
    expect(info['type'], 'contract_info');
    await opened.client.close();
  }, skip: skip);
}
