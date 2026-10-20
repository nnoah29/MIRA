import 'package:flutter_test/flutter_test.dart';
import 'package:html/parser.dart' as html_parser;
import 'package:mira/engine/mail_layout.dart';

String _style(String html, String selector) =>
    html_parser
        .parse(prepareMailHtml(html))
        .querySelector(selector)!
        .attributes['style'] ??
    '';

void main() {
  test('table align=center : l’attribut part, des marges automatiques restent', () {
    const html =
        '<table align="center" style="width:512px"><tr><td>a</td></tr></table>';
    final doc = html_parser.parse(prepareMailHtml(html));
    final table = doc.querySelector('table')!;
    expect(table.attributes.containsKey('align'), isFalse);
    expect(table.attributes['style'], contains('margin-left:auto'));
    expect(table.attributes['style'], contains('margin-right:auto'));
  });

  test('table align=right ne garde qu’une marge gauche automatique', () {
    const html = '<table align="right"><tr><td>a</td></tr></table>';
    expect(_style(html, 'table'), 'margin-left:auto;');
  });

  test('une table à une colonne de 100 % reçoit sa largeur CSS', () {
    const html = '<table width="100%"><tr><td>a</td></tr></table>';
    expect(_style(html, 'table'), contains('width:100%'));
  });

  test('une largeur fixe en pixels reste ignorée', () {
    const html = '<table width="600"><tr><td>a</td></tr></table>';
    expect(_style(html, 'table'), isEmpty);
  });

  test('une ligne avatar | texte à largeur fixe n’est pas étirée', () {
    const html =
        '<table width="100%"><tr><td width="48">a</td>'
        '<td><table width="100%"><tr><td>b</td></tr></table></td></tr></table>';
    expect(_style(html, 'table'), isEmpty);
  });

  test('un bloc d’arrondi reçoit le cadre de la cellule, la cellule garde le reste', () {
    const html =
        '<table><tr><td style="border:1px solid #ccc;border-radius:8px;padding:4px;text-align:center">x</td></tr></table>';
    final cell = html_parser.parse(prepareMailHtml(html)).querySelector('td')!;
    expect(cell.attributes['style'], 'text-align:center');
    final wrapper = cell.querySelector('div')!;
    expect(wrapper.attributes['style'], contains('border-radius:8px'));
    expect(wrapper.attributes['style'], contains('padding:4px'));
    expect(wrapper.text, 'x');
  });
}
