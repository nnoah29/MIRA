import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/connection.dart';
import 'package:mira/engine/remote/frame_builders.dart';
import 'package:mira/engine/remote/host_errors.dart';

import 'support/fake_daemon.dart';

Future<RemoteConnection> open(dynamic channel, {String token = 'tok'}) =>
    RemoteConnection.open(
      channel,
      token: token,
      clientId: 'c1',
      clientName: 'tests',
      protocolVersion: clientProtocolVersion,
      contractVersion: appContractVersion,
    );

void main() {
  test(
    'un démon plein refuse la poignée de main avec la raison busy',
    () async {
      final (channel, _) = FakeDaemon.pair(busy: true);
      await expectLater(
        open(channel),
        throwsA(isA<HandshakeRefused>().having((e) => e.reason, 'r', 'busy')),
      );
    },
  );

  test('un mauvais jeton reçoit auth, jamais busy', () async {
    final (channel, _) = FakeDaemon.pair(token: 'good', busy: true);
    await expectLater(
      open(channel, token: 'bad'),
      throwsA(isA<HandshakeRefused>().having((e) => e.reason, 'r', 'auth')),
    );
  });

  test('busy devient une erreur lisible et distincte, sans relance', () {
    final error = refusalError(const HandshakeRefused('busy'));
    expect(error.code, 'host.busy');
    expect(error.message, isNot(unavailableError().message));
    expect(error.message, contains('MIRA'));
  });

  test('une raison inconnue reste le message générique', () {
    final error = refusalError(const HandshakeRefused('plus_tard'));
    expect(error.code, unavailableError().code);
  });
}
