import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/web_document.dart';

String _policy(String doc) =>
    RegExp(r'<meta http-equiv="Content-Security-Policy" content="([^"]*)">')
        .firstMatch(doc)!
        .group(1)!;

void main() {
  test('la politique interdit formulaires, base, cadres et objets', () {
    for (final remote in [false, true]) {
      final policy = _policy(webDocument('<p>x</p>', remoteImages: remote));
      expect(policy, contains("form-action 'none'"));
      expect(policy, contains("base-uri 'none'"));
      expect(policy, contains("frame-src 'none'"));
      expect(policy, contains("object-src 'none'"));
    }
  });

  test('une redirection automatique du courriel est retirée', () {
    final variants = [
      '<meta http-equiv="refresh" content="0;url=https://a.fr">',
      "<META HTTP-EQUIV='Refresh' CONTENT='0; URL=data:text/html,x'>",
      '<meta content="0;url=https://a.fr" http-equiv=refresh>',
      '<meta\n http-equiv = "refresh"\n content="1">',
    ];
    for (final tag in variants) {
      final doc = webDocument(
        '<html><head>$tag</head><p>t</p></html>',
        remoteImages: false,
      );
      expect(doc.toLowerCase(), isNot(contains('refresh')), reason: tag);
      expect(doc, contains('<p>t</p>'));
    }
  });

  test('la balise base du courriel est retirée', () {
    final doc = webDocument(
      '<head><base href="https://evil.fr/"></head><a href="x">l</a>',
      remoteImages: false,
    );
    expect(doc.toLowerCase(), isNot(contains('<base')));
    expect(doc, contains('<a href="x">l</a>'));
  });

  test('une valeur http-equiv déguisée en entité HTML est retirée', () {
    final doc = webDocument(
      '<meta http-equiv="ref&#114;esh" content="0;url=https://a.fr"><p>t</p>',
      remoteImages: false,
    );
    expect(doc, isNot(contains('http-equiv="ref')));
    expect(doc, contains('<p>t</p>'));
  });

  test('les http-equiv inoffensifs (type de contenu) sont conservés', () {
    const tag = '<meta http-equiv="Content-Type" content="text/html">';
    expect(webDocument(tag, remoteImages: false), contains(tag));
  });

  test('les autres balises meta restent intactes', () {
    final doc = webDocument(
      '<meta charset="utf-8"><meta name="viewport" content="width=600">',
      remoteImages: false,
    );
    expect(doc, contains('<meta charset="utf-8">'));
    expect(doc, contains('name="viewport" content="width=600"'));
  });
}
