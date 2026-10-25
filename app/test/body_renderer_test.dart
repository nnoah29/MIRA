import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/body_renderer.dart';

void main() {
  test('une notification textuelle reste simple', () {
    const html =
        '<div>Merged <a href="https://x.org/6">#6</a> into main.</div><p>-- <br>Reply to this email</p>';
    expect(isSimpleHtml(html), isTrue);
  });

  test('citations imbriquées, signature et logo restent simples', () {
    const html =
        '<div>Bonjour</div><blockquote><blockquote>Re</blockquote></blockquote><img src="cid:logo">';
    expect(isSimpleHtml(html), isTrue);
  });

  test('une table de mise en page imbriquée exige la vue web', () {
    const html =
        '<table><tr><td><table><tr><td>x</td></tr></table></td></tr></table>';
    expect(isSimpleHtml(html), isFalse);
  });

  test('le CSS de mise en page exige la vue web', () {
    expect(
      isSimpleHtml('<style>@media (max-width:600px){a{color:red}}</style>x'),
      isFalse,
    );
    expect(isSimpleHtml('<div style="position:absolute">x</div>'), isFalse);
    expect(isSimpleHtml('<div style="background:url(a.png)">x</div>'), isFalse);
  });

  test('trop d\'images ou un HTML volumineux exigent la vue web', () {
    expect(isSimpleHtml('<img><img><img><img>'), isFalse);
    expect(isSimpleHtml('<p>${'a' * 20001}</p>'), isFalse);
  });

  test('les balises actives exigent la vue web', () {
    expect(isSimpleHtml('<form><input></form>'), isFalse);
    expect(isSimpleHtml('<svg></svg>'), isFalse);
  });

  test('sans HTML d\'origine, jamais de vue web', () {
    expect(usesWebView(null), isFalse);
  });

  test(
    'le mode d’affichage enregistré se relit, un texte inconnu donne auto',
    () {
      expect(HtmlMode.parse('web'), HtmlMode.web);
      expect(HtmlMode.parse('widgets'), HtmlMode.widgets);
      expect(HtmlMode.parse('auto'), HtmlMode.auto);
      expect(HtmlMode.parse(null), HtmlMode.auto);
      expect(HtmlMode.parse('autre'), HtmlMode.auto);
    },
  );

  test('le mode impose le rendu quel que soit le message (Linux)', () {
    const simple = '<p>Bonjour</p>';
    const rich = '<table></table><table></table>';
    expect(usesWebView(simple, HtmlMode.auto), isFalse);
    expect(usesWebView(rich, HtmlMode.auto), isTrue);
    expect(usesWebView(simple, HtmlMode.web), isTrue);
    expect(usesWebView(rich, HtmlMode.widgets), isFalse);
    expect(usesWebView(null, HtmlMode.web), isFalse);
  }, skip: !webViewEnabled);
}
