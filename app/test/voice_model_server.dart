import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:crypto/crypto.dart';
import 'package:mira/voice/dictation_types.dart';
import 'package:mira/voice/voice_model_catalog.dart';

enum ServeMode { normal, cutHalfway, hangHalfway }

/// Faux serveur de modèles sur 127.0.0.1 : sert une petite archive tar.bz2.
class ModelServer {
  ModelServer({this.folder = 'model-fr', this.extraFile = true});

  final String folder;
  final bool extraFile;
  late final HttpServer _server;
  late Uint8List archive;
  ServeMode mode = ServeMode.normal;
  final Completer<void> release = Completer<void>();
  final Completer<void> halfway = Completer<void>();

  /// Construit l'archive puis écoute sur un port libre.
  Future<void> start({List<String>? only}) async {
    archive = buildArchive(
      only ??
          const ['encoder.onnx', 'decoder.onnx', 'joiner.onnx', 'tokens.txt'],
    );
    _server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    _server.listen(_serve);
  }

  /// Arrête le serveur.
  Future<void> stop() async {
    if (!release.isCompleted) release.complete();
    await _server.close(force: true);
  }

  /// Entrée de catalogue qui pointe sur ce serveur avec la vraie taille et empreinte.
  VoiceModelEntry entry([DictationLanguage language = DictationLanguage.fr]) =>
      VoiceModelEntry(
        language: language,
        url: Uri.parse('http://127.0.0.1:${_server.port}/$folder.tar.bz2'),
        archiveBytes: archive.length,
        sha256: sha256.convert(archive).toString(),
        folder: folder,
        files: const [
          'encoder.onnx',
          'decoder.onnx',
          'joiner.onnx',
          'tokens.txt',
        ],
      );

  /// Archive tar.bz2 contenant les fichiers donnés sous un dossier racine, comme les vraies.
  Uint8List buildArchive(List<String> names) {
    final random = Random(7);
    final a = Archive();
    for (final n in names) {
      final data = n == 'tokens.txt'
          ? utf8.encode('<blk> 0\n')
          : List<int>.generate(60000, (_) => random.nextInt(256));
      a.addFile(ArchiveFile.bytes('$folder/$n', data));
    }
    if (extraFile) {
      a.addFile(ArchiveFile.bytes('$folder/README.md', utf8.encode('x')));
    }
    return Uint8List.fromList(
      BZip2Encoder().encodeBytes(TarEncoder().encodeBytes(a)),
    );
  }

  Future<void> _serve(HttpRequest request) async {
    if (mode == ServeMode.normal) return _serveAll(request.response);
    return _serveHalf(request.response);
  }

  /// Réponse normale : toute l'archive, par paquets.
  Future<void> _serveAll(HttpResponse response) async {
    response.contentLength = archive.length;
    for (var i = 0; i < archive.length; i += 8192) {
      response.add(archive.sublist(i, min(i + 8192, archive.length)));
      await response.flush();
      await Future<void>.delayed(const Duration(milliseconds: 5));
    }
    await response.close();
  }

  /// Annonce toute l'archive, n'en envoie que la moitié, puis bloque ou coupe la connexion.
  Future<void> _serveHalf(HttpResponse response) async {
    final socket = await response.detachSocket(writeHeaders: false);
    socket.write(
      'HTTP/1.1 200 OK\r\ncontent-length: ${archive.length}\r\n\r\n',
    );
    socket.add(archive.sublist(0, archive.length ~/ 2));
    await socket.flush();
    if (!halfway.isCompleted) halfway.complete();
    if (mode == ServeMode.hangHalfway) await release.future;
    socket.destroy();
  }
}
