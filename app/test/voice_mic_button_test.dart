import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/voice/dictation_types.dart';
import 'package:mira/voice/voice_services.dart';
import 'package:mira/ui/mic_button.dart';

import 'voice_fakes.dart';

Widget host(Widget child, {FakeVoice? voice}) {
  final page = MaterialApp(
    theme: buildTheme(false),
    home: Scaffold(body: Center(child: child)),
  );
  return voice == null
      ? page
      : VoiceScope(services: voice.services, child: page);
}

/// Laisse s'écouler du vrai temps : l'arrêt d'une dictée passe par des futurs que le temps simulé n'avance pas.
Future<void> settle(WidgetTester tester) async {
  for (var i = 0; i < 6; i++) {
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 10)),
    );
    await tester.pump();
  }
}

Future<void> finish(WidgetTester tester, FakeVoice voice) async {
  await tester.runAsync(voice.dispose);
}

Finder get mic => find.byKey(const ValueKey('voice-mic-button'));
Finder get listening => find.byKey(const ValueKey('voice-mic-listening'));

void main() {
  testWidgets('sans services de dictée, aucun bouton', (tester) async {
    await tester.pumpWidget(host(MicButton(target: TextEditingController())));
    expect(mic, findsNothing);
  });

  testWidgets('toucher démarre la dictée sur le champ, retoucher l’arrête', (
    tester,
  ) async {
    final voice = await FakeVoice.create();
    final field = TextEditingController();
    await tester.pumpWidget(host(MicButton(target: field), voice: voice));
    expect(listening, findsNothing);
    await tester.tap(mic);
    await tester.pump();
    expect(voice.services.dictation.state, DictationState.listening);
    expect(voice.services.dictation.target, same(field));
    expect(listening, findsOneWidget);
    await tester.tap(mic);
    await settle(tester);
    expect(voice.services.dictation.state, DictationState.idle);
    expect(listening, findsNothing);
    await finish(tester, voice);
  });

  testWidgets('le bouton d’un autre champ ne montre pas l’écoute', (
    tester,
  ) async {
    final voice = await FakeVoice.create();
    final a = TextEditingController();
    final b = TextEditingController();
    await tester.pumpWidget(
      host(
        Column(
          children: [
            MicButton(key: const ValueKey('a'), target: a),
            MicButton(key: const ValueKey('b'), target: b),
          ],
        ),
        voice: voice,
      ),
    );
    await tester.tap(
      find.descendant(of: find.byKey(const ValueKey('a')), matching: mic),
    );
    await tester.pump();
    expect(
      find.descendant(of: find.byKey(const ValueKey('a')), matching: listening),
      findsOneWidget,
    );
    expect(
      find.descendant(of: find.byKey(const ValueKey('b')), matching: listening),
      findsNothing,
    );
    await finish(tester, voice);
  });

  testWidgets('retirer le bouton pendant l’écoute libère le champ', (
    tester,
  ) async {
    final voice = await FakeVoice.create();
    final field = TextEditingController();
    await tester.pumpWidget(host(MicButton(target: field), voice: voice));
    await tester.tap(mic);
    await tester.pump();
    voice.recognizer.say(const DictationPartial('un'));
    expect(field.text, 'un');
    await tester.pumpWidget(host(const SizedBox(), voice: voice));
    voice.recognizer.say(const DictationPartial('deux'));
    expect(field.text, 'un');
    await settle(tester);
    expect(voice.services.dictation.state, DictationState.idle);
    await finish(tester, voice);
  });

  testWidgets('micro refusé : le problème est signalé', (tester) async {
    final voice = await FakeVoice.create();
    voice.audio.permission = false;
    DictationProblem? reported;
    await tester.pumpWidget(
      host(
        MicButton(
          target: TextEditingController(),
          onProblem: (p) => reported = p,
        ),
        voice: voice,
      ),
    );
    await tester.tap(mic);
    await settle(tester);
    expect(reported, DictationProblem.microphoneDenied);
    expect(listening, findsNothing);
    await finish(tester, voice);
  });

  testWidgets('modèle absent : le problème est signalé', (tester) async {
    final voice = await FakeVoice.create(installed: {});
    DictationProblem? reported;
    await tester.pumpWidget(
      host(
        MicButton(
          target: TextEditingController(),
          onProblem: (p) => reported = p,
        ),
        voice: voice,
      ),
    );
    await tester.tap(mic);
    await settle(tester);
    expect(reported, DictationProblem.modelMissing);
    await finish(tester, voice);
  });
}
