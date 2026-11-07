import 'dart:async';

import 'package:mira/engine/remote/byte_channel.dart';

/// Une extrémité d'un canal en mémoire : ce qu'elle envoie arrive à l'autre, de façon asynchrone.
class MemoryChannel implements ByteChannel {
  MemoryChannel._(this._in, this._out);

  final StreamController<List<int>> _in;
  final StreamController<List<int>> _out;
  MemoryChannel? _peer;
  final Completer<void> _done = Completer<void>();

  /// Deux canaux reliés : fermer l'un ferme l'autre.
  static (MemoryChannel, MemoryChannel) pair() {
    final a = StreamController<List<int>>();
    final b = StreamController<List<int>>();
    final left = MemoryChannel._(a, b);
    final right = MemoryChannel._(b, a);
    left._peer = right;
    right._peer = left;
    return (left, right);
  }

  @override
  Stream<List<int>> get incoming => _in.stream;

  @override
  void send(List<int> bytes) {
    if (_done.isCompleted) return;
    final copy = List<int>.of(bytes);
    scheduleMicrotask(() {
      if (!_out.isClosed) _out.add(copy);
    });
  }

  @override
  Future<void> get done => _done.future;

  @override
  Future<void> close() async {
    if (_done.isCompleted) return;
    _done.complete();
    scheduleMicrotask(() async {
      if (!_out.isClosed) await _out.close();
      await _peer?.close();
    });
  }
}
