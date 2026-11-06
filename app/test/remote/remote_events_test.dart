import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/remote/event_stream.dart';
import 'package:mira/engine/remote/remote_engine_client.dart';

import 'support/fake_daemon.dart';
import 'support/rig.dart';

const _fast = Duration(milliseconds: 10);

/// Client attaché au rig, avec une attente de reconnexion courte.
Future<RemoteEngineClient> attached(
  DaemonRig rig, {
  Duration backoff = _fast,
}) => RemoteEngineClient.attach(
  rig.endpoint,
  clientId: 'c1',
  channelFactory: rig.factory,
  eventBackoff: backoff,
);

/// Pousse les événements `from`…`to` du flux `id`, dans la limite des crédits donnés par le client.
Future<void> feed(
  FakeDaemon daemon,
  int id, {
  required int from,
  required int to,
  required int credits,
}) async {
  var credit = credits;
  var seq = from;
  while (seq <= to) {
    if (credit == 0) {
      credit += (await daemon.next('credit'))['n'] as int;
      continue;
    }
    credit--;
    daemon.push({
      'type': 'event',
      'id': id,
      'event': {'seq': seq++},
    });
    if (seq % 50 == 0) await Future<void>.delayed(Duration.zero);
  }
}

/// Numéros `seq` d'une liste d'événements.
List<int> seqs(List<Map<String, dynamic>> events) => [
  for (final e in events) e['seq'] as int,
];

void main() {
  test('events_open_with_a_credit_window_and_deliver_event_objects', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final received = <Map<String, dynamic>>[];
    final sub = client.events().listen(received.add);
    final open = await rig.daemon.next('open_events');
    expect(open['credit'], 256);
    expect(open['after'], isNull);
    rig.daemon.push({
      'type': 'event',
      'id': open['id'],
      'event': {'seq': 1, 'kind': 'x'},
    });
    await Future<void>.delayed(const Duration(milliseconds: 20));
    expect(received, [
      {'seq': 1, 'kind': 'x'},
    ]);
    await sub.cancel();
    await client.close();
  });

  test('after_is_sent_when_given', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final sub = client.events(after: 5).listen((_) {});
    expect((await rig.daemon.next('open_events'))['after'], 5);
    await sub.cancel();
    await client.close();
  });

  test(
    'credit_is_replenished_every_128_events_so_a_long_stream_never_blocks',
    () async {
      final rig = DaemonRig();
      final client = await attached(rig);
      final received = <Map<String, dynamic>>[];
      final sub = client.events().listen(received.add);
      final open = await rig.daemon.next('open_events');
      await feed(
        rig.daemon,
        open['id'] as int,
        from: 1,
        to: 1000,
        credits: 256,
      );
      await Future<void>.delayed(const Duration(milliseconds: 50));
      expect(seqs(received), [for (var i = 1; i <= 1000; i++) i]);
      await sub.cancel();
      await client.close();
    },
    timeout: const Timeout(Duration(seconds: 10)),
  );

  test('a_lagged_end_reopens_with_after_set_to_the_last_seq_without_loss_or_duplicate', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final received = <Map<String, dynamic>>[];
    final sub = client.events().listen(received.add);
    final first = await rig.daemon.next('open_events');
    await feed(rig.daemon, first['id'] as int, from: 1, to: 40, credits: 256);
    rig.daemon.push({
      'type': 'end',
      'id': first['id'],
      'reason': {'type': 'lagged'},
    });
    final second = await rig.daemon.next('open_events');
    expect(second['after'], 40);
    await feed(
      rig.daemon,
      second['id'] as int,
      from: 41,
      to: 100,
      credits: 256,
    );
    await Future<void>.delayed(const Duration(milliseconds: 50));
    expect(seqs(received), [for (var i = 1; i <= 100; i++) i]);
    expect(rig.connects, 1, reason: 'la reprise reste sur la même connexion');
    await sub.cancel();
    await client.close();
  });

  test(
    'a_dropped_connection_reconnects_and_resumes_after_the_last_seq',
    () async {
      final rig = DaemonRig();
      final client = await attached(rig);
      final received = <Map<String, dynamic>>[];
      final sub = client.events().listen(received.add);
      final first = await rig.daemon.next('open_events');
      await feed(rig.daemon, first['id'] as int, from: 1, to: 3, credits: 256);
      await Future<void>.delayed(const Duration(milliseconds: 20));
      final dropped = rig.daemon;
      await dropped.close();
      while (rig.daemons.length < 2) {
        await Future<void>.delayed(const Duration(milliseconds: 10));
      }
      final second = await rig.daemon.next('open_events');
      expect(second['after'], 3);
      await feed(rig.daemon, second['id'] as int, from: 4, to: 6, credits: 256);
      await Future<void>.delayed(const Duration(milliseconds: 30));
      expect(seqs(received), [1, 2, 3, 4, 5, 6]);
      await sub.cancel();
      await client.close();
    },
  );

  test('backoff_doubles_up_to_five_seconds', () {
    var wait = const Duration(milliseconds: 500);
    final steps = <int>[];
    for (var i = 0; i < 7; i++) {
      steps.add(wait.inMilliseconds);
      wait = EventPump.nextBackoff(wait);
    }
    expect(steps, [500, 1000, 2000, 4000, 5000, 5000, 5000]);
  });

  test('retries_back_off_and_the_wait_resets_after_an_event', () async {
    final rig = DaemonRig();
    final client = await attached(
      rig,
      backoff: const Duration(milliseconds: 40),
    );
    final received = <Map<String, dynamic>>[];
    final sub = client.events().listen(received.add);
    await rig.daemon.next('open_events');
    rig.refuseConnections = true;
    await rig.daemon.close();
    while (rig.attemptTimes.length < 5) {
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    final times = rig.attemptTimes;
    int gap(int i) => (times[i] - times[i - 1]).inMilliseconds;
    expect(gap(2), greaterThanOrEqualTo(70), reason: '40 ms doublé : $times');
    expect(gap(3), greaterThanOrEqualTo(150), reason: '80 ms doublé : $times');
    rig.refuseConnections = false;
    while (rig.daemons.length < 2) {
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    final open = await rig.daemon.next('open_events');
    rig.daemon.push({
      'type': 'event',
      'id': open['id'],
      'event': {'seq': 1},
    });
    await Future<void>.delayed(const Duration(milliseconds: 30));
    expect(received, hasLength(1));
    rig.refuseConnections = true;
    final before = times.length;
    await rig.daemon.close();
    while (times.length < before + 1) {
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    expect(gap(before), lessThan(gap(3)), reason: 'repart de 40 ms : $times');
    await sub.cancel();
    await client.close();
  });

  test(
    'cancelling_the_subscription_sends_cancel_and_stops_reconnecting',
    () async {
      final rig = DaemonRig();
      final client = await attached(rig);
      final sub = client.events().listen((_) {});
      final open = await rig.daemon.next('open_events');
      await sub.cancel();
      final cancel = await rig.daemon.next('cancel');
      expect(cancel['id'], open['id']);
      final connects = rig.connects;
      await rig.daemon.close();
      await Future<void>.delayed(const Duration(milliseconds: 100));
      expect(rig.connects, connects);
      await client.close();
    },
  );

  test('an_error_end_is_delivered_as_a_stream_error_then_closes', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final stream = client.events();
    final errors = <Object>[];
    final done = Completer<void>();
    stream.listen((_) {}, onError: errors.add, onDone: done.complete);
    final open = await rig.daemon.next('open_events');
    rig.daemon.push({
      'type': 'end',
      'id': open['id'],
      'reason': {'type': 'error', 'code': 'internal.bug', 'message': 'boum'},
    });
    await done.future.timeout(const Duration(seconds: 2));
    expect(
      errors.single,
      isA<EngineException>().having((e) => e.code, 'code', 'internal.bug'),
    );
    await client.close();
  });

  test('events_end_after_the_client_is_closed_and_stop_reconnecting', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final done = Completer<void>();
    client.events().listen((_) {}, onDone: done.complete);
    await rig.daemon.next('open_events');
    await client.close();
    await done.future.timeout(const Duration(seconds: 2));
    final connects = rig.connects;
    await Future<void>.delayed(const Duration(milliseconds: 100));
    expect(rig.connects, connects);
  });

  test('cancelling_during_a_reconnection_never_reopens_a_stream', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final sub = client.events().listen((_) {});
    await rig.daemon.next('open_events');
    rig.connectDelay = const Duration(milliseconds: 120);
    await rig.daemon.close();
    await Future<void>.delayed(const Duration(milliseconds: 60));
    await sub.cancel();
    await Future<void>.delayed(const Duration(milliseconds: 250));
    expect(rig.daemons, hasLength(2), reason: 'la reconnexion en vol a abouti');
    expect(
      rig.daemons.last.received.where((f) => f['type'] == 'open_events'),
      isEmpty,
    );
    await client.close();
  });
}
