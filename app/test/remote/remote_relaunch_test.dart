import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/remote/remote_engine_client.dart';

import 'support/rig.dart';

/// Client attaché à un faux démon.
Future<RemoteEngineClient> attach(DaemonRig rig) => RemoteEngineClient.attach(
  rig.endpoint,
  clientId: 'c',
  channelFactory: rig.factory,
);

/// Répond `ok` au prochain appel reçu par le dernier faux démon.
Future<void> answer(DaemonRig rig) async {
  final call = await rig.daemons.last.next('call');
  rig.daemons.last.reply(call['id'] as int, {
    'ok': true,
    'data': {'type': 'ok'},
  });
}

/// Perd le démon : la connexion tombe et plus aucune ne se fait.
Future<void> lose(DaemonRig rig) async {
  await rig.daemon.close();
  rig.refuseConnections = true;
  await Future<void>.delayed(const Duration(milliseconds: 20));
}

void main() {
  test('a_lost_daemon_is_relaunched_by_the_hook_on_the_next_call', () async {
    final rig = DaemonRig();
    final client = await attach(rig);
    var relaunches = 0;
    client.relaunch = () async {
      relaunches++;
      rig.refuseConnections = false;
      return true;
    };
    await lose(rig);
    final pending = client.call(req('a', 'a'));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    await answer(rig);
    expect(await pending, {'type': 'ok'});
    expect(relaunches, 1);
    await client.close();
  });

  test('a_hook_that_fails_leaves_the_engine_unavailable', () async {
    final rig = DaemonRig();
    final client = await attach(rig);
    client.relaunch = () async => false;
    await lose(rig);
    await expectLater(
      client.call(req('a', 'a')),
      throwsA(
        isA<EngineException>().having(
          (e) => e.code,
          'code',
          'transport.unavailable',
        ),
      ),
    );
    await client.close();
  });

  test('simultaneous_calls_share_a_single_relaunch', () async {
    final rig = DaemonRig();
    final client = await attach(rig);
    var relaunches = 0;
    client.relaunch = () async {
      relaunches++;
      rig.refuseConnections = false;
      return true;
    };
    await lose(rig);
    final a = client.call(req('a', 'a'));
    final b = client.call(req('b', 'b'));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    await answer(rig);
    await answer(rig);
    await Future.wait([a, b]);
    expect(relaunches, 1);
    await client.close();
  });
}
