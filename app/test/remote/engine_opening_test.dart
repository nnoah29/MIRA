import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/remote/connection.dart';
import 'package:mira/engine/remote/daemon_paths.dart';
import 'package:mira/engine/remote/engine_opening.dart';
import 'package:mira/engine/remote/remote_engine_client.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'support/rig.dart';

/// Moteur embarqué factice.
class EmbeddedClient implements EngineClient {
  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}

/// Monde factice : un démon vivant ou non, des appels comptés, un temps simulé par les pauses.
class World {
  final DaemonRig rig = DaemonRig();
  bool alive = false;
  Object? attachError;
  bool binaryExists = false;
  bool launchMakesItAlive = true;
  int attaches = 0;
  int launches = 0;
  int embedded = 0;
  Duration paused = Duration.zero;

  /// Après autant d'attaches, l'état annoncé devient `ready` (-1 : jamais).
  int readyAfter = 0;

  /// Dépendances branchées sur ce monde.
  EngineOpeningDeps deps({bool desktop = true}) => EngineOpeningDeps(
    env: const {'MIRA_HOME': '/m'},
    isLinuxDesktop: desktop,
    attachDaemon: (DaemonPaths paths) async {
      attaches++;
      if (readyAfter >= 0 && attaches > readyAfter) rig.hostState = 'ready';
      final error = attachError;
      if (error != null) throw error;
      if (!alive) throw const DaemonAbsent();
      return RemoteEngineClient.attach(
        rig.endpoint,
        clientId: 'c',
        channelFactory: rig.factory,
      );
    },
    launchDaemon: (profile, home) async {
      launches++;
      if (!binaryExists) return false;
      alive = launchMakesItAlive;
      return true;
    },
    openEmbedded: () async {
      embedded++;
      return EmbeddedClient();
    },
    pause: (d) async => paused += d,
  );
}

Matcher failsWith(String code) =>
    isA<EngineException>().having((e) => e.code, 'code', code);

void main() {
  test('embedded_mode_never_touches_the_daemon', () async {
    final world = World()..alive = true;
    final opened = await openEngineClient(
      deps: world.deps(),
      mode: EngineMode.embedded,
    );
    expect(opened.used, EngineMode.embedded);
    expect(world.attaches + world.launches, 0);
  });

  test('non_desktop_platforms_always_use_the_embedded_engine', () async {
    final world = World()..alive = true;
    final opened = await openEngineClient(
      deps: world.deps(desktop: false),
      mode: EngineMode.daemon,
    );
    expect(opened.used, EngineMode.embedded);
    expect(world.attaches, 0);
  });

  test('auto_attaches_to_a_live_daemon', () async {
    final world = World()..alive = true;
    final opened = await openEngineClient(deps: world.deps());
    expect(opened.used, EngineMode.daemon);
    expect(opened.client, isA<RemoteEngineClient>());
    expect(world.launches, 0);
    expect(world.embedded, 0);
  });

  test(
    'auto_launches_the_daemon_when_absent_and_a_binary_exists_then_attaches',
    () async {
      final world = World()..binaryExists = true;
      final opened = await openEngineClient(deps: world.deps());
      expect(opened.used, EngineMode.daemon);
      expect(world.launches, 1);
    },
  );

  test('auto_falls_back_to_embedded_when_no_daemon_and_no_binary', () async {
    final world = World();
    final opened = await openEngineClient(deps: world.deps());
    expect(opened.used, EngineMode.embedded);
    expect(world.embedded, 1);
  });

  test('auto_falls_back_to_embedded_when_the_launched_daemon_never_answers_within_20_seconds', () async {
    final world = World()
      ..binaryExists = true
      ..launchMakesItAlive = false;
    final opened = await openEngineClient(deps: world.deps());
    expect(opened.used, EngineMode.embedded);
    expect(world.paused, greaterThanOrEqualTo(const Duration(seconds: 20)));
    expect(world.paused, lessThan(const Duration(seconds: 21)));
  });

  test('daemon_mode_reports_the_failure_instead_of_falling_back', () async {
    final world = World();
    await expectLater(
      openEngineClient(deps: world.deps(), mode: EngineMode.daemon),
      throwsA(failsWith('transport.unavailable')),
    );
    expect(world.embedded, 0);
  });

  test('a_starting_daemon_is_polled_until_ready', () async {
    final world = World()
      ..alive = true
      ..readyAfter = 3;
    world.rig.hostState = 'starting';
    final opened = await openEngineClient(deps: world.deps());
    expect(opened.used, EngineMode.daemon);
    expect(world.attaches, 4);
    expect(world.paused, const Duration(milliseconds: 750));
  });

  test(
    'a_starting_daemon_that_never_gets_ready_fails_after_30_seconds',
    () async {
      final world = World()
        ..alive = true
        ..readyAfter = -1;
      world.rig.hostState = 'starting';
      await expectLater(
        openEngineClient(deps: world.deps()),
        throwsA(failsWith('transport.unavailable')),
      );
      expect(world.paused, greaterThanOrEqualTo(const Duration(seconds: 30)));
      expect(world.embedded, 0);
    },
  );

  test(
    'keyring_locked_raises_a_localized_exception_and_never_falls_back',
    () async {
      final world = World()..alive = true;
      world.rig.hostState = 'keyring_locked';
      world.readyAfter = -1;
      Object? error;
      try {
        await openEngineClient(deps: world.deps());
      } on EngineException catch (e) {
        error = e;
      }
      expect(error, failsWith('host.keyring_locked'));
      expect((error! as EngineException).localized, isTrue);
      expect(world.embedded, 0);
    },
  );

  test(
    'update_required_raises_a_localized_exception_and_never_falls_back',
    () async {
      final world = World()..alive = true;
      world.rig.hostState = 'update_required';
      world.readyAfter = -1;
      await expectLater(
        openEngineClient(deps: world.deps()),
        throwsA(failsWith('host.update_required')),
      );
      expect(world.embedded, 0);
    },
  );

  test('a_refused_token_never_falls_back_to_embedded', () async {
    final world = World()..attachError = const HandshakeRefused('auth');
    await expectLater(
      openEngineClient(deps: world.deps()),
      throwsA(failsWith('host.auth_refused')),
    );
    final other = World()
      ..attachError = const HandshakeRefused('contract_version');
    await expectLater(
      openEngineClient(deps: other.deps()),
      throwsA(failsWith('host.update_required')),
    );
    expect(world.embedded + other.embedded, 0);
  });

  test(
    'a_missing_token_file_with_a_live_socket_is_reported_not_hidden',
    () async {
      final world = World()
        ..attachError = EngineException.local('transport.token_missing', 'x');
      await expectLater(
        openEngineClient(deps: world.deps()),
        throwsA(failsWith('transport.token_missing')),
      );
      expect(world.embedded, 0);
    },
  );

  test('a_stale_socket_or_unreachable_daemon_counts_as_absent', () async {
    final world = World()..attachError = const SocketException('refusé');
    final opened = await openEngineClient(deps: world.deps());
    expect(opened.used, EngineMode.embedded);
  });

  test('the_client_id_is_generated_once_and_kept', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final first = loadClientId(prefs);
    expect(first, matches(RegExp(r'^[0-9a-f]{32}$')));
    expect(loadClientId(prefs), first);
    expect(prefs.getString('engineClientId'), first);
  });

  test('parse_engine_mode_defaults_to_auto', () {
    expect(parseEngineMode(null), EngineMode.auto);
    expect(parseEngineMode(''), EngineMode.auto);
    expect(parseEngineMode('embedded'), EngineMode.embedded);
    expect(parseEngineMode('daemon'), EngineMode.daemon);
    expect(parseEngineMode('nimporte'), EngineMode.auto);
  });
}
