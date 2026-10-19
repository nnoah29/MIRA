import 'package:html/dom.dart' as dom;
import 'package:html/parser.dart' as html_parser;

import 'css_inliner.dart';

final _wordBreak = RegExp(r'word-break\s*:[^;]*;?', caseSensitive: false);
final _maxWidth = RegExp(
  r'(?<![-\w])max-width\s*:\s*([^;]+);?',
  caseSensitive: false,
);
final _minWidth = RegExp(
  r'(?<![-\w])min-width\s*:[^;]*;?',
  caseSensitive: false,
);

/// Prépare le HTML d'un courriel pour le rendu : styles reportés, puis mise en page corrigée.
String prepareMailHtml(String html) => fixLayout(inlineStyles(html));

/// Neutralise ce qui écrase les colonnes dans le moteur de rendu : `word-break` et bornes de largeur des tableaux.
String fixLayout(String html) {
  final lower = html.toLowerCase();
  if (!lower.contains('word-break') && !lower.contains('width:')) return html;
  final doc = html_parser.parse(html);
  for (final e in doc.querySelectorAll('[style]')) {
    e.attributes['style'] = e.attributes['style']!.replaceAll(_wordBreak, '');
  }
  doc.querySelectorAll('table[style]').forEach(_boundOutside);
  return doc.outerHtml;
}

void _boundOutside(dom.Element table) {
  final style = table.attributes['style']!;
  final max = _maxWidth.firstMatch(style)?.group(1)?.trim();
  table.attributes['style'] = style
      .replaceAll(_maxWidth, '')
      .replaceAll(_minWidth, '');
  if (max == null || table.parent == null) return;
  final box = dom.Element.tag('div')
    ..attributes['style'] = 'max-width:$max;margin:0 auto';
  table.replaceWith(box);
  box.append(table);
}
