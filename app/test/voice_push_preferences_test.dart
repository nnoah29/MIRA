import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/voice_preferences.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<VoicePreferences> prefs([Map<String, Object> initial = const {}]) async {
  SharedPreferences.setMockInitialValues(initial);
  return VoicePreferences(await SharedPreferences.getInstance());
}

void main() {
  test(
    'valeurs par défaut : Espace actif, 400 ms, modèle gardé 3 minutes',
    () async {
      final p = await prefs();
      expect(p.holdSpaceEnabled, isTrue);
      expect(p.holdDelay, const Duration(milliseconds: 400));
      expect(p.keepWarm, const Duration(minutes: 3));
    },
  );

  test('chaque réglage se mémorise et se relit', () async {
    SharedPreferences.setMockInitialValues({});
    final raw = await SharedPreferences.getInstance();
    final p = VoicePreferences(raw);
    await p.setHoldSpaceEnabled(false);
    await p.setHoldDelay(const Duration(milliseconds: 600));
    await p.setKeepWarm(const Duration(minutes: 10));
    final again = VoicePreferences(raw);
    expect(again.holdSpaceEnabled, isFalse);
    expect(again.holdDelay, const Duration(milliseconds: 600));
    expect(again.keepWarm, const Duration(minutes: 10));
  });

  test('0 minute : le modèle n’est pas gardé', () async {
    final p = await prefs();
    await p.setKeepWarm(Duration.zero);
    expect(p.keepWarm, Duration.zero);
  });

  test('une valeur enregistrée illisible retombe sur le défaut', () async {
    final p = await prefs({
      'voice.holdDelayMs': 'beaucoup',
      'voice.keepWarmMinutes': -4,
    });
    expect(p.holdDelay, const Duration(milliseconds: 400));
    expect(p.keepWarm, const Duration(minutes: 3));
  });

  test('le délai de maintien reste dans une plage raisonnable', () async {
    final p = await prefs({'voice.holdDelayMs': 5});
    expect(p.holdDelay, const Duration(milliseconds: 400));
    final q = await prefs({'voice.holdDelayMs': 99999});
    expect(q.holdDelay, const Duration(milliseconds: 400));
  });
}
