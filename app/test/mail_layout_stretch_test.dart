import 'package:flutter_test/flutter_test.dart';
import 'package:html/parser.dart' as html_parser;
import 'package:mira/engine/mail_layout.dart';

String _style(String html) =>
    html_parser
        .parse(prepareMailHtml(html))
        .querySelector('table')!
        .attributes['style'] ??
    '';

void main() {
  test('libellé | montant aligné à droite : la table de 100 % est étirée', () {
    const html =
        '<table width="100%"><tr><td>Total</td><td align="right">3,000 F</td></tr></table>';
    expect(_style(html), contains('width:100%'));
  });

  test('l’alignement peut venir du style : text-align:right', () {
    const html =
        '<table width="100%"><tr><td>Total</td><td style="text-align:right">3 F</td></tr></table>';
    expect(_style(html), contains('width:100%'));
  });

  test('une cellule centrée suffit aussi', () {
    const html =
        '<table width="100%"><tr><td>a</td><td align="center">b</td></tr></table>';
    expect(_style(html), contains('width:100%'));
  });

  test('deux cellules sans alignement restent ajustées à leur contenu', () {
    const html = '<table width="100%"><tr><td>a</td><td>b</td></tr></table>';
    expect(_style(html), isEmpty);
  });
}
