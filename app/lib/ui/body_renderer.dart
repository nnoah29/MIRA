import '../platform_info.dart';

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

/// Sous Linux (WebKitGTK) et Android (WebView), le corps HTML s'affiche dans une vraie vue web ; ailleurs, par le moteur de widgets.
final bool webViewEnabled = runsOnLinux || runsOnAndroid;

/// Le HTML est-il assez simple (texte, liens, citations, quelques images) pour se passer de la vue web ?
bool isSimpleHtml(String html) {
  if (html.length > _maxSimpleLength) return false;
  final lower = html.toLowerCase();
  if (_heavyTags.hasMatch(lower) || _heavyCss.hasMatch(lower)) return false;
  if ('<img'.allMatches(lower).length > _maxSimpleImages) return false;
  return '<table'.allMatches(lower).length <= _maxSimpleTables;
}

/// Façon d'afficher le HTML sous Linux : choix selon le message, toujours la vue web, ou toujours les widgets.
enum HtmlMode {
  auto('auto'),
  web('web'),
  widgets('widgets');

  final String code;
  const HtmlMode(this.code);

  /// Le mode enregistré ; tout autre texte donne le choix automatique.
  static HtmlMode parse(String? code) =>
      values.firstWhere((m) => m.code == code, orElse: () => auto);
}

/// Ce corps passe-t-il par la vue web ? Décision stable pour un même HTML et un même mode.
bool usesWebView(String? htmlRaw, [HtmlMode mode = HtmlMode.auto]) {
  if (!webViewEnabled || htmlRaw == null) return false;
  return switch (mode) {
    HtmlMode.web => true,
    HtmlMode.widgets => false,
    HtmlMode.auto => !(simpleBodiesNative && isSimpleHtml(htmlRaw)),
  };
}
