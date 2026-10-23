import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/web_document.dart';

final _png = Uint8List.fromList([
  0x89,
  0x50,
  0x4E,
  0x47,
  0x0D,
  0x0A,
  0x1A,
  0x0A,
  1,
  2,
]);
final _jpeg = Uint8List.fromList([0xFF, 0xD8, 0xFF, 0xE0, 1, 2]);

void main() {
  test('la politique de contenu bloque tout sauf les images incorporées', () {
    final doc = webDocument('<p>Bonjour</p>', remoteImages: false);
    expect(doc, contains('Content-Security-Policy'));
    expect(doc, contains("default-src 'none'"));
    expect(doc, contains('img-src data:'));
    expect(doc, isNot(contains('https:')));
    expect(doc, contains('<p>Bonjour</p>'));
  });

  test('autoriser les images distantes ajoute seulement img-src https', () {
    final doc = webDocument('<p>x</p>', remoteImages: true);
    expect(doc, contains('img-src data: https: http:'));
    expect(doc, contains("default-src 'none'"));
    expect(doc, isNot(contains('script-src')));
  });

  test(
    'la politique précède tout le contenu, même un script avant le head',
    () {
      final doc = webDocument(
        '<html><script>alert(1)</script><head><title>t</title></head><body>b</body></html>',
        remoteImages: false,
      );
      expect(doc, startsWith('<meta http-equiv="Content-Security-Policy"'));
      expect(
        doc.indexOf('Content-Security-Policy'),
        lessThan(doc.indexOf('<script>')),
      );
    },
  );

  test('le doctype reste en tête, la politique juste après', () {
    final doc = webDocument(
      '<!DOCTYPE html><html><head></head><body>b</body></html>',
      remoteImages: false,
    );
    expect(doc, startsWith('<!DOCTYPE html><meta http-equiv='));
  });

  test('la page ne défile pas elle-même', () {
    final doc = webDocument('texte', remoteImages: false);
    expect(doc, contains('overflow-y:hidden'));
    expect(doc, contains('texte'));
  });

  test('les images cid: deviennent des données locales du bon type', () {
    final doc = webDocument(
      '<img src="cid:logo"><img src="cid:photo"><img src="CID:<up>">',
      remoteImages: false,
      inline: {'logo': _png, 'photo': _jpeg, 'up': _png},
    );
    expect(doc, contains('data:image/png;base64,${base64Encode(_png)}'));
    expect(doc, contains('data:image/jpeg;base64,${base64Encode(_jpeg)}'));
    expect(doc, isNot(contains('cid:')));
  });

  test('un cid inconnu reste tel quel, sans rien charger', () {
    final doc = webDocument('<img src="cid:absent">', remoteImages: false);
    expect(doc, contains('cid:absent'));
  });

  test(
    'les éventuels guillemets du contenu ne sortent pas de la politique',
    () {
      final doc = webDocument('<p>"a" \'b\'</p>', remoteImages: false);
      final policy = RegExp(
        r'<meta http-equiv="Content-Security-Policy" content="([^"]*)">',
      ).firstMatch(doc)!.group(1)!;
      expect(policy, isNot(contains('<')));
    },
  );

  test('une image citée plusieurs fois est encodée une seule fois', () {
    final doc = webDocument(
      '<img src="cid:a"><img src="cid:a">',
      remoteImages: false,
      inline: {'a': _png},
    );
    final b64 = base64Encode(_png);
    expect(b64.allMatches(doc).length, 2);
    expect(doc, isNot(contains('cid:a')));
  });
}
