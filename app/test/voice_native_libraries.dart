//! Chargement des bibliothèques natives de sherpa pour les tests d'intégration de la dictée.
import 'dart:ffi';
import 'dart:io';

import 'package:sherpa_onnx/sherpa_onnx.dart' as so;

/// Dossier du `PATH` qui contient la bibliothèque de sherpa, `null` s'il n'y en a pas.
Directory? sherpaDirectoryInPath() {
  final path = Platform.environment['PATH'] ?? '';
  for (final entry in path.split(';')) {
    final library = File('$entry\\sherpa-onnx-c-api.dll');
    if (entry.isNotEmpty && library.existsSync()) return Directory(entry);
  }
  return null;
}

/// Initialise sherpa en chargeant d'abord son `onnxruntime.dll`, que Windows prendrait sinon dans `System32`.
void initSherpaBindings() {
  final directory = Platform.isWindows ? sherpaDirectoryInPath() : null;
  if (directory != null) {
    DynamicLibrary.open('${directory.path}\\onnxruntime.dll');
  }
  so.initBindings();
}
