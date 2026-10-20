import 'dart:convert';
import 'dart:typed_data';

final _cid = RegExp(r'cid:<?([^"\s>]+)>?', caseSensitive: false);
final _head = RegExp(r'<head(\s[^>]*)?>', caseSensitive: false);
final _html = RegExp(r'<html(\s[^>]*)?>', caseSensitive: false);

/// Politique de contenu du document : rien ne se charge, sauf les images incorporées (et distantes si autorisées).
String contentPolicy({required bool remoteImages}) {
  final images = remoteImages ? 'data: https: http:' : 'data:';
  return "default-src 'none'; img-src $images; style-src 'unsafe-inline'; font-src data:";
}

/// Document à afficher dans la vue web : le HTML reçu, tel quel, sous une politique de contenu stricte, avec les images `cid:` remplacées par leurs données.
String webDocument(
  String html, {
  required bool remoteImages,
  Map<String, Uint8List> inline = const {},
}) {
  final meta =
      '<meta http-equiv="Content-Security-Policy" content="${contentPolicy(remoteImages: remoteImages)}">';
  return _withHead(_inlineImages(html, inline), meta);
}

String _withHead(String html, String meta) {
  final head = _head.firstMatch(html);
  if (head != null) {
    return html.replaceRange(head.end, head.end, meta);
  }
  final root = _html.firstMatch(html);
  if (root != null) {
    return html.replaceRange(root.end, root.end, '<head>$meta</head>');
  }
  return '<html><head>$meta</head><body>$html</body></html>';
}

String _inlineImages(String html, Map<String, Uint8List> inline) {
  if (inline.isEmpty) return html;
  return html.replaceAllMapped(_cid, (found) {
    final bytes = inline[found.group(1)];
    if (bytes == null) return found.group(0)!;
    return 'data:${_mimeOf(bytes)};base64,${base64Encode(bytes)}';
  });
}

/// Type d'une image d'après ses premiers octets.
String _mimeOf(Uint8List b) {
  bool starts(List<int> magic) =>
      b.length >= magic.length &&
      [for (var i = 0; i < magic.length; i++) b[i] == magic[i]].every((x) => x);
  if (starts([0x89, 0x50, 0x4E, 0x47])) return 'image/png';
  if (starts([0xFF, 0xD8, 0xFF])) return 'image/jpeg';
  if (starts([0x47, 0x49, 0x46, 0x38])) return 'image/gif';
  if (starts([0x52, 0x49, 0x46, 0x46])) return 'image/webp';
  return 'application/octet-stream';
}
