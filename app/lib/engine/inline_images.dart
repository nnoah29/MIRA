import 'dart:io';
import 'dart:typed_data';

import 'engine_client.dart';

const _maxInline = 20;
const _maxBytes = 5 * 1024 * 1024;

/// Total d'images incorporées gardées en mémoire par message ; les autres restent des pièces jointes.
const inlineBudget = 8 * 1024 * 1024;

/// Garde, dans l'ordre, les images qui tiennent encore dans le budget du message.
List<Map<String, dynamic>> withinInlineBudget(
  List<Map<String, dynamic>> images,
) {
  var left = inlineBudget;
  final kept = <Map<String, dynamic>>[];
  for (final a in images) {
    final size = ((a['size'] as num?) ?? 0).toInt();
    if (size > left) continue;
    left -= size;
    kept.add(a);
  }
  return kept;
}

/// Identifiants `cid:` cités par un HTML, sans chevrons et en minuscules.
Set<String> citedCids(String html) => {
  for (final m in RegExp(
    r'cid:([^"\s>)]+)',
    caseSensitive: false,
  ).allMatches(html))
    m.group(1)!.toLowerCase(),
};

/// Identifiant de contenu d'une pièce jointe, sans chevrons et en minuscules.
String? cidOf(Map<String, dynamic> attachment) {
  final raw = attachment['content_id'] as String?;
  return raw?.replaceAll(RegExp(r'^<|>$'), '').toLowerCase();
}

/// Lit les images incorporées citées par le HTML, en les enregistrant dans un dossier temporaire.
Future<Map<String, Uint8List>> loadInlineImages(
  EngineClient client,
  Set<String> wanted,
  List<Map<String, dynamic>> attachments,
) async {
  final wantedNow = withinInlineBudget([
    for (final a in attachments)
      if (cidOf(a) case final cid? when wanted.contains(cid))
        if (((a['size'] as num?) ?? 0) <= _maxBytes) a,
  ]).take(_maxInline).toList();
  final bytes = await Future.wait([
    for (final a in wantedNow) _fetch(client, '${a['id']}'),
  ]);
  return {
    for (final (i, a) in wantedNow.indexed)
      if (bytes[i] != null) cidOf(a)!: bytes[i]!,
  };
}

Future<Uint8List?> _fetch(EngineClient client, String id) =>
    fetchAttachment(client, id);

/// Lit une pièce jointe : le moteur l'écrit dans un dossier temporaire, lu une fois complet.
Future<Uint8List?> fetchAttachment(EngineClient client, String id) async {
  final dir = await Directory.systemTemp.createTemp('mira-att');
  final file = File('${dir.path}/file');
  try {
    await client.call(
      req('message', 'save_attachment', {
        'attachment': id,
        'destination': file.path,
      }),
    );
    return await _whenWritten(file);
  } on EngineException {
    return null;
  } finally {
    await dir.delete(recursive: true).catchError((_) => dir);
  }
}

/// Attend que le fichier existe et ne grossisse plus.
Future<Uint8List?> _whenWritten(File file) async {
  var last = -1;
  for (var i = 0; i < 1500; i++) {
    final size = file.existsSync() ? file.lengthSync() : -1;
    if (size >= 0 && size == last) return file.readAsBytes();
    last = size;
    await Future<void>.delayed(const Duration(milliseconds: 20));
  }
  return null;
}
