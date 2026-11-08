import 'dart:io';

import 'engine_opening.dart';

/// Lit le port TCP que le démon publie dans son fichier ; [DaemonAbsent] si le fichier manque ou ne contient pas un port valide.
Future<int> readDaemonPort(String path) async {
  final String text;
  try {
    text = (await File(path).readAsString()).trim();
  } on FileSystemException {
    throw const DaemonAbsent();
  } on FormatException {
    throw const DaemonAbsent();
  }
  final port = int.tryParse(text);
  if (port == null || port < 1 || port > 65535) throw const DaemonAbsent();
  return port;
}
