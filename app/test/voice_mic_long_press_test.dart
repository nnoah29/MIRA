import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/mic_button.dart';
import 'package:mira/voice/dictation_types.dart';
import 'package:mira/voice/voice_services.dart';

import 'voice_fakes.dart';

Widget host(Widget child, FakeVoice voice) => VoiceScope(
  services: voice.services,
  child: MaterialApp(
    theme: buildTheme(false),
    home: Scaffold(body: Center(child: child)),
  ),
);

Future<void> settle(WidgetTester t) async {
  for (var i = 0; i < 8; i++) {
    await t.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 10)),
    );
    await t.pump();
  }
}

final mic = find.byKey(const ValueKey('voice-mic-button'));

void main() {
  testWidgets(
    'appui long : dictée phrase par phrase tant que le doigt reste posé',
    (t) async {
      final voice = await FakeVoice.create();
      final field = TextEditingController();
      await t.pumpWidget(host(MicButton(target: field), voice));
      final gesture = await t.startGesture(t.getCenter(mic));
      await t.pump(const Duration(milliseconds: 700));
      final d = voice.services.dictation;
      expect(d.state, isNot(DictationState.idle));
      expect(d.mode, DictationMode.pushToTalk);
      expect(d.target, same(field));
      await gesture.up();
      await settle(t);
      expect(d.state, DictationState.idle);
      await t.runAsync(voice.dispose);
    },
  );

  testWidgets('relâcher valide le texte dicté', (t) async {
    final voice = await FakeVoice.create();
    voice.recognizer.textOnStop = 'Bonjour.';
    final field = TextEditingController();
    await t.pumpWidget(host(MicButton(target: field), voice));
    final gesture = await t.startGesture(t.getCenter(mic));
    await t.pump(const Duration(milliseconds: 700));
    await settle(t);
    await gesture.up();
    await settle(t);
    expect(field.text, 'Bonjour. ');
    await t.runAsync(voice.dispose);
  });

  testWidgets('un appui court reste une dictée continue', (t) async {
    final voice = await FakeVoice.create();
    await t.pumpWidget(host(MicButton(target: TextEditingController()), voice));
    await t.tap(mic);
    await t.pump();
    final d = voice.services.dictation;
    expect(d.state, DictationState.listening);
    expect(d.mode, DictationMode.continuous);
    await t.tap(mic);
    await settle(t);
    await t.runAsync(voice.dispose);
  });

  testWidgets('appui long sans modèle : le problème est signalé', (t) async {
    final voice = await FakeVoice.create(installed: {});
    final problems = <DictationProblem>[];
    await t.pumpWidget(
      host(
        MicButton(target: TextEditingController(), onProblem: problems.add),
        voice,
      ),
    );
    final gesture = await t.startGesture(t.getCenter(mic));
    await t.pump(const Duration(milliseconds: 700));
    await gesture.up();
    await settle(t);
    expect(problems, [DictationProblem.modelMissing]);
    await t.runAsync(voice.dispose);
  });

  testWidgets('appui long pendant une dictée continue : rien ne change', (
    t,
  ) async {
    final voice = await FakeVoice.create();
    final field = TextEditingController();
    await t.pumpWidget(host(MicButton(target: field), voice));
    await t.tap(mic);
    await t.pump();
    final gesture = await t.startGesture(t.getCenter(mic));
    await t.pump(const Duration(milliseconds: 700));
    await gesture.up();
    await settle(t);
    expect(voice.services.dictation.mode, DictationMode.continuous);
    expect(voice.services.dictation.state, DictationState.listening);
    await t.runAsync(voice.dispose);
  });
}
