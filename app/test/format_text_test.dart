import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/format.dart';

/// Une chaîne est bien formée si aucune moitié de paire de substitution ne reste seule.
bool _wellFormed(String text) =>
    !text.runes.any((rune) => rune >= 0xD800 && rune <= 0xDFFF);

void main() {
  test('clipText ne sépare jamais un emoji à la frontière de coupe', () {
    final text = '${'a' * 169}🦾 suite';
    final clipped = clipText(text, 170);
    expect(_wellFormed(clipped), isTrue);
    expect(clipped, 'a' * 169);
  });

  test('clipText garde un texte court et coupe un texte simple net', () {
    expect(clipText('court', 170), 'court');
    expect(clipText('abcdef', 3), 'abc');
  });

  test('clipText garde un emoji qui tient entièrement', () {
    expect(clipText('ab🦾cd', 4), 'ab🦾');
  });

  test('initials reste bien formé avec des lettres hors plan de base', () {
    final result = initials('𝐉𝐨hn 𝐃oe');
    expect(_wellFormed(result), isTrue);
    expect(result.runes.length, 2);
  });

  test('initials d’un seul mot hors plan de base reste bien formé', () {
    final result = initials('𝐉𝐨𝐡𝐧');
    expect(_wellFormed(result), isTrue);
    expect(result.runes.length, 2);
  });
}
