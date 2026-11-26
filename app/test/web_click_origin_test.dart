import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/web_navigation.dart';
import 'package:mira/ui/web_mail_body.dart';

void main() {
  test('une navigation demandée par la page n\'ouvre jamais le navigateur', () {
    for (final url in ['http://a.fr', 'https://a.fr/b', 'mailto:a@b.fr']) {
      expect(
        automaticNavigationVerdict(url, loading: true),
        NavigationVerdict.block,
        reason: url,
      );
      expect(
        automaticNavigationVerdict(url, loading: false),
        NavigationVerdict.block,
        reason: url,
      );
    }
  });

  test('le document lui-même se charge toujours, rien d\'autre', () {
    expect(
      automaticNavigationVerdict('about:blank', loading: true),
      NavigationVerdict.load,
    );
    expect(
      automaticNavigationVerdict('data:text/html;base64,AAAA', loading: true),
      NavigationVerdict.load,
    );
    expect(
      automaticNavigationVerdict('data:text/html,x', loading: false),
      NavigationVerdict.block,
    );
  });

  test('un message de clic ne livre que les adresses ouvrables', () {
    expect(clickedLink({'link': 'https://a.fr/x'}), 'https://a.fr/x');
    expect(clickedLink({'link': 'mailto:a@b.fr'}), 'mailto:a@b.fr');
    for (final bad in [
      {'link': 'javascript:alert(1)'},
      {'link': 'file:///etc/passwd'},
      {'link': 'data:text/html,x'},
      {'link': 42},
      {'height': 10},
    ]) {
      expect(clickedLink(bad), isNull, reason: '$bad');
    }
  });

  test('le script de page ne relaie que les vrais clics sur un lien', () {
    expect(webPageScriptForTest, contains('event.isTrusted'));
    expect(webPageScriptForTest, contains("closest('a[href],area[href]')"));
    expect(webPageScriptForTest, contains('preventDefault'));
    expect(webPageScriptForTest, contains('capture: true'));
  });
}
