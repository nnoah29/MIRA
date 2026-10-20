import 'dart:math';

import 'package:html/dom.dart' as dom;
import 'package:html/parser.dart' as html_parser;

import 'css_inliner.dart';

final _wordBreak = RegExp(r'word-break\s*:[^;]*;?', caseSensitive: false);
final _maxWidth = RegExp(
  r'(?<![-\w])max-width\s*:\s*([^;]+);?',
  caseSensitive: false,
);
final _radiusRule = RegExp(
  r'(?<![-\w])border-radius\s*:\s*[^;]*[1-9]',
  caseSensitive: false,
);
final _boxRule = RegExp(
  r'^(border(?!-collapse|-spacing)|overflow|background|padding)',
  caseSensitive: false,
);
final _widthRule = RegExp(r'(?<![-\w])width\s*:');
final _percentWidth = RegExp(r'^\s*(\d+(?:\.\d+)?)\s*%\s*$');
final _minWidth = RegExp(
  r'(?<![-\w])min-width\s*:[^;]*;?',
  caseSensitive: false,
);

/// Prépare le HTML d'un courriel pour le rendu : styles reportés, puis mise en page corrigée.
String prepareMailHtml(String html) => fixLayout(inlineStyles(html));

/// Neutralise ce qui écrase les colonnes dans le moteur de rendu : `word-break`, bornes de largeur et `align` des tableaux.
String fixLayout(String html) {
  final lower = html.toLowerCase();
  final styled =
      lower.contains('word-break') ||
      lower.contains('width:') ||
      lower.contains('border-radius');
  final attributes =
      lower.contains('<table') &&
      (lower.contains('align=') || lower.contains('width='));
  if (!styled && !attributes) return html;
  final doc = html_parser.parse(html);
  doc.querySelectorAll('table[align]').forEach(_placeTable);
  doc.querySelectorAll('table[width]').forEach(_sizeTable);
  doc.querySelectorAll('td[style], th[style]').forEach(_roundCell);
  for (final e in doc.querySelectorAll('[style]')) {
    e.attributes['style'] = e.attributes['style']!.replaceAll(_wordBreak, '');
  }
  doc.querySelectorAll('table[style]').forEach(_boundOutside);
  return doc.outerHtml;
}

/// `<table align>` place la table dans la page ; le moteur de rendu en ferait un `text-align` hérité par tout son contenu.
void _placeTable(dom.Element table) {
  final margin = switch (table.attributes.remove('align')?.toLowerCase()) {
    'center' => 'margin-left:auto;margin-right:auto;',
    'right' => 'margin-left:auto;',
    _ => '',
  };
  if (margin.isEmpty) return;
  table.attributes['style'] = '$margin${table.attributes['style'] ?? ''}';
}

/// Le moteur ne dessine pas `border-radius` sur une cellule : le cadre (bordure, arrondi, fond, marges) passe sur un bloc qui enveloppe son contenu.
void _roundCell(dom.Element cell) {
  final style = cell.attributes['style'] ?? '';
  if (!_radiusRule.hasMatch(style)) return;
  final declarations = style.split(';').map((d) => d.trim()).where(_filled);
  final frame = declarations.where(_boxRule.hasMatch).toList();
  final rest = declarations.where((d) => !_boxRule.hasMatch(d)).toList();
  final wrapper = dom.Element.tag('div')..attributes['style'] = frame.join(';');
  for (final child in cell.nodes.toList()) {
    wrapper.append(child);
  }
  cell.append(wrapper);
  cell.attributes['style'] = rest.join(';');
}

bool _filled(String text) => text.isNotEmpty;

/// L'attribut `width` en pourcentage d'une table devient une largeur CSS, que le moteur de rendu applique (les largeurs fixes restent ignorées).
void _sizeTable(dom.Element table) {
  final style = table.attributes['style'] ?? '';
  final found = _percentWidth.firstMatch(table.attributes['width'] ?? '');
  if (found == null || _widthRule.hasMatch(style) || !_spreads(table)) return;
  table.attributes['style'] = 'width:${found.group(1)}%;$style';
}

/// Le moteur partage une table de largeur imposée en colonnes égales : on ne l'impose qu'aux tables à une colonne et à celles qui poussent une cellule à droite.
bool _spreads(dom.Element table) {
  final rows = [
    for (final part in table.children)
      ...(part.localName == 'tr' ? [part] : part.children),
  ];
  final cells = [for (final row in rows) row.children.where(_isCell).toList()];
  final widest = cells.fold<int>(0, (most, row) => max(most, row.length));
  final all = cells.expand((row) => row).toList();
  return widest <= 1 || (all.any(_pushesRight) && !all.any(_hasFixedWidth));
}

/// Une cellule à largeur fixe garde sa largeur : l'autre prend le reste, sans qu'on étire la table.
bool _hasFixedWidth(dom.Element cell) =>
    cell.attributes.containsKey('width') ||
    _widthRule.hasMatch(cell.attributes['style'] ?? '');

bool _isCell(dom.Element e) => e.localName == 'td' || e.localName == 'th';

/// Une cellule dont l'unique contenu est une table de 100 % prend l'espace restant de sa ligne (l'astuce qui pousse un élément à droite).
bool _pushesRight(dom.Element cell) {
  final only = cell.children.length == 1 ? cell.children.first : null;
  return only?.localName == 'table' &&
      _percentWidth.hasMatch(only?.attributes['width'] ?? '');
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
