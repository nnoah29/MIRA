import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/dictation_types.dart';

void main() {
  test('fromLocale choisit le français pour fr_FR', () {
    expect(DictationLanguage.fromLocale('fr_FR'), DictationLanguage.fr);
  });

  test('fromLocale choisit l’anglais pour en-GB', () {
    expect(DictationLanguage.fromLocale('en-GB'), DictationLanguage.en);
  });

  test('fromLocale se replie sur le français pour une autre langue', () {
    expect(DictationLanguage.fromLocale('de'), DictationLanguage.fr);
  });

  test('le code de l’anglais est en', () {
    expect(DictationLanguage.en.code, 'en');
  });
}
