//! Côté isolate de la reconnaissance : modèle sherpa-onnx, décodage et événements renvoyés à l'interface.

import 'dart:isolate';
import 'dart:typed_data';

import 'package:sherpa_onnx/sherpa_onnx.dart' as so;

import 'sherpa_config.dart';

/// Un reconnaisseur en flux et le dernier texte provisoire déjà annoncé.
class SherpaSession {
  SherpaSession._(this._recognizer, this._stream, this._send);

  final so.OnlineRecognizer _recognizer;
  so.OnlineStream _stream;
  final void Function(List<Object> message) _send;
  String _last = '';

  /// Charge le modèle du dossier [modelDir] (encoder, decoder, joiner, tokens).
  factory SherpaSession.open(
    String modelDir,
    void Function(List<Object> message) send,
  ) {
    so.initBindings();
    final recognizer = so.OnlineRecognizer(recognizerConfigFor(modelDir));
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

  /// Fin de la parole : vide le modèle avec un court silence, valide le reste, puis repart d'un flux neuf (le modèle reste chargé).
  void finishStream() {
    _stream.acceptWaveform(samples: Float32List(6400), sampleRate: 16000);
    _stream.inputFinished();
    _decodeAvailable();
    _last = _recognizer.getResult(_stream).text;
    if (_last.trim().isNotEmpty) _send(['final', _last]);
    _stream.free();
    _stream = _recognizer.createStream();
    _last = '';
  }

  /// Nouvelle dictée sur le modèle déjà chargé.
  void restart() {
    _recognizer.reset(_stream);
    _last = '';
  }

  /// Libère le flux et le modèle.
  void dispose() {
    _stream.free();
    _recognizer.free();
  }
}

/// Point d'entrée de l'isolate : `['init', dossier]`, des paquets audio, `'stop'` ; `'begin'` et `'release'` pour garder le modèle.
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
    session?.dispose();
    final opened = SherpaSession.open(message[1] as String, out.send);
    out.send(['ready']);
    return opened;
  }
  if (message is TransferableTypedData) {
    session?.accept(message.materialize().asFloat32List());
    return session;
  }
  return _control(session, message, out);
}

/// Commandes sans audio : nouvelle dictée sur le modèle chargé, fin de dictée, libération du modèle.
SherpaSession? _control(SherpaSession? session, Object? message, SendPort out) {
  switch (message) {
    case 'begin':
      session?.restart();
      out.send(['ready']);
      return session;
    case 'stop':
      session?.finishStream();
      out.send(['done']);
      return session;
    case 'release':
      session?.dispose();
      out.send(['released']);
      return null;
    default:
      return session;
  }
}
