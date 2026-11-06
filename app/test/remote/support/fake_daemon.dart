import 'dart:async';

import 'package:mira/engine/remote/byte_channel.dart';
import 'package:mira/engine/remote/frames.dart';

import 'memory_channel.dart';

/// Faux démon : vérifie `hello` comme le vrai, répond `pong`, et laisse le test lire et pousser des trames.
class FakeDaemon {
  FakeDaemon._(this._channel, this.token, this.hostState, this.contractMinor) {
    _channel.incoming.listen(_onBytes, onDone: () => closedByClient = true);
  }

  final MemoryChannel _channel;
  final FrameDecoder _decoder = FrameDecoder();
  final String token;

  /// Valeur de `host_state` annoncée.
  String hostState;
  final int contractMinor;

  /// Toutes les trames reçues, dans l'ordre.
  final List<Map<String, dynamic>> received = [];
  final List<Map<String, dynamic>> _unconsumed = [];
  final List<(String, Completer<Map<String, dynamic>>)> _waiters = [];

  /// Vrai : ne répond plus à rien (ni `hello`, ni `ping`).
  bool mute = false;

  /// Faux : ne répond plus aux `ping`.
  bool autoPong = true;

  /// Vrai quand le client a fermé son côté.
  bool closedByClient = false;

  /// Un canal client et le faux démon qui lui répond.
  static (ByteChannel, FakeDaemon) pair({
    String token = 'tok',
    String hostState = 'ready',
    int contractMinor = 6,
    bool mute = false,
  }) {
    final (client, server) = MemoryChannel.pair();
    final daemon = FakeDaemon._(server, token, hostState, contractMinor)
      ..mute = mute;
    return (client, daemon);
  }

  void _onBytes(List<int> bytes) {
    for (final frame in _decoder.add(bytes)) {
      received.add(frame);
      _answer(frame);
      _deliver(frame);
    }
  }

  void _deliver(Map<String, dynamic> frame) {
    final index = _waiters.indexWhere((w) => w.$1 == frame['type']);
    if (index < 0) {
      _unconsumed.add(frame);
      return;
    }
    _waiters.removeAt(index).$2.complete(frame);
  }

  void _answer(Map<String, dynamic> frame) {
    if (mute) return;
    switch (frame['type']) {
      case 'hello':
        push(_helloReply(frame));
      case 'ping' when autoPong:
        push({'type': 'pong', 'nonce': frame['nonce']});
    }
  }

  Map<String, dynamic> _helloReply(Map<String, dynamic> hello) {
    final auth = hello['auth'] as Map<String, dynamic>;
    final protocol = hello['protocol_version'] as Map<String, dynamic>;
    final contract = hello['contract_version'] as Map<String, dynamic>;
    String? refusal;
    if (auth['token'] != token) {
      refusal = 'auth';
    } else if (protocol['major'] != 1) {
      refusal = 'protocol_version';
    } else if (contract['major'] != 1 ||
        (contract['minor'] as int) > contractMinor) {
      refusal = 'contract_version';
    }
    if (refusal != null) return {'type': 'hello_err', 'reason': refusal};
    return {
      'type': 'hello_ok',
      'protocol_version': {'major': 1, 'minor': 0},
      'contract_version': {'major': 1, 'minor': contractMinor},
      'host_state': hostState,
    };
  }

  /// Prochaine trame reçue de ce type (celles déjà reçues et non lues d'abord).
  Future<Map<String, dynamic>> next(String type) {
    final index = _unconsumed.indexWhere((f) => f['type'] == type);
    if (index >= 0) return Future.value(_unconsumed.removeAt(index));
    final waiter = Completer<Map<String, dynamic>>();
    _waiters.add((type, waiter));
    return waiter.future;
  }

  /// Envoie une trame au client.
  void push(Map<String, dynamic> frame) => _channel.send(encodeFrame(frame));

  /// Répond `result` à l'appel `id`.
  void reply(int id, Map<String, dynamic> envelope) =>
      push({'type': 'result', 'id': id, 'envelope': envelope});

  /// Envoie des octets bruts (trame de test illisible ou découpée).
  void pushRaw(List<int> bytes) => _channel.send(bytes);

  /// Ferme la connexion côté démon.
  Future<void> close() => _channel.close();
}
