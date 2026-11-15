import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/dictation_types.dart';
import 'package:mira/voice/voice_preferences.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<VoicePreferences> prefsWith(Map<String, Object> values) async {
  SharedPreferences.setMockInitialValues(values);
  return VoicePreferences(await SharedPreferences.getInstance());
}

void main() {
  test('sans choix, la langue suit celle de l’interface', () async {
    final p = await prefsWith({});
    expect(p.language(uiLocale: 'en_US'), DictationLanguage.en);
    expect(p.language(uiLocale: 'fr_FR'), DictationLanguage.fr);
  });

  test('le choix enregistré l’emporte sur la locale', () async {
    final p = await prefsWith({});
    await p.setLanguage(DictationLanguage.en);
    expect(p.language(uiLocale: 'fr'), DictationLanguage.en);
  });

  test('une valeur enregistrée inconnue retombe sur la locale', () async {
    final p = await prefsWith({'voice.language': 'klingon'});
    expect(p.language(uiLocale: 'en'), DictationLanguage.en);
  });
}
