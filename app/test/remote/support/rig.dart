import 'dart:async';
import 'dart:io';

import 'package:mira/engine/remote/byte_channel.dart';
import 'package:mira/engine/remote/endpoint.dart';

import 'fake_daemon.dart';

/// Point d'accès factice : chaque connexion obtient un nouveau [FakeDaemon], gardé dans [daemons].
class DaemonRig {
  DaemonRig({this.token = 'tok', this.hostState = 'ready'});

  final String token;
  String hostState;
  final List<FakeDaemon> daemons = [];

  /// Instants (depuis la création du rig) de chaque connexion demandée.
  final List<Duration> attemptTimes = [];
  final Stopwatch _clock = Stopwatch()..start();

  /// Délai avant qu'une connexion aboutisse (démon lent à répondre).
  Duration connectDelay = Duration.zero;

  /// Si vrai, la prochaine connexion échoue comme un démon absent.
  bool refuseConnections = false;

  /// Nombre de connexions demandées.
  int get connects => daemons.length + _refused;
  int _refused = 0;

  /// Dernier faux démon connecté.
  FakeDaemon get daemon => daemons.last;

  /// Point d'accès que le client ne sait pas distinguer d'un vrai.
  DaemonEndpoint get endpoint =>
      DaemonEndpoint(unixSocket: '/tmp/fake.sock', token: token);

  /// Fabrique de canaux à passer à `RemoteEngineClient.attach`.
  Future<ByteChannel> factory() async {
    attemptTimes.add(_clock.elapsed);
    if (connectDelay > Duration.zero) await Future<void>.delayed(connectDelay);
    if (refuseConnections) {
      _refused++;
      throw const SocketException('connexion refusée');
    }
    final (channel, daemon) = FakeDaemon.pair(
      token: token,
      hostState: hostState,
    );
    daemons.add(daemon);
    return channel;
  }
}
