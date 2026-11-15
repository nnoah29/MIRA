import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/transcript_text.dart';

void main() {
  test('retire les blancs de tête, de fin et réduit les doubles', () {
    expect(
      normalizeTranscript(' Peux-tu m’envoyer  la facture '),
      "Peux-tu m'envoyer la facture",
    );
  });

  test('une chaîne vide reste vide', () {
    expect(normalizeTranscript(''), '');
  });

  test('une chaîne de blancs devient vide', () {
    expect(normalizeTranscript('   '), '');
  });

  test('l’apostrophe typographique devient droite en anglais aussi', () {
    expect(normalizeTranscript('Hi, it’s me.'), "Hi, it's me.");
  });
}
