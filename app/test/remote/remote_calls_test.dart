import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/remote/remote_engine_client.dart';

import 'support/rig.dart';

/// Client attaché au rig, avec un jeton valide.
Future<RemoteEngineClient> attached(DaemonRig rig) => RemoteEngineClient.attach(
  rig.endpoint,
  clientId: 'c1',
  channelFactory: rig.factory,
);

/// Répond à l'appel suivant du dernier démon avec ces données.
Future<void> answerNext(DaemonRig rig, Map<String, dynamic> data) async {
  final call = await rig.daemon.next('call');
  rig.daemon.reply(call['id'] as int, {
    'contract_version': {'major': 1, 'minor': 6},
    'ok': true,
    'data': data,
  });
}

void main() {
  test('call_returns_the_response_object_of_the_envelope', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final future = client.call(req('engine', 'get_contract_info'));
    await answerNext(rig, {
      'type': 'contract_info',
      'data': {'v': 1},
    });
    expect(await future, {
      'type': 'contract_info',
      'data': {'v': 1},
    });
    final sent = rig.daemon.received.firstWhere((f) => f['type'] == 'call');
    expect(sent['request'], req('engine', 'get_contract_info'));
    await client.close();
  });

  test(
    'an_error_envelope_becomes_an_engine_exception_with_code_and_message',
    () async {
      final rig = DaemonRig();
      final client = await attached(rig);
      final future = client.call(req('x', 'y'));
      final call = await rig.daemon.next('call');
      rig.daemon.reply(call['id'] as int, {
        'ok': false,
        'error': {'code': 'not_found.account', 'message': 'introuvable'},
      });
      await expectLater(
        future,
        throwsA(
          isA<EngineException>()
              .having((e) => e.code, 'code', 'not_found.account')
              .having((e) => e.message, 'message', 'introuvable'),
        ),
      );
      await client.close();
    },
  );

  test('concurrent_calls_are_matched_by_id', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final a = client.call(req('a', 'a'));
    final b = client.call(req('b', 'b'));
    final first = await rig.daemon.next('call');
    final second = await rig.daemon.next('call');
    for (final call in [second, first]) {
      final domain = call['request']['domain'];
      rig.daemon.reply(call['id'] as int, {
        'ok': true,
        'data': {'domain': domain},
      });
    }
    expect((await a)['domain'], 'a');
    expect((await b)['domain'], 'b');
    await client.close();
  });

  test('original_html_returns_the_html_or_null', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final html = client.originalHtml('m1');
    var frame = await rig.daemon.next('get_original_html');
    expect(frame['message'], 'm1');
    rig.daemon.reply(frame['id'] as int, {
      'ok': true,
      'data': {'html': '<p>x</p>'},
    });
    expect(await html, '<p>x</p>');
    final none = client.originalHtml('m2');
    frame = await rig.daemon.next('get_original_html');
    rig.daemon.reply(frame['id'] as int, {
      'ok': true,
      'data': {'html': null},
    });
    expect(await none, isNull);
    await client.close();
  });

  test('close_detaches_without_sending_a_host_op', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    await client.close();
    await Future<void>.delayed(const Duration(milliseconds: 20));
    expect(rig.daemon.received.where((f) => f['type'] == 'host_op'), isEmpty);
    expect(rig.daemon.closedByClient, isTrue);
  });

  test('shutdown_sends_host_op_shutdown_then_detaches', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    final done = client.shutdown();
    final op = await rig.daemon.next('host_op');
    expect(op['op'], 'shutdown');
    rig.daemon.reply(op['id'] as int, {'ok': true, 'data': null});
    await done;
    await Future<void>.delayed(const Duration(milliseconds: 20));
    expect(rig.daemon.closedByClient, isTrue);
  });

  test(
    'a_call_in_flight_fails_with_transport_disconnected_when_the_daemon_drops',
    () async {
      final rig = DaemonRig();
      final client = await attached(rig);
      final future = client.call(req('a', 'a'));
      final failed = expectLater(
        future,
        throwsA(
          isA<EngineException>().having(
            (e) => e.code,
            'code',
            'transport.disconnected',
          ),
        ),
      );
      await rig.daemon.next('call');
      await rig.daemon.close();
      await failed;
    },
  );

  test('the_next_call_reconnects_once_and_succeeds', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    await rig.daemon.close();
    await Future<void>.delayed(const Duration(milliseconds: 20));
    final future = client.call(req('a', 'a'));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    expect(rig.connects, 2);
    await answerNext(rig, {'type': 'ok'});
    expect(await future, {'type': 'ok'});
    await client.close();
  });

  test(
    'two_simultaneous_calls_after_a_drop_open_a_single_connection',
    () async {
      final rig = DaemonRig();
      final client = await attached(rig);
      await rig.daemon.close();
      await Future<void>.delayed(const Duration(milliseconds: 20));
      final a = client.call(req('a', 'a'));
      final b = client.call(req('b', 'b'));
      await answerNext(rig, {'n': 1});
      await answerNext(rig, {'n': 2});
      await Future.wait([a, b]);
      expect(rig.connects, 2);
      await client.close();
    },
  );

  test('reconnection_failure_raises_transport_unavailable', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    await rig.daemon.close();
    await Future<void>.delayed(const Duration(milliseconds: 20));
    rig.refuseConnections = true;
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
  });

  test('a_call_after_close_fails_with_a_clear_error', () async {
    final rig = DaemonRig();
    final client = await attached(rig);
    await client.close();
    await expectLater(
      client.call(req('a', 'a')),
      throwsA(
        isA<EngineException>().having(
          (e) => e.code,
          'code',
          'transport.closed',
        ),
      ),
    );
    expect(rig.connects, 1);
  });

  test(
    'an_unreachable_daemon_at_attach_time_raises_the_underlying_error',
    () async {
      final rig = DaemonRig()..refuseConnections = true;
      await expectLater(attached(rig), throwsA(anything));
    },
  );

  test(
    'a_request_too_large_to_encode_fails_cleanly_and_the_client_stays_usable',
    () async {
      final rig = DaemonRig();
      final client = await attached(rig);
      final huge = {'blob': 'x' * (17 * 1024 * 1024)};
      await expectLater(
        client.call(huge),
        throwsA(
          isA<EngineException>().having(
            (e) => e.code,
            'code',
            'transport.request_too_large',
          ),
        ),
      );
      final next = client.call(req('a', 'a'));
      await answerNext(rig, {'type': 'ok'});
      expect(await next, {'type': 'ok'});
      await client.close();
    },
  );

  test('reconnection_rereads_the_token_so_a_restarted_daemon_with_a_new_token_is_reached', () async {
    final rig = DaemonRig(token: 'old');
    var current = 'old';
    final client = await RemoteEngineClient.attach(
      rig.endpoint,
      clientId: 'c1',
      channelFactory: rig.factory,
      tokenProvider: () async => current,
    );
    await rig.daemon.close();
    await Future<void>.delayed(const Duration(milliseconds: 20));
    rig.token = 'new';
    current = 'new';
    final future = client.call(req('a', 'a'));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    await answerNext(rig, {'n': 1});
    expect(await future, {'n': 1});
    await client.close();
  });

  test(
    'a_refusal_after_a_restart_is_reported_precisely_not_as_unreachable',
    () async {
      final rig = DaemonRig(token: 'old');
      final client = await RemoteEngineClient.attach(
        rig.endpoint,
        clientId: 'c1',
        channelFactory: rig.factory,
        tokenProvider: () async => 'old',
      );
      await rig.daemon.close();
      await Future<void>.delayed(const Duration(milliseconds: 20));
      rig.token = 'rotated';
      await expectLater(
        client.call(req('a', 'a')),
        throwsA(
          isA<EngineException>().having(
            (e) => e.code,
            'code',
            'host.auth_refused',
          ),
        ),
      );
    },
  );
}
