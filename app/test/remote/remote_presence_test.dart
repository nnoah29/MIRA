import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/remote_engine_client.dart';

import 'support/rig.dart';

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
}
