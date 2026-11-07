import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart' show req;
import 'package:mira/engine/remote/remote_engine_client.dart';

import 'support/rig.dart';

/// Client attaché à un faux démon.
Future<RemoteEngineClient> attach(DaemonRig rig) => RemoteEngineClient.attach(
  rig.endpoint,
  clientId: 'c',
  channelFactory: rig.factory,
);

void main() {
  test('set_presence_sends_the_focus_and_waits_for_the_result', () async {
    final rig = DaemonRig();
    final client = await RemoteEngineClient.attach(
      rig.endpoint,
      clientId: 'c',
      channelFactory: rig.factory,
    );
    await client.setPresence(true);
    final frame = await rig.daemon.next('set_presence');
    expect(frame['focused'], isTrue);
    await client.setPresence(false);
    expect((await rig.daemon.next('set_presence'))['focused'], isFalse);
    await client.close();
  });

  test('retry_unlock_sends_the_host_operation', () async {
    final rig = DaemonRig();
    final client = await RemoteEngineClient.attach(
      rig.endpoint,
      clientId: 'c',
      channelFactory: rig.factory,
    );
    await client.retryUnlock();
    expect((await rig.daemon.next('host_op'))['op'], 'retry_unlock');
    await client.close();
  });

  test('the_last_presence_is_announced_again_after_a_reconnection', () async {
    final rig = DaemonRig();
    final client = await attach(rig);
    await client.setPresence(true);
    await rig.daemon.close();
    await Future<void>.delayed(const Duration(milliseconds: 20));
    final pending = client.call(req('a', 'a'));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    final second = rig.daemons.last;
    expect(identical(second, rig.daemons.first), isFalse);
    expect((await second.next('set_presence'))['focused'], isTrue);
    final call = await second.next('call');
    second.reply(call['id'] as int, {
      'ok': true,
      'data': {'type': 'ok'},
    });
    await pending;
    await client.close();
  });

  test('without_a_known_focus_nothing_is_announced_on_reconnection', () async {
    final rig = DaemonRig();
    final client = await attach(rig);
    await rig.daemon.close();
    await Future<void>.delayed(const Duration(milliseconds: 20));
    final pending = client.call(req('a', 'a'));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    final second = rig.daemons.last;
    final call = await second.next('call');
    second.reply(call['id'] as int, {
      'ok': true,
      'data': {'type': 'ok'},
    });
    await pending;
    expect(
      second.received.map((f) => f['type']),
      isNot(contains('set_presence')),
    );
    await client.close();
  });
}
