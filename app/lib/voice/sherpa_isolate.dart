//! Côté isolate de la reconnaissance : modèle sherpa-onnx, décodage et événements renvoyés à l'interface.

import 'dart:isolate';
import 'dart:typed_data';

import 'package:sherpa_onnx/sherpa_onnx.dart' as so;

/// Un reconnaisseur en flux et le dernier texte provisoire déjà annoncé.
class SherpaSession {
  SherpaSession._(this._recognizer, this._stream, this._send);

  final so.OnlineRecognizer _recognizer;
  final so.OnlineStream _stream;
  final void Function(List<Object> message) _send;
  String _last = '';

  /// Charge le modèle du dossier [modelDir] (encoder, decoder, joiner, tokens).
  factory SherpaSession.open(
    String modelDir,
    void Function(List<Object> message) send,
  ) {
    so.initBindings();
    final model = so.OnlineModelConfig(
      transducer: so.OnlineTransducerModelConfig(
        encoder: '$modelDir/encoder.onnx',
        decoder: '$modelDir/decoder.onnx',
        joiner: '$modelDir/joiner.onnx',
      ),
      tokens: '$modelDir/tokens.txt',
      numThreads: 2,
    );
    final recognizer = so.OnlineRecognizer(
      so.OnlineRecognizerConfig(model: model),
    );
    return SherpaSession._(recognizer, recognizer.createStream(), send);
  }

  /// Décode un paquet d'échantillons ; annonce le texte s'il a changé, le valide en fin de phrase.
  void accept(Float32List samples) {
    _stream.acceptWaveform(samples: samples, sampleRate: 16000);
    _decodeAvailable();
    final text = _recognizer.getResult(_stream).text;
    if (text != _last) {
      _last = text;
      _send(['partial', text]);
    }
    if (_recognizer.isEndpoint(_stream)) _endSentence();
  }

  /// Valide la phrase en cours et repart d'un flux propre.
  void _endSentence() {
    if (_last.trim().isNotEmpty) _send(['final', _last]);
    _recognizer.reset(_stream);
    _last = '';
  }

  /// Décode tout ce que le modèle peut décoder avec l'audio reçu.
  void _decodeAvailable() {
    while (_recognizer.isReady(_stream)) {
      _recognizer.decode(_stream);
    }
  }

  /// Fin de la parole : ajoute un court silence pour vider le modèle, valide le reste, libère tout.
  void finish() {
    _stream.acceptWaveform(samples: Float32List(6400), sampleRate: 16000);
    _stream.inputFinished();
    _decodeAvailable();
    _last = _recognizer.getResult(_stream).text;
    if (_last.trim().isNotEmpty) _send(['final', _last]);
    _stream.free();
    _recognizer.free();
  }
}

/// Point d'entrée de l'isolate : reçoit `['init', dossier]`, des paquets audio, puis `'stop'`.
void sherpaIsolateMain(SendPort toMain) {
  final inbox = ReceivePort();
  toMain.send(inbox.sendPort);
  SherpaSession? session;
  inbox.listen((message) {
    try {
      session = _handle(session, message, toMain);
    } on Object catch (e) {
      toMain.send(['error', '$e']);
    }
  });
}

/// Traite un message de l'interface et rend la session à garder.
SherpaSession? _handle(SherpaSession? session, Object? message, SendPort out) {
  if (message is List && message.first == 'init') {
    final opened = SherpaSession.open(message[1] as String, out.send);
    out.send(['ready']);
    return opened;
  }
  if (message is TransferableTypedData) {
    session?.accept(message.materialize().asFloat32List());
    return session;
  }
  if (message == 'stop') {
    session?.finish();
    out.send(['done']);
    return null;
  }
  return session;
}
