import 'package:flutter/widgets.dart';
import 'package:flutter_widget_from_html_core/flutter_widget_from_html_core.dart';
import 'package:html/dom.dart' as dom;

/// Juste après les marges du moteur de rendu (`BoxModel.margin`), avant `sizingMinWidthZero`.
const _afterMargin = 5000000000000000 + 6500000000;

/// Fabrique de widgets des courriels : ajoute au moteur ce qu'un navigateur fait et qu'il ne fait pas.
class MailWidgetFactory extends WidgetFactory {
  @override
  void parse(BuildTree tree) {
    super.parse(tree);
    final alignment = autoMarginAlignment(tree.element);
    if (alignment == null) return;
    tree.register(
      BuildOp.v2(
        debugLabel: 'margin: auto',
        priority: _afterMargin,
        onRenderBlock: (_, placeholder) => placeholder.wrapWith(
          (_, child) =>
              Align(alignment: alignment, heightFactor: 1, child: child),
        ),
      ),
    );
  }
}

/// Un bloc de largeur bornée (`width`, `max-width`) avec des marges `auto` est centré, ou poussé à droite si seule la marge gauche est `auto`.
/// Le moteur l'étirerait sur toute la largeur d'une cellule de tableau ; un navigateur le place.
Alignment? autoMarginAlignment(dom.Element e) {
  var bounded = false;
  var left = '';
  var right = '';
  for (final (name, value) in _declarations(e.attributes['style'] ?? '')) {
    switch (name) {
      case 'width' || 'max-width':
        bounded = !value.startsWith('100%') && value != 'auto';
      case 'margin':
        (left, right) = _horizontal(value.split(RegExp(r'\s+')));
      case 'margin-left':
        left = value;
      case 'margin-right':
        right = value;
    }
  }
  if (!bounded || !left.startsWith('auto')) return null;
  return right.startsWith('auto') ? Alignment.topCenter : Alignment.topRight;
}

/// Déclarations d'un style en ligne, dans l'ordre, noms en minuscules.
Iterable<(String, String)> _declarations(String style) sync* {
  for (final declaration in style.split(';')) {
    final colon = declaration.indexOf(':');
    if (colon < 0) continue;
    yield (
      declaration.substring(0, colon).trim().toLowerCase(),
      declaration.substring(colon + 1).trim().toLowerCase(),
    );
  }
}

/// Marges gauche et droite du raccourci `margin` (1 à 4 valeurs).
(String, String) _horizontal(List<String> parts) => switch (parts.length) {
  1 => (parts[0], parts[0]),
  2 || 3 => (parts[1], parts[1]),
  _ => (parts[3], parts[1]),
};
