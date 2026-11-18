import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/voice_settings.dart';
import 'package:mira/ui/widgets.dart';
import 'package:mira/voice/voice_services.dart';

import 'voice_fakes.dart';

Key k(String id) => ValueKey(id);

Future<FakeVoice> pumpSettings(WidgetTester tester) async {
  final voice = await FakeVoice.create();
  tester.view.physicalSize = const Size(1200, 2400);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  await tester.pumpWidget(
    VoiceScope(
      services: voice.services,
      child: MaterialApp(
        theme: buildTheme(false),
        home: const Scaffold(
          body: SingleChildScrollView(child: VoiceSettings()),
        ),
      ),
    ),
  );
  await tester.pump();
  return voice;
}

bool isOn(WidgetTester tester, String id) =>
    tester.widget<Btn>(find.byKey(k(id))).on;

void main() {
  testWidgets(
    'la section présente les trois réglages avec leurs valeurs par défaut',
    (t) async {
      final voice = await pumpSettings(t);
      expect(find.byKey(k('voice-push-settings')), findsOneWidget);
      expect(isOn(t, 'voice-hold-on'), isTrue);
      expect(isOn(t, 'voice-hold-off'), isFalse);
      expect(isOn(t, 'voice-delay-400'), isTrue);
      expect(isOn(t, 'voice-warm-3'), isTrue);
      await t.runAsync(voice.dispose);
    },
  );

  testWidgets('couper Espace maintenu se mémorise et se voit', (t) async {
    final voice = await pumpSettings(t);
    await t.tap(find.byKey(k('voice-hold-off')));
    await t.pump();
    expect(voice.services.preferences.holdSpaceEnabled, isFalse);
    expect(isOn(t, 'voice-hold-off'), isTrue);
    expect(isOn(t, 'voice-hold-on'), isFalse);
    await t.tap(find.byKey(k('voice-hold-on')));
    await t.pump();
    expect(voice.services.preferences.holdSpaceEnabled, isTrue);
    await t.runAsync(voice.dispose);
  });

  testWidgets('le délai de maintien se choisit et se mémorise', (t) async {
    final voice = await pumpSettings(t);
    await t.tap(find.byKey(k('voice-delay-600')));
    await t.pump();
    expect(
      voice.services.preferences.holdDelay,
      const Duration(milliseconds: 600),
    );
    expect(isOn(t, 'voice-delay-600'), isTrue);
    expect(isOn(t, 'voice-delay-400'), isFalse);
    await t.runAsync(voice.dispose);
  });

  testWidgets(
    'la durée de garde se mémorise et s’applique tout de suite au reconnaisseur',
    (t) async {
      final voice = await pumpSettings(t);
      await t.tap(find.byKey(k('voice-warm-10')));
      await t.pump();
      expect(voice.services.preferences.keepWarm, const Duration(minutes: 10));
      expect(voice.warm.last, const Duration(minutes: 10));
      expect(isOn(t, 'voice-warm-10'), isTrue);
      await t.runAsync(voice.dispose);
    },
  );

  testWidgets('« Jamais » : le modèle n’est plus gardé', (t) async {
    final voice = await pumpSettings(t);
    await t.tap(find.byKey(k('voice-warm-0')));
    await t.pump();
    expect(voice.services.preferences.keepWarm, Duration.zero);
    expect(voice.warm.last, Duration.zero);
    await t.runAsync(voice.dispose);
  });
}
