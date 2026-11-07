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
    this._baseline,
  }) : _lastSeq = after,
       _initialBackoff = backoff,
       _backoff = backoff {
    _controller = StreamController<Map<String, dynamic>>(
      onListen: () => unawaited(_run()),
      onCancel: _stop,
      onResume: _giveBackOwed,
    );
  }

  /// Fenêtre de crédits donnée à l'ouverture, et nombre de crédits rendus à chaque palier.
  static const int window = 256;
  static const int refill = 128;
  static const Duration maxBackoff = Duration(seconds: 5);

  final Future<RemoteConnection> Function() _live;
  final Future<int> Function()? _baseline;
  final Duration _initialBackoff;
  int _owed = 0;
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
    await _learnBaseline();
    while (!_stopped) {
      var next = _Next.afterBackoff;
      try {
        next = await _consume(await _live());
      } on EngineException catch (error) {
        next = error.code == 'transport.closed'
            ? _Next.never
            : _Next.afterBackoff;
      } on Object {
        next = _Next.afterBackoff;
      }
      if (next == _Next.never || _stopped) {
        await _controller.close();
        return;
      }
      if (next == _Next.afterBackoff) await _wait();
    }
  }

  /// Sans point de départ, note la fin actuelle du journal : une coupure avant le premier événement ne perd alors rien.
  Future<void> _learnBaseline() async {
    final baseline = _baseline;
    if (_lastSeq != null || baseline == null) return;
    try {
      _lastSeq = await baseline();
    } on Object {
      _lastSeq = null;
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
    if (_stopped) return _Next.never;
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
      _deliver(connection, id, frame['event']);
    } else if (frame['type'] == 'end') {
      finish(_onEnd(frame['reason'] as Map<String, dynamic>));
    }
  }

  void _deliver(RemoteConnection connection, int id, Object? event) {
    if (event is Map<String, dynamic>) {
      final seq = event['seq'];
      if (seq is int) _lastSeq = seq;
      _backoff = _initialBackoff;
      if (!_controller.isClosed) _controller.add(event);
    }
    if (++_sinceRefill < refill) return;
    _sinceRefill = 0;
    if (_controller.isPaused) {
      _owed += refill;
    } else {
      connection.send(creditFrame(id, refill));
    }
  }

  /// Rend les crédits retenus pendant la pause de l'abonné.
  void _giveBackOwed() {
    final id = _streamId;
    if (_owed == 0 || id == null) return;
    final owed = _owed;
    _owed = 0;
    _connection?.send(creditFrame(id, owed));
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
