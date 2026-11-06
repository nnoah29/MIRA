import 'dart:async';

import '../engine_client.dart';
import 'connection.dart';
import 'frame_builders.dart';

/// Résultat d'une lecture du flux : quand rouvrir.
enum _Next { now, afterBackoff, never }

/// Flux d'événements du démon qui survit aux coupures : crédits réapprovisionnés, reprise par numéro.
class EventPump {
  EventPump(
    this._live, {
    int? after,
    Duration backoff = const Duration(milliseconds: 500),
  }) : _lastSeq = after,
       _initialBackoff = backoff,
       _backoff = backoff {
    _controller = StreamController<Map<String, dynamic>>(
      onListen: () => unawaited(_run()),
      onCancel: _stop,
    );
  }

  /// Fenêtre de crédits donnée à l'ouverture, et nombre de crédits rendus à chaque palier.
  static const int window = 256;
  static const int refill = 128;
  static const Duration maxBackoff = Duration(seconds: 5);

  final Future<RemoteConnection> Function() _live;
  final Duration _initialBackoff;
  Duration _backoff;
  late final StreamController<Map<String, dynamic>> _controller;
  int? _lastSeq;
  bool _stopped = false;
  RemoteConnection? _connection;
  int? _streamId;
  int _sinceRefill = 0;

  /// Les événements (objets `Event`), à abonné unique.
  Stream<Map<String, dynamic>> get stream => _controller.stream;

  Future<void> _run() async {
    while (!_stopped) {
      var next = _Next.afterBackoff;
      try {
        next = await _consume(await _live());
      } on Object {
        next = _Next.afterBackoff;
      }
      if (next == _Next.never || _stopped) return;
      if (next == _Next.afterBackoff) await _wait();
    }
  }

  /// Attente suivante : le double de l'actuelle, au plus [maxBackoff].
  static Duration nextBackoff(Duration current) {
    final doubled = current * 2;
    return doubled > maxBackoff ? maxBackoff : doubled;
  }

  Future<void> _wait() async {
    await Future<void>.delayed(_backoff);
    _backoff = nextBackoff(_backoff);
  }

  /// Ouvre un flux sur cette connexion et le lit jusqu'à sa fin.
  Future<_Next> _consume(RemoteConnection connection) async {
    final id = connection.nextId();
    _connection = connection;
    _streamId = id;
    _sinceRefill = 0;
    final ended = Completer<_Next>();
    void finish(_Next next) {
      if (!ended.isCompleted) ended.complete(next);
    }

    final subscription = connection
        .openStream(openEventsFrame(id, after: _lastSeq, credit: window))
        .listen(
          (frame) => _onFrame(connection, id, frame, finish),
          onError: (Object _) => finish(_Next.afterBackoff),
          onDone: () => finish(_Next.afterBackoff),
        );
    final next = await ended.future;
    await subscription.cancel();
    return next;
  }

  void _onFrame(
    RemoteConnection connection,
    int id,
    Map<String, dynamic> frame,
    void Function(_Next) finish,
  ) {
    if (frame['type'] == 'event') {
      _deliver(connection, id, frame['event'] as Map<String, dynamic>);
    } else if (frame['type'] == 'end') {
      finish(_onEnd(frame['reason'] as Map<String, dynamic>));
    }
  }

  void _deliver(
    RemoteConnection connection,
    int id,
    Map<String, dynamic> event,
  ) {
    final seq = event['seq'];
    if (seq is int) _lastSeq = seq;
    _backoff = _initialBackoff;
    if (!_controller.isClosed) _controller.add(event);
    if (++_sinceRefill >= refill) {
      _sinceRefill = 0;
      connection.send(creditFrame(id, refill));
    }
  }

  _Next _onEnd(Map<String, dynamic> reason) {
    switch (reason['type']) {
      case 'lagged':
        return _Next.now;
      case 'error':
        if (!_controller.isClosed) {
          _controller.addError(
            EngineException(
              '${reason['code'] ?? 'internal.bug'}',
              '${reason['message'] ?? 'erreur du moteur'}', // i18n:ignore
            ),
          );
        }
        _stopped = true;
        unawaited(_controller.close());
        return _Next.never;
      default:
        return _Next.afterBackoff;
    }
  }

  Future<void> _stop() async {
    _stopped = true;
    final id = _streamId;
    if (id != null) _connection?.send(cancelFrame(id));
  }
}
