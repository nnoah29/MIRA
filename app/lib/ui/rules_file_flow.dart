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

/// Refuse un fichier de plus de 2 Mo avec le code du moteur, que sa taille ait été connue avant lecture ou non.
void ensureRulesFileSize(int bytes) {
  if (bytes > maxRulesFileBytes) {
    throw const EngineException(
      'invalid_input.export_too_large',
      'export file too large',
    );
  }
}

/// Boîte d'ouverture du système, limitée aux fichiers `.yaml` et `.yml`.
Future<String?> pickYamlWithPicker() async {
  final found = await FilePicker.pickFiles(
    type: FileType.custom,
    allowedExtensions: const ['yaml', 'yml'],
  );
  if (found.isEmpty) return null;
  final file = found.first;
  ensureRulesFileSize(await file.length() ?? 0);
  final bytes = await file.readAsBytes();
  ensureRulesFileSize(bytes.length);
  return utf8.decode(bytes, allowMalformed: true);
}

/// Nom d'un fichier d'export du jour, par exemple `mira-regles-2026-10-27.yaml`.
String _dated(String prefix, DateTime now) {
  String two(int n) => n.toString().padLeft(2, '0');
  return 'mira-$prefix-${now.year}-${two(now.month)}-${two(now.day)}.yaml';
}

/// Nom du fichier d'export des règles du jour.
String rulesExportName(DateTime now) => _dated('regles', now);

/// Nom du fichier d'export des libellés du jour.
String labelsExportName(DateTime now) => _dated('libelles', now);

/// Enregistre un texte YAML sous ce nom, là où l'utilisateur le choisit.
Future<void> saveYamlText(SaveFile save, String name, String yaml) =>
    save(name, Uint8List.fromList(utf8.encode(yaml)));

/// Exporte les règles dans le fichier choisi ; rend les noms des règles laissées de côté.
Future<List<String>> exportRulesFile(
  RuleAdmin admin,
  SaveFile save,
  DateTime now,
) async {
  final export = await admin.exportRules();
  await saveYamlText(save, rulesExportName(now), export.yaml);
  return export.skipped;
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

/// Avertissement après un export qui a écarté des règles dont une cible (libellé, modèle, boîte, compte) a disparu.
String skippedNotice(List<String> names) => t(
  '{0} non exportée(s), une cible a disparu : {1}',
  [names.length, names.join(', ')],
);
