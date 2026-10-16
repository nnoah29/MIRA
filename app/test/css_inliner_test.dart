import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/css_inliner.dart';

void main() {
  test('les règles <style> passent en style en ligne, le style en ligne gagne', () {
    final out = inlineStyles(
      '<style>.a{color:#f00} td{padding:4px} a:hover{color:blue}</style>'
      '<p class="a" style="color:#00f">X</p><table><tr><td>Y</td></tr></table>',
    );
    expect(out, contains('color:#f00;color:#00f'));
    expect(out, contains('padding:4px'));
    expect(out, isNot(contains('style="color:blue')));
  });

  test('sans <style>, le HTML est rendu tel quel', () {
    const html = '<p style="color:red">A</p>';
    expect(inlineStyles(html), html);
  });

  test('une feuille illisible ne casse rien', () {
    expect(inlineStyles('<style>{{{</style><p>ok</p>'), contains('ok'));
  });
}
