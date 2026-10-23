import 'dart:io';

/// Les messages simples s'affichent par le moteur de widgets ; mettre à `false` pour renvoyer tous les HTML à la vue web.
const bool simpleBodiesNative = true;

/// Au-delà, le HTML est jugé trop riche pour le rendu natif.
const _maxSimpleLength = 20000;
const _maxSimpleImages = 3;
const _maxSimpleTables = 1;

final _heavyTags = RegExp(
  r'<(form|iframe|video|audio|svg|canvas|object|embed|picture|center)\b',
);
final _heavyCss = RegExp(
  r'(@media|@font-face|position\s*:\s*(absolute|fixed)|float\s*:|background(-image)?\s*:[^;"}]*url\()',
);

/// Sous Linux, le corps HTML s'affiche dans une vraie vue web (WebKitGTK) ; ailleurs, par le moteur de widgets.
final bool webViewEnabled = Platform.isLinux;

/// Le HTML est-il assez simple (texte, liens, citations, quelques images) pour se passer de la vue web ?
bool isSimpleHtml(String html) {
  if (html.length > _maxSimpleLength) return false;
  final lower = html.toLowerCase();
  if (_heavyTags.hasMatch(lower) || _heavyCss.hasMatch(lower)) return false;
  if ('<img'.allMatches(lower).length > _maxSimpleImages) return false;
  return '<table'.allMatches(lower).length <= _maxSimpleTables;
}

/// Ce corps passe-t-il par la vue web ? Décision stable pour un même HTML.
bool usesWebView(String? htmlRaw) {
  if (!webViewEnabled || htmlRaw == null) return false;
  return !(simpleBodiesNative && isSimpleHtml(htmlRaw));
}
