//! Vérification de l'empreinte et extraction d'une archive de modèle, hors du fil de l'interface.

import 'dart:io';
import 'dart:isolate';

import 'package:archive/archive.dart';
import 'package:crypto/crypto.dart';

/// Empreinte SHA-256 (hexadécimale) d'un fichier, lue en flux.
Future<String> fileSha256(File file) async =>
    (await sha256.bind(file.openRead()).first).toString();

/// Extrait de l'archive les seules [files] voulues dans [outDir], dans un isolate ; échoue s'il en manque.
Future<void> extractModelFiles(
  String archivePath,
  String outDir,
  List<String> files,
) => Isolate.run(() => _extract(archivePath, outDir, files));

/// Décompresse puis écrit les fichiers retenus, à plat, sous leur seul nom.
void _extract(String archivePath, String outDir, List<String> files) {
  final raw = File(archivePath).readAsBytesSync();
  final archive = TarDecoder().decodeBytes(BZip2Decoder().decodeBytes(raw));
  Directory(outDir).createSync(recursive: true);
  final found = <String>{};
  for (final entry in archive.files) {
    final name = entry.name.split('/').last;
    if (!entry.isFile || !files.contains(name)) continue;
    File('$outDir/$name').writeAsBytesSync(entry.content);
    found.add(name);
  }
  final missing = files.where((f) => !found.contains(f)).toList();
  if (missing.isNotEmpty) throw FormatException('fichiers absents : $missing');
}
