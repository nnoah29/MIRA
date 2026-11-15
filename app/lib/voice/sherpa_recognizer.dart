//! Reconnaissance vocale locale en flux : pilote l'isolate sherpa-onnx et en relaie les textes.

import 'dart:async';
import 'dart:isolate';
import 'dart:typed_data';

import 'dictation_types.dart';
import 'sherpa_isolate.dart';
import 'voice_model_store.dart';

/// Reconnaisseur qui décode dans un isolate pour ne jamais figer l'interface.
class SherpaRecognizer implements SpeechRecognizer {
  SherpaRecognizer(this.store);

  final VoiceModelStore store;
  final StreamController<DictationEvent> _events =
      StreamController<DictationEvent>.broadcast(sync: true);
  Isolate? _isolate;
  ReceivePort? _inbox;
  SendPort? _toIsolate;
  Completer<void>? _ready;
  Completer<void>? _done;

  @override
  Stream<DictationEvent> get events => _events.stream;

  @override
  Future<void> start(DictationLanguage language) async {
    final dir = store.directoryOf(language);
    if (dir == null) {
      throw const DictationException(DictationProblem.modelMissing);
    }
    await _release();
    final inbox = _inbox = ReceivePort();
    final handshake = Completer<SendPort>();
    inbox.listen((m) => _onMessage(m, handshake));
    _ready = Completer<void>();
    _isolate = await Isolate.spawn(sherpaIsolateMain, inbox.sendPort);
    _toIsolate = await handshake.future;
    _toIsolate!.send(['init', dir.path]);
    await _ready!.future;
  }

  /// Aiguille un message de l'isolate : poignée de main, prêt, texte, erreur ou fin.
  void _onMessage(Object? m, Completer<SendPort> handshake) {
    if (m is SendPort) return handshake.complete(m);
    if (m is! List) return;
    switch (m.first) {
      case 'ready':
        _ready?.complete();
      case 'partial':
        _events.add(DictationPartial(m[1] as String));
      case 'final':
        _events.add(DictationFinal(m[1] as String));
      case 'done':
        _done?.complete();
      case 'error':
        _onError();
    }
  }

  /// Une erreur native : l'attente en cours échoue, sinon l'erreur part dans le flux d'événements.
  void _onError() {
    const failure = DictationException(DictationProblem.recognizerFailed);
    final ready = _ready;
    if (ready != null && !ready.isCompleted) {
      ready.completeError(failure);
      return;
    }
    final done = _done;
    if (done != null && !done.isCompleted) done.complete();
    _events.addError(failure);
  }

  @override
  void feed(Float32List samples) {
    _toIsolate?.send(TransferableTypedData.fromList([samples]));
  }

  @override
  Future<void> stop() async {
    final port = _toIsolate;
    if (port == null) return;
    _done = Completer<void>();
    port.send('stop');
    await _done!.future.timeout(const Duration(seconds: 10), onTimeout: () {});
    await _release();
  }

  /// Termine l'isolate et ferme les ports.
  Future<void> _release() async {
    _isolate?.kill(priority: Isolate.immediate);
    _isolate = null;
    _inbox?.close();
    _inbox = null;
    _toIsolate = null;
  }
}
