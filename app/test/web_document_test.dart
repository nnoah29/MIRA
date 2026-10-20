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

  test('la politique est placée dans le head existant, en premier', () {
    final doc = webDocument(
      '<html><head><title>t</title></head><body>b</body></html>',
      remoteImages: false,
    );
    expect(
      doc.indexOf('Content-Security-Policy'),
      lessThan(doc.indexOf('<title>')),
    );
    expect('<head'.allMatches(doc).length, 1);
  });

  test('un document sans head en reçoit un', () {
    final doc = webDocument('<html><body>b</body></html>', remoteImages: false);
    expect(
      doc.indexOf('Content-Security-Policy'),
      lessThan(doc.indexOf('<body>')),
    );
  });

  test('un fragment sans html ni head est enveloppé', () {
    final doc = webDocument('texte', remoteImages: false);
    expect(doc, contains('<head>'));
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
}
