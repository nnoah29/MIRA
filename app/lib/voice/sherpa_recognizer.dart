//! Reconnaissance vocale locale en flux : pilote l'isolate sherpa-onnx, en garde le modèle chargé un moment, relaie les textes.

import 'dart:async';
import 'dart:isolate';
import 'dart:typed_data';

import 'dictation_types.dart';
import 'model_keeper.dart';
import 'sherpa_isolate.dart';
import 'voice_model_store.dart';

/// Reconnaisseur qui décode dans un isolate pour ne jamais figer l'interface.
class SherpaRecognizer implements SpeechRecognizer, KeepWarmSetting {
  SherpaRecognizer(
    this.store, {
    Duration keepWarm = const Duration(minutes: 3),
  }) {
    _keeper = ModelKeeper(
      keepFor: keepWarm,
      release: () => unawaited(_release()),
    );
  }

  final VoiceModelStore store;
  final StreamController<DictationEvent> _events =
      StreamController<DictationEvent>.broadcast(sync: true);
  late final ModelKeeper _keeper;
  Isolate? _isolate;
  ReceivePort? _inbox;
  SendPort? _toIsolate;
  String? _warmDir;
  Completer<void>? _ready;
  Completer<void>? _done;
  Completer<void>? _released;
  Future<void>? _releasing;

  @override
  Stream<DictationEvent> get events => _events.stream;

  @override
  set keepWarm(Duration keep) => _keeper.keepFor = keep;

  /// Dit si le modèle est chargé, qu'une dictée soit en cours ou non.
  bool get isWarm => _warmDir != null;

  @override
  Future<void> start(DictationLanguage language) async {
    final dir = store.directoryOf(language);
    if (dir == null) {
      throw const DictationException(DictationProblem.modelMissing);
    }
    _keeper.used();
    await _releasing;
    if (_warmDir == dir.path && _toIsolate != null) return _begin();
    await _release();
    await _load(dir.path);
  }

  /// Nouvelle dictée sur le modèle déjà chargé.
  Future<void> _begin() async {
    _ready = Completer<void>();
    _toIsolate!.send('begin');
    await _ready!.future;
  }

  /// Lance l'isolate et charge le modèle du dossier [path].
  Future<void> _load(String path) async {
    final inbox = _inbox = ReceivePort();
    final handshake = Completer<SendPort>();
    inbox.listen((m) => _onMessage(m, handshake));
    _ready = Completer<void>();
    _isolate = await Isolate.spawn(sherpaIsolateMain, inbox.sendPort);
    _toIsolate = await handshake.future;
    _toIsolate!.send(['init', path]);
    await _ready!.future;
    _warmDir = path;
  }

  /// Aiguille un message de l'isolate : poignée de main, prêt, texte, erreur, fin ou libération.
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
      case 'released':
        _released?.complete();
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
    _keeper.idle();
  }

  /// Libère le modèle puis termine l'isolate ; une libération déjà en cours est attendue.
  Future<void> _release() => _releasing ??= _doRelease().whenComplete(() {
    _releasing = null;
  });

  /// Demande à l'isolate de libérer ses objets natifs, puis le termine et ferme les ports.
  Future<void> _doRelease() async {
    final port = _toIsolate;
    if (port != null) {
      _released = Completer<void>();
      port.send('release');
      await _released!.future.timeout(
        const Duration(seconds: 5),
        onTimeout: () {},
      );
    }
    _isolate?.kill(priority: Isolate.immediate);
    _isolate = null;
    _inbox?.close();
    _inbox = null;
    _toIsolate = null;
    _warmDir = null;
  }
}
