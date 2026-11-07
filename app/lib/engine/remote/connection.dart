import 'dart:async';

import 'byte_channel.dart';
import 'frame_builders.dart';
import 'frames.dart';
import 'host_state.dart';

/// Le démon a refusé la poignée de main ; la raison est fixe et ne contient jamais le jeton.
class HandshakeRefused implements Exception {
  /// `protocol_version`, `contract_version`, `auth` ou `malformed`.
  final String reason;
  const HandshakeRefused(this.reason);

  @override
  String toString() => 'HandshakeRefused($reason)';
}

/// La connexion au démon est perdue ou n'a jamais été établie.
class ConnectionLost implements Exception {
  /// Cause courte : `closed`, `silence`, `protocol`, `handshake`…
  final String why;
  const ConnectionLost(this.why);

  @override
  String toString() => 'ConnectionLost($why)';
}

/// Connexion attachée à un démon : poignée de main faite, appariement des `result` et des flux par `id`.
class RemoteConnection {
  RemoteConnection._(this._channel, this._pingInterval);

  final ByteChannel _channel;
  final Duration _pingInterval;
  final FrameDecoder _decoder = FrameDecoder();
  final Completer<Map<String, dynamic>> _hello =
      Completer<Map<String, dynamic>>();
  final Map<int, Completer<Map<String, dynamic>>> _requests = {};
  final Map<int, StreamController<Map<String, dynamic>>> _streams = {};
  final Completer<void> _closed = Completer<void>();
  Timer? _keepAlive;
  int _nextId = 1;
  int _nonce = 0;
  int _silentTicks = 0;

  /// État de l'hôte annoncé par `hello_ok`.
  late final HostState hostState;

  /// Versions annoncées par le démon (`protocol_version`, `contract_version`).
  late final Map<String, dynamic> serverVersions;

  /// Fait la poignée de main sur ce canal ; ferme le canal si elle échoue.
  static Future<RemoteConnection> open(
    ByteChannel channel, {
    required String token,
    required String clientId,
    required String clientName,
    Duration helloTimeout = const Duration(seconds: 5),
    Duration pingInterval = const Duration(seconds: 15),
    Map<String, int> protocolVersion = clientProtocolVersion,
    Map<String, int> contractVersion = appContractVersion,
  }) async {
    final connection = RemoteConnection._(channel, pingInterval);
    connection._listen();
    channel.send(
      encodeFrame(
        helloFrame(
          clientId: clientId,
          clientName: clientName,
          token: token,
          protocolVersion: protocolVersion,
          contractVersion: contractVersion,
        ),
      ),
    );
    try {
      final reply = await connection._hello.future.timeout(
        helloTimeout,
        onTimeout: () => throw const ConnectionLost('handshake'),
      );
      if (reply['type'] == 'hello_err') {
        throw HandshakeRefused('${reply['reason'] ?? 'malformed'}');
      }
      connection.hostState = HostState.parse(reply['host_state'] as String?);
      connection.serverVersions = {
        'protocol_version': reply['protocol_version'],
        'contract_version': reply['contract_version'],
      };
    } on Object {
      await connection._shutdown(const ConnectionLost('handshake'));
      rethrow;
    }
    connection._startKeepAlive();
    return connection;
  }

  /// Identifiant d'échange neuf (entiers croissants à partir de 1).
  int nextId() => _nextId++;

  /// Aboutit à la fermeture de la connexion, quelle qu'en soit la cause.
  Future<void> get closed => _closed.future;

  /// Vrai une fois la connexion fermée.
  bool get isClosed => _closed.isCompleted;

  /// Envoie une trame portant un `id` et rend la trame `result` de même `id`.
  Future<Map<String, dynamic>> request(Map<String, dynamic> frame) {
    if (_closed.isCompleted) {
      return Future.error(const ConnectionLost('closed'));
    }
    final id = frame['id'] as int;
    final completer = Completer<Map<String, dynamic>>();
    final bytes = encodeFrame(frame);
    _requests[id] = completer;
    _channel.send(bytes);
    return completer.future;
  }

  /// Ouvre un flux (`open_events`, `open_chat`) : toutes les trames de même `id`, `end` compris ; le flux se ferme après `end`.
  Stream<Map<String, dynamic>> openStream(Map<String, dynamic> openFrame) {
    final controller = StreamController<Map<String, dynamic>>();
    if (_closed.isCompleted) {
      controller.addError(const ConnectionLost('closed'));
      controller.close();
      return controller.stream;
    }
    final bytes = encodeFrame(openFrame);
    _streams[openFrame['id'] as int] = controller;
    _channel.send(bytes);
    return controller.stream;
  }

  /// Envoie une trame (crédit, annulation) ; sans effet si la connexion est fermée.
  void send(Map<String, dynamic> frame) {
    if (_closed.isCompleted) return;
    _channel.send(encodeFrame(frame));
  }

  /// Ferme la connexion (détachement).
  Future<void> close() => _shutdown(const ConnectionLost('closed'));

  void _listen() {
    _channel.incoming.listen(
      _onBytes,
      onError: (Object _) => _shutdown(const ConnectionLost('closed')),
      onDone: () => _shutdown(const ConnectionLost('closed')),
    );
  }

  void _onBytes(List<int> bytes) {
    if (_closed.isCompleted) return;
    try {
      for (final frame in _decoder.add(bytes)) {
        _silentTicks = 0;
        _dispatch(frame);
      }
    } on ProtocolException {
      _shutdown(const ConnectionLost('protocol'));
    }
  }

  void _dispatch(Map<String, dynamic> frame) {
    final id = frame['id'];
    switch (frame['type']) {
      case 'hello_ok' || 'hello_err':
        if (!_hello.isCompleted) _hello.complete(frame);
      case 'result':
        _requests.remove(id)?.complete(frame);
      case 'event' || 'chat_delta':
        _streams[id]?.add(frame);
      case 'end':
        final stream = _streams.remove(id);
        stream?.add(frame);
        stream?.close();
      case 'ping':
        send({'type': 'pong', 'nonce': frame['nonce']});
    }
  }

  void _startKeepAlive() {
    _keepAlive = Timer.periodic(_pingInterval, (_) {
      if (_silentTicks >= 3) {
        _shutdown(const ConnectionLost('silence'));
        return;
      }
      _silentTicks++;
      send(pingFrame(_nonce++));
    });
  }

  Future<void> _shutdown(ConnectionLost why) async {
    if (_closed.isCompleted) return;
    _closed.complete();
    _keepAlive?.cancel();
    if (!_hello.isCompleted) _hello.completeError(why);
    final pendings = List.of(_requests.values);
    _requests.clear();
    for (final pending in pendings) {
      pending.completeError(why);
    }
    final streams = List.of(_streams.values);
    _streams.clear();
    for (final stream in streams) {
      stream.addError(why);
      await stream.close();
    }
    await _channel.close();
  }
}
