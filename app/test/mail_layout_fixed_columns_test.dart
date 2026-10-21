import 'package:flutter_test/flutter_test.dart';
import 'package:html/parser.dart' as html_parser;
import 'package:mira/engine/mail_layout.dart';

String _tableStyle(String html) =>
    html_parser
        .parse(prepareMailHtml(html))
        .querySelector('table')!
        .attributes['style'] ??
    '';

void main() {
  test('toutes les cellules à largeur fixe : la largeur de la table est retirée', () {
    const html =
        '<table style="width:350px;margin-left:auto"><tr>'
        '<td style="width:25px">a</td><td style="width:120px">b</td></tr></table>';
    final style = _tableStyle(html);
    expect(style, isNot(contains('width:350px')));
    expect(style, contains('margin-left:auto'));
  });

  test('la largeur en attribut est retirée aussi', () {
    const html =
        '<table width="600"><tr><td width="200">a</td><td width="400">b</td></tr></table>';
    final doc = html_parser.parse(prepareMailHtml(html));
    expect(
      doc.querySelector('table')!.attributes.containsKey('width'),
      isFalse,
    );
  });

  test('une cellule sans largeur garde la largeur de la table', () {
    const html =
        '<table style="width:350px"><tr>'
        '<td style="width:25px">a</td><td>b</td></tr></table>';
    expect(_tableStyle(html), contains('width:350px'));
  });

  test('une table d’une seule cellule garde sa largeur', () {
    const html =
        '<table style="width:350px"><tr><td style="width:350px">a</td></tr></table>';
    expect(_tableStyle(html), contains('width:350px'));
  });

  test('min-width et max-width ne sont pas confondus avec width', () {
    const html =
        '<table style="min-width:100px;width:350px"><tr>'
        '<td style="width:25px">a</td><td style="width:50px">b</td></tr></table>';
    final style = _tableStyle(html);
    expect(style, isNot(contains('width:350px')));
  });
}
