import 'dart:io';

import '../../i18n/i18n.dart';
import '../engine_client.dart';

final RegExp _hex64 = RegExp(r'^[0-9a-fA-F]{64}$');

/// Lit le jeton local du démon : le fichier doit exister, n'être lisible que par son propriétaire (hors Windows, où le dossier de l'utilisateur est déjà privé) et contenir 64 chiffres hexadécimaux.
/// Les erreurs ne reprennent jamais le contenu du fichier.
Future<String> readDaemonToken(
  String path, {
  bool enforcePrivateMode = true,
}) async {
  final stat = await FileStat.stat(path);
  if (stat.type == FileSystemEntityType.notFound) {
    throw EngineException.local(
      'transport.token_missing',
      t('Jeton du moteur en arrière-plan introuvable'),
    );
  }
  if (enforcePrivateMode && !Platform.isWindows && stat.mode & 0x3F != 0) {
    throw EngineException.local(
      'transport.token_unsafe',
      t('Le jeton du moteur est lisible par d’autres utilisateurs'),
    );
  }
  final String text;
  try {
    text = (await File(path).readAsString()).trim();
  } on FileSystemException {
    throw _unreadable();
  } on FormatException {
    throw _unreadable();
  }
  if (!_hex64.hasMatch(text)) {
    throw _unreadable();
  }
  return text;
}

/// Jeton impossible à lire ou mal formé : jamais le chemin ni le contenu dans le message.
EngineException _unreadable() => EngineException.local(
  'transport.token_invalid',
  t('Jeton du moteur en arrière-plan illisible'),
);
