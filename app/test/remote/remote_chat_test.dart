import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/remote/remote_engine_client.dart';

import 'support/fake_daemon.dart';
import 'support/rig.dart';

/// Client attaché au rig.
Future<RemoteEngineClient> attached(DaemonRig rig) => RemoteEngineClient.attach(
  rig.endpoint,
  clientId: 'c1',
  channelFactory: rig.factory,
);

/// Pousse `count` lignes `text` du flux `id` dans la limite des crédits donnés.
Future<void> feed(
  FakeDaemon daemon,
  int id,
  int count, {
  required int credits,
}) async {
  var credit = credits;
  var n = 0;
  while (n < count) {
    if (credit == 0) {
      credit += (await daemon.next('credit'))['n'] as int;
      continue;
    }
    credit--;
    daemon.push({
      'type': 'chat_delta',
      'id': id,
      'line': {'type': 'text', 'text': '${n++}'},
    });
  }
}

void main() {
  test(
    'lines_arrive_in_order_then_the_stream_closes_on_end_finished',
    () async {
      final rig = DaemonRig();
      final client = await attached(rig);
      final lines = client.chatStream({'request': {}}).toList();
      final open = await rig.daemon.next('open_chat');
      expect(open['credit'], 64);
      final id = open['id'];
      for (final line in [
        {'type': 'started', 'operation': 'op1'},
        {'type': 'text', 'text': 'Bon'},
        {'type': 'done', 'reason': 'end_turn'},
      ]) {
        rig.daemon.push({'type': 'chat_delta', 'id': id, 'line': line});
      }
      rig.daemon.push({
        'type': 'end',
        'id': id,
        'reason': {'type': 'finished'},
      });
      final got = await lines;
      expect([for (final l in got) l['type']], ['started', 'text', 'done']);
      await client.close();
    },
  );

  test('credit_is_replenished_every_32_lines', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final lines = <Map<String, dynamic>>[];
    final sub = client.chatStream({}).listen(lines.add);
    final open = await rig.daemon.next('open_chat');
    await feed(rig.daemon, open['id'] as int, 300, credits: 64);
    await Future<void>.delayed(const Duration(milliseconds: 50));
    expect(lines, hasLength(300));
    final credits = rig.daemon.received.where((f) => f['type'] == 'credit');
    expect(credits, isNotEmpty);
    expect(credits.every((f) => f['n'] == 32), isTrue);
    await sub.cancel();
    await client.close();
  }, timeout: const Timeout(Duration(seconds: 10)));

  test('cancelling_the_subscription_sends_cancel', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final sub = client.chatStream({}).listen((_) {});
    final open = await rig.daemon.next('open_chat');
    await sub.cancel();
    final cancel = await rig.daemon.next('cancel');
    expect(cancel['id'], open['id']);
    await client.close();
  });

  test('no_line_is_delivered_after_cancel', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final lines = <Map<String, dynamic>>[];
    final sub = client.chatStream({}).listen(lines.add);
    final open = await rig.daemon.next('open_chat');
    rig.daemon.push({
      'type': 'chat_delta',
      'id': open['id'],
      'line': {'n': 1},
    });
    await Future<void>.delayed(const Duration(milliseconds: 20));
    await sub.cancel();
    rig.daemon.push({
      'type': 'chat_delta',
      'id': open['id'],
      'line': {'n': 2},
    });
    rig.daemon.push({
      'type': 'end',
      'id': open['id'],
      'reason': {'type': 'cancelled'},
    });
    await Future<void>.delayed(const Duration(milliseconds: 20));
    expect(lines, [
      {'n': 1},
    ]);
    await client.close();
  });

  test('end_error_becomes_an_engine_exception', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final errors = <Object>[];
    final done = Completer<void>();
    client
        .chatStream({})
        .listen((_) {}, onError: errors.add, onDone: done.complete);
    final open = await rig.daemon.next('open_chat');
    rig.daemon.push({
      'type': 'end',
      'id': open['id'],
      'reason': {
        'type': 'error',
        'code': 'server.ai_unavailable',
        'message': 'x',
      },
    });
    await done.future.timeout(const Duration(seconds: 2));
    expect(
      errors.single,
      isA<EngineException>().having(
        (e) => e.code,
        'code',
        'server.ai_unavailable',
      ),
    );
    await client.close();
  });

  test('a_dropped_connection_mid_chat_raises_transport_disconnected_and_does_not_reconnect', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final errors = <Object>[];
    final done = Completer<void>();
    client
        .chatStream({})
        .listen((_) {}, onError: errors.add, onDone: done.complete);
    await rig.daemon.next('open_chat');
    await rig.daemon.close();
    await done.future.timeout(const Duration(seconds: 2));
    expect(
      errors.single,
      isA<EngineException>().having(
        (e) => e.code,
        'code',
        'transport.disconnected',
      ),
    );
    await Future<void>.delayed(const Duration(milliseconds: 50));
    expect(rig.connects, 1);
    await client.close();
  });

  test('an_unknown_start_object_is_forwarded_as_is', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final start = {
      'connection': null,
      'weird': {
        'nested': [1, 2],
      },
    };
    final sub = client.chatStream(start).listen((_) {});
    final open = await rig.daemon.next('open_chat');
    expect(open['request'], start);
    await sub.cancel();
    await client.close();
  });

  test('the_stream_works_through_the_ChatStreamingClient_interface', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final EngineClient plain = client;
    expect(plain is ChatStreamingClient, isTrue);
    final sub = (plain as ChatStreamingClient).chatStream({}).listen((_) {});
    await rig.daemon.next('open_chat');
    await sub.cancel();
    await client.close();
  });

  test(
    'a_chat_delta_without_an_object_is_skipped_and_the_stream_goes_on',
    () async {
      final rig = DaemonRig();
      final client = await attached(rig);
      final lines = <Map<String, dynamic>>[];
      final sub = client.chatStream({}).listen(lines.add);
      final open = await rig.daemon.next('open_chat');
      rig.daemon.push({'type': 'chat_delta', 'id': open['id'], 'line': null});
      rig.daemon.push({
        'type': 'chat_delta',
        'id': open['id'],
        'line': {'n': 1},
      });
      await Future<void>.delayed(const Duration(milliseconds: 30));
      expect(lines, [
        {'n': 1},
      ]);
      await sub.cancel();
      await client.close();
    },
  );

  test('chat_credits_are_withheld_while_the_subscriber_is_paused_and_returned_on_resume', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final lines = <Map<String, dynamic>>[];
    final sub = client.chatStream({}).listen(lines.add);
    final open = await rig.daemon.next('open_chat');
    sub.pause();
    final fed = feed(rig.daemon, open['id'] as int, 300, credits: 64);
    await Future<void>.delayed(const Duration(milliseconds: 150));
    expect(rig.daemon.received.where((f) => f['type'] == 'credit'), isEmpty);
    sub.resume();
    await fed.timeout(const Duration(seconds: 5));
    await Future<void>.delayed(const Duration(milliseconds: 100));
    expect(lines, hasLength(300));
    await sub.cancel();
    await client.close();
  }, timeout: const Timeout(Duration(seconds: 10)));
}
