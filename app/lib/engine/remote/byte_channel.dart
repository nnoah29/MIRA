import 'dart:async';
import 'dart:io';

/// Flux d'octets bidirectionnel fiable : tout ce que le protocole demande à un transport.
abstract interface class ByteChannel {
  /// Octets reçus ; le flux se termine quand l'autre côté ferme.
  Stream<List<int>> get incoming;

  /// Envoie des octets ; sans effet une fois le canal fermé.
  void send(List<int> bytes);

  /// Ferme le canal (sans effet s'il est déjà fermé).
  Future<void> close();

  /// Aboutit à la fermeture du canal, quel qu'en soit l'auteur.
  Future<void> get done;
}

/// Canal sur une socket `dart:io` : Unix ou TCP loopback.
class SocketChannel implements ByteChannel {
  SocketChannel._(this._socket) {
    _socket.listen(
      _in.add,
      onError: (Object _) => _finish(),
      onDone: _finish,
      cancelOnError: true,
    );
    // Un échec d'écriture (pair fermé) complète `done` avec une erreur que personne d'autre n'écoute.
    unawaited(_socket.done.then((_) {}, onError: (Object _) => _finish()));
  }

  final Socket _socket;
  final StreamController<List<int>> _in = StreamController<List<int>>();
  final Completer<void> _done = Completer<void>();

  /// Se connecte à une socket Unix.
  static Future<SocketChannel> connectUnix(
    String path, {
    Duration timeout = const Duration(seconds: 5),
  }) async => SocketChannel._(
    await Socket.connect(
      InternetAddress(path, type: InternetAddressType.unix),
      0,
      timeout: timeout,
    ),
  );

  /// Se connecte en TCP.
  static Future<SocketChannel> connectTcp(
    String host,
    int port, {
    Duration timeout = const Duration(seconds: 5),
  }) async =>
      SocketChannel._(await Socket.connect(host, port, timeout: timeout));

  @override
  Stream<List<int>> get incoming => _in.stream;

  @override
  void send(List<int> bytes) {
    if (_done.isCompleted) return;
    try {
      _socket.add(bytes);
    } on Object {
      _finish();
    }
  }

  @override
  Future<void> get done => _done.future;

  @override
  Future<void> close() async {
    _socket.destroy();
    await _finish();
  }

  /// Termine le flux entrant et marque la fermeture, une seule fois.
  Future<void> _finish() async {
    if (_done.isCompleted) return;
    _done.complete();
    _socket.destroy();
    if (!_in.isClosed) await _in.close();
  }
}
