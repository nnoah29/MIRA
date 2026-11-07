import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/byte_channel.dart';
import 'package:mira/engine/remote/connection.dart';
import 'package:mira/engine/remote/frame_builders.dart';
import 'package:mira/engine/remote/host_state.dart';

import 'support/fake_daemon.dart';

/// Ouvre une connexion sur un faux démon avec le jeton `tok`.
Future<RemoteConnection> open(
  ByteChannel channel, {
  String token = 'tok',
  Duration pingInterval = const Duration(seconds: 15),
  Duration helloTimeout = const Duration(seconds: 5),
  Map<String, int>? protocolVersion,
  Map<String, int>? contractVersion,
}) => RemoteConnection.open(
  channel,
  token: token,
  clientId: 'c1',
  clientName: 'tests',
  pingInterval: pingInterval,
  helloTimeout: helloTimeout,
  protocolVersion: protocolVersion ?? clientProtocolVersion,
  contractVersion: contractVersion ?? appContractVersion,
);

void main() {
  test('a_valid_handshake_reports_versions_and_host_state', () async {
    final (channel, daemon) = FakeDaemon.pair(hostState: 'starting');
    final connection = await open(channel);
    expect(connection.hostState, HostState.starting);
    expect(connection.serverVersions['contract_version'], {
      'major': 1,
      'minor': 6,
    });
    final hello = await daemon.next('hello');
    expect((hello['auth'] as Map)['token'], 'tok');
    await connection.close();
  });

  test(
    'a_wrong_token_is_refused_with_reason_auth_and_the_channel_closes',
    () async {
      final (channel, _) = FakeDaemon.pair(token: 'good');
      await expectLater(
        open(channel, token: 'bad'),
        throwsA(
          isA<HandshakeRefused>().having((e) => e.reason, 'reason', 'auth'),
        ),
      );
      await channel.done;
    },
  );

  test('incompatible_versions_are_refused_with_their_reason', () async {
    final (c1, _) = FakeDaemon.pair();
    await expectLater(
      open(c1, protocolVersion: {'major': 2, 'minor': 0}),
      throwsA(
        isA<HandshakeRefused>().having(
          (e) => e.reason,
          'r',
          'protocol_version',
        ),
      ),
    );
    final (c2, _) = FakeDaemon.pair();
    await expectLater(
      open(c2, contractVersion: {'major': 1, 'minor': 7}),
      throwsA(
        isA<HandshakeRefused>().having(
          (e) => e.reason,
          'r',
          'contract_version',
        ),
      ),
    );
  });

  test('a_silent_daemon_times_out_the_handshake', () {
    fakeAsync((async) {
      final (channel, _) = FakeDaemon.pair(mute: true);
      Object? error;
      open(channel).then<void>((_) {}, onError: (Object e) => error = e);
      async.elapse(const Duration(seconds: 6));
      expect(error, isA<ConnectionLost>());
    });
  });

  test('the_token_never_appears_in_exception_text', () async {
    final (channel, _) = FakeDaemon.pair(token: 'good');
    Object? refused;
    try {
      await open(channel, token: 'SECRET-TOKEN-123');
    } on Object catch (e) {
      refused = e;
    }
    expect('$refused', isNot(contains('SECRET-TOKEN-123')));
    expect(const ConnectionLost('x').toString(), isNot(contains('SECRET')));
  });

  test('results_are_matched_by_id_even_out_of_order', () async {
    final (channel, daemon) = FakeDaemon.pair();
    final connection = await open(channel);
    final futures = [
      for (var i = 0; i < 3; i++)
        connection.request(callFrame(connection.nextId(), {'n': i})),
    ];
    final calls = [for (var i = 0; i < 3; i++) await daemon.next('call')];
    for (final call in calls.reversed) {
      daemon.reply(call['id'] as int, {'echo': (call['request'] as Map)['n']});
    }
    final results = await Future.wait(futures);
    expect(
      [for (final r in results) (r['envelope'] as Map)['echo']],
      [0, 1, 2],
    );
    await connection.close();
  });

  test(
    'pending_requests_fail_with_connection_lost_when_the_channel_closes',
    () async {
      final (channel, daemon) = FakeDaemon.pair();
      final connection = await open(channel);
      final pending = connection.request(callFrame(connection.nextId(), {}));
      final failed = expectLater(pending, throwsA(isA<ConnectionLost>()));
      await daemon.next('call');
      await daemon.close();
      await failed;
      await connection.closed;
      await expectLater(
        connection.request(callFrame(connection.nextId(), {})),
        throwsA(isA<ConnectionLost>()),
      );
    },
  );

  test('unknown_frame_types_are_ignored', () async {
    final (channel, daemon) = FakeDaemon.pair();
    final connection = await open(channel);
    daemon.push({'type': 'teleport', 'id': 1});
    final result = connection.request(callFrame(connection.nextId(), {}));
    final call = await daemon.next('call');
    daemon.reply(call['id'] as int, {'ok': true});
    expect((await result)['envelope'], {'ok': true});
    await connection.close();
  });

  test('a_stream_delivers_its_frames_then_closes_after_end', () async {
    final (channel, daemon) = FakeDaemon.pair();
    final connection = await open(channel);
    final id = connection.nextId();
    final frames = connection
        .openStream(openEventsFrame(id, credit: 8))
        .toList();
    await daemon.next('open_events');
    daemon.push({
      'type': 'event',
      'id': id,
      'event': {'seq': 1},
    });
    daemon.push({
      'type': 'event',
      'id': id,
      'event': {'seq': 2},
    });
    daemon.push({
      'type': 'end',
      'id': id,
      'reason': {'type': 'finished'},
    });
    final got = await frames;
    expect([for (final f in got) f['type']], ['event', 'event', 'end']);
    await connection.close();
  });

  test('frames_of_another_id_never_reach_a_stream', () async {
    final (channel, daemon) = FakeDaemon.pair();
    final connection = await open(channel);
    final id = connection.nextId();
    final seen = <Map<String, dynamic>>[];
    final sub = connection
        .openStream(openEventsFrame(id, credit: 8))
        .listen(seen.add);
    await daemon.next('open_events');
    daemon.push({
      'type': 'event',
      'id': id + 99,
      'event': {'seq': 1},
    });
    daemon.push({
      'type': 'event',
      'id': id,
      'event': {'seq': 2},
    });
    await Future<void>.delayed(const Duration(milliseconds: 20));
    expect([for (final f in seen) (f['event'] as Map)['seq']], [2]);
    await sub.cancel();
    await connection.close();
  });

  test(
    'a_ping_is_sent_every_interval_and_a_pong_keeps_the_connection_alive',
    () {
      fakeAsync((async) {
        final (channel, daemon) = FakeDaemon.pair();
        RemoteConnection? connection;
        open(
          channel,
          pingInterval: const Duration(seconds: 10),
        ).then((c) => connection = c);
        async.elapse(const Duration(seconds: 55));
        expect(
          daemon.received.where((f) => f['type'] == 'ping').length,
          greaterThanOrEqualTo(4),
        );
        var closed = false;
        connection!.closed.then((_) => closed = true);
        async.flushMicrotasks();
        expect(closed, isFalse);
      });
    },
  );

  test('three_silent_intervals_close_the_connection', () {
    fakeAsync((async) {
      final (channel, daemon) = FakeDaemon.pair();
      RemoteConnection? connection;
      open(
        channel,
        pingInterval: const Duration(seconds: 10),
      ).then((c) => connection = c);
      async.flushMicrotasks();
      daemon.autoPong = false;
      var closed = false;
      connection!.closed.then((_) => closed = true);
      async.elapse(const Duration(seconds: 25));
      expect(closed, isFalse);
      async.elapse(const Duration(seconds: 20));
      expect(closed, isTrue);
    });
  });

  test('ids_grow_from_one', () async {
    final (channel, _) = FakeDaemon.pair();
    final connection = await open(channel);
    expect(
      [connection.nextId(), connection.nextId(), connection.nextId()],
      [1, 2, 3],
    );
    await connection.close();
  });

  test(
    'a_burst_of_ends_followed_by_a_close_raises_no_uncaught_error',
    () async {
      final uncaught = <Object>[];
      await runZonedGuarded(() async {
        final (channel, daemon) = FakeDaemon.pair();
        final connection = await open(channel);
        final ids = [for (var i = 0; i < 5; i++) connection.nextId()];
        final listeners = [
          for (final id in ids)
            connection
                .openStream(openEventsFrame(id, credit: 8))
                .listen((_) {}, onError: (_) {}),
        ];
        for (final id in ids) {
          daemon.push({
            'type': 'end',
            'id': id,
            'reason': {'type': 'finished'},
          });
          daemon.push({
            'type': 'event',
            'id': id,
            'event': {'seq': 1},
          });
        }
        await daemon.close();
        await connection.closed;
        await Future<void>.delayed(const Duration(milliseconds: 50));
        for (final l in listeners) {
          await l.cancel();
        }
      }, (error, _) => uncaught.add(error));
      expect(uncaught, isEmpty);
    },
  );
}
