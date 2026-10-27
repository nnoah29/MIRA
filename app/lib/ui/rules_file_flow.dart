import 'dart:convert';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';

import '../engine/engine_client.dart';
import '../engine/rule_admin.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';

/// Enregistre un fichier là où l'utilisateur le choisit.
typedef SaveFile = Future<Uri?> Function(String name, Uint8List bytes);

/// Fait choisir un fichier YAML et rend son texte ; `null` si l'on annule.
typedef PickYaml = Future<String?> Function();

/// Boîte d'enregistrement du système.
Future<Uri?> saveWithPicker(String name, Uint8List bytes) =>
    FilePicker.saveFile(fileName: name, bytes: bytes);

/// Taille au-delà de laquelle le fichier n'est pas lu : le moteur le refuserait de toute façon.
const maxRulesFileBytes = 2 * 1024 * 1024;

/// Boîte d'ouverture du système, limitée aux fichiers `.yaml` et `.yml`.
Future<String?> pickYamlWithPicker() async {
  final found = await FilePicker.pickFiles(
    type: FileType.custom,
    allowedExtensions: const ['yaml', 'yml'],
  );
  if (found.isEmpty) return null;
  final file = found.first;
  if ((await file.length() ?? 0) > maxRulesFileBytes) {
    throw const EngineException(
      'invalid_input.export_too_large',
      'export file too large',
    );
  }
  return utf8.decode(await file.readAsBytes(), allowMalformed: true);
}

/// Nom du fichier d'export du jour, par exemple `mira-regles-2026-10-27.yaml`.
String rulesExportName(DateTime now) {
  String two(int n) => n.toString().padLeft(2, '0');
  return 'mira-regles-${now.year}-${two(now.month)}-${two(now.day)}.yaml';
}

/// Exporte les règles dans le fichier choisi par l'utilisateur.
Future<void> exportRulesFile(
  RuleAdmin admin,
  SaveFile save,
  DateTime now,
) async {
  final yaml = await admin.exportRules();
  await save(rulesExportName(now), Uint8List.fromList(utf8.encode(yaml)));
}

/// Erreur d'un fichier d'export dite à l'utilisateur, avec la ligne et la colonne d'une erreur de syntaxe.
String fileErrorText(Object error) {
  final text = engineErrorText(error);
  if (error is! EngineException || error.code != 'invalid_input.yaml_syntax') {
    return text;
  }
  final at = RegExp(r'ligne (\d+), colonne (\d+)').firstMatch(error.message);
  if (at == null) return text;
  return '$text (${t('ligne {0}, colonne {1}', [at.group(1), at.group(2)])})';
}
