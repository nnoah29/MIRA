import 'dart:io';
import 'dart:typed_data';

import 'engine_client.dart';

const _maxInline = 20;
const _maxBytes = 5 * 1024 * 1024;

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
  final out = <String, Uint8List>{};
  for (final a in attachments) {
    final cid = cidOf(a);
    if (cid == null || !wanted.contains(cid) || out.length >= _maxInline) {
      continue;
    }
    if (((a['size'] as num?) ?? 0) > _maxBytes) continue;
    final bytes = await _fetch(client, '${a['id']}');
    if (bytes != null) out[cid] = bytes;
  }
  return out;
}

Future<Uint8List?> _fetch(EngineClient client, String id) async {
  final dir = await Directory.systemTemp.createTemp('mira-inline');
  final file = File('${dir.path}/image');
  try {
    await client.call(
      req('message', 'save_attachment', {
        'attachment': id,
        'destination': file.path,
      }),
    );
    for (var i = 0; i < 100; i++) {
      if (file.existsSync()) return await file.readAsBytes();
      await Future<void>.delayed(const Duration(milliseconds: 100));
    }
    return null;
  } on EngineException {
    return null;
  } finally {
    await dir.delete(recursive: true).catchError((_) => dir);
  }
}
