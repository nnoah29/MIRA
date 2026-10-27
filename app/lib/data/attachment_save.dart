import 'dart:io';
import 'dart:typed_data';

import '../i18n/i18n.dart';
import 'models.dart';

/// Noms des pièces enregistrées et des pièces qui n'ont pas pu l'être.
class SaveReport {
  final List<String> saved = [];
  final List<String> failed = [];
}

/// Nom de fichier sans chemin : seul le dernier élément est gardé, `.` et `..` deviennent un nom neutre.
String safeFileName(String name) {
  final last = name.split(RegExp(r'[/\\]')).last.trim();
  return (last.isEmpty || last == '.' || last == '..')
      ? t('pièce jointe')
      : last;
}

/// Premier nom libre du dossier : `a.pdf`, puis `a (2).pdf`, `a (3).pdf`…
String freeName(Directory dir, String name) {
  if (!File('${dir.path}/$name').existsSync()) return name;
  final dot = name.lastIndexOf('.');
  final stem = dot > 0 ? name.substring(0, dot) : name;
  final ext = dot > 0 ? name.substring(dot) : '';
  var n = 2;
  while (File('${dir.path}/$stem ($n)$ext').existsSync()) {
    n++;
  }
  return '$stem ($n)$ext';
}

/// Enregistre chaque pièce dans [dir] sous un nom libre ; une pièce illisible n'arrête pas les autres.
Future<SaveReport> saveAttachments(
  List<Attachment> list,
  Directory dir,
  Future<Uint8List?> Function(Attachment) bytesOf,
) async {
  final report = SaveReport();
  for (final a in list) {
    final name = freeName(dir, safeFileName(a.n));
    try {
      final bytes = await bytesOf(a);
      if (bytes == null) throw const FileSystemException();
      await File('${dir.path}/$name').writeAsBytes(bytes);
      report.saved.add(name);
    } on Object {
      report.failed.add(a.n);
    }
  }
  return report;
}
