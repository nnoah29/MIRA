import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/web_navigation.dart';

void main() {
  group('pendant le chargement du document', () {
    test('about: et data: chargent le document lui-même', () {
      expect(
        navigationVerdict('about:blank', loading: true),
        NavigationVerdict.load,
      );
      expect(
        navigationVerdict('data:text/html;base64,AAAA', loading: true),
        NavigationVerdict.load,
      );
    });

    test('une adresse web part dans le navigateur du système', () {
      expect(
        navigationVerdict('https://exemple.fr/a', loading: true),
        NavigationVerdict.openExternal,
      );
    });
  });

  group('une fois le document chargé', () {
    test('un lien data: est refusé (page sans politique de contenu)', () {
      expect(
        navigationVerdict('data:text/html,<script>1</script>', loading: false),
        NavigationVerdict.block,
      );
      expect(
        navigationVerdict('DATA:text/html,x', loading: false),
        NavigationVerdict.block,
      );
    });

    test('about:, javascript:, file: et blob: sont refusés', () {
      for (final url in [
        'about:blank',
        'javascript:alert(1)',
        'file:///etc/passwd',
        'blob:https://x/1',
        'ftp://x/y',
      ]) {
        expect(navigationVerdict(url, loading: false), NavigationVerdict.block);
      }
    });

    test('http, https et mailto s\'ouvrent dans le système', () {
      for (final url in [
        'http://a.fr',
        'https://a.fr/b?c=d',
        'mailto:a@b.fr',
      ]) {
        expect(
          navigationVerdict(url, loading: false),
          NavigationVerdict.openExternal,
        );
      }
    });

    test('une adresse illisible est refusée', () {
      expect(navigationVerdict('', loading: false), NavigationVerdict.block);
      expect(
        navigationVerdict('::::', loading: false),
        NavigationVerdict.block,
      );
    });
  });
}
