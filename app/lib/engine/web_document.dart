import 'dart:convert';
import 'dart:typed_data';

final _cid = RegExp(r'cid:<?([^"\s>]+)>?', caseSensitive: false);
final _doctype = RegExp(r'^\s*<!doctype[^>]*>', caseSensitive: false);

/// Politique de contenu du document : rien ne se charge ni ne s'exécute, sauf les images incorporées (et distantes si autorisées).
String contentPolicy({required bool remoteImages}) {
  final images = remoteImages ? 'data: https: http:' : 'data:';
  return "default-src 'none'; img-src $images; style-src 'unsafe-inline'; font-src data:; "
      "form-action 'none'; base-uri 'none'; frame-src 'none'; object-src 'none'";
}

final _meta = RegExp(
  r'''<meta\b(?:[^>"']|"[^"]*"|'[^']*')*>''',
  caseSensitive: false,
);
final _equiv = RegExp(
  r'''\bhttp-equiv\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+))''',
  caseSensitive: false,
);
final _base = RegExp(r'<base\b[^>]*>', caseSensitive: false);
const _keptEquiv = {'content-type', 'x-ua-compatible', 'content-language'};

/// Garde une balise `meta` sans `http-equiv` ou à `http-equiv` inoffensif ; toute autre est retirée.
String _keepHarmlessMeta(Match tag) {
  final equiv = _equiv.firstMatch(tag.group(0)!);
  if (equiv == null) return tag.group(0)!;
  final value = (equiv.group(1) ?? equiv.group(2) ?? equiv.group(3) ?? '')
      .trim()
      .toLowerCase();
  return _keptEquiv.contains(value) ? tag.group(0)! : '';
}

/// Retire du courriel ce qui redirige ou change la base des adresses (`refresh`, `<base>`, `http-equiv` inconnu).
String _withoutNavigation(String html) =>
    html.replaceAllMapped(_meta, _keepHarmlessMeta).replaceAll(_base, '');

/// La page ne défile pas elle-même : c'est le lecteur qui la fait défiler, à sa hauteur complète.
/// Ses hauteurs ne suivent pas celle de la vue (`height:100%`, fréquent dans les courriels) : la vue prend la hauteur du contenu, qui sinon grandirait avec elle sans fin.
const _noOwnScroll =
    '<style>html{overflow-y:hidden !important}'
    'html,body{height:auto !important;min-height:0 !important}</style>';

/// Sur mobile, la page se met en page à la largeur de l'écran (sans cela, la vue la suppose large de 980 px et la réduit).
const _viewport =
    '<meta name="viewport" content="width=device-width, initial-scale=1">';

/// Document à afficher dans la vue web : le HTML reçu, tel quel, avec les images `cid:` remplacées par leurs données.
/// La politique de contenu est la toute première balise, avant tout script que contiendrait le courriel (seul le `doctype` la précède, pour garder le mode de rendu).
String webDocument(
  String html, {
  required bool remoteImages,
  Map<String, Uint8List> inline = const {},
  bool viewport = false,
}) {
  final meta =
      '<meta http-equiv="Content-Security-Policy" content="${contentPolicy(remoteImages: remoteImages)}">${viewport ? _viewport : ''}$_noOwnScroll';
  final body = _inlineImages(_withoutNavigation(html), inline);
  final doctype = _doctype.firstMatch(body);
  final at = doctype?.end ?? 0;
  return body.replaceRange(at, at, meta);
}

String _inlineImages(String html, Map<String, Uint8List> inline) {
  if (inline.isEmpty) return html;
  final encoded = <String, String>{};
  return html.replaceAllMapped(_cid, (found) {
    final id = found.group(1)!;
    final bytes = inline[id];
    if (bytes == null) return found.group(0)!;
    return encoded[id] ??=
        'data:${_mimeOf(bytes)};base64,${base64Encode(bytes)}';
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
