import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/dictation_types.dart';

import 'ai_panel_harness.dart';
import 'composer_wiring_test.dart' show pumpComposer;
import 'voice_fakes.dart';

Finder get mic => find.byKey(const ValueKey('voice-mic-button'));

void main() {
  testWidgets('chat sans services de dictée : pas de micro', (tester) async {
    final store = await pumpPanel(tester, withDefaultConnection());
    expect(mic, findsNothing);
    await close(tester, store);
  });

  testWidgets(
    'chat avec dictée : le texte dicté arrive dans le champ de saisie',
    (tester) async {
      final voice = await FakeVoice.create();
      final store = await pumpPanel(
        tester,
        withDefaultConnection(),
        voice: voice.services,
      );
      expect(mic, findsOneWidget);
      await tester.tap(mic);
      await tester.pump();
      expect(voice.services.dictation.state, DictationState.listening);
      voice.recognizer.say(const DictationPartial('Bonjour tout le monde'));
      await tester.pump();
      expect(find.text('Bonjour tout le monde'), findsOneWidget);
      await tester.runAsync(voice.services.dictation.stop);
      for (var i = 0; i < 6; i++) {
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 10)),
        );
        await tester.pump();
      }
      expect(voice.services.dictation.state, DictationState.idle);
      await close(tester, store);
      await tester.runAsync(voice.dispose);
    },
  );

  testWidgets(
    'composeur avec dictée : le texte dicté arrive dans le corps du brouillon',
    (tester) async {
      final voice = await FakeVoice.create();
      final (store, _) = await pumpComposer(tester, voice: voice.services);
      expect(mic, findsOneWidget);
      await tester.tap(mic);
      await tester.pump();
      voice.recognizer.say(const DictationPartial('Bonjour Marie'));
      voice.recognizer.say(const DictationFinal('Bonjour Marie,'));
      await tester.pump();
      expect(store.c!.body.text, 'Bonjour Marie, ');
      expect(store.c!.body.flags.length, store.c!.body.text.length);
      await tester.runAsync(voice.services.dictation.stop);
      for (var i = 0; i < 6; i++) {
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 10)),
        );
        await tester.pump();
      }
      store.dispose();
      await tester.pumpWidget(const SizedBox());
      await tester.runAsync(voice.dispose);
    },
  );

  testWidgets('composeur sans services de dictée : pas de micro', (
    tester,
  ) async {
    final (store, _) = await pumpComposer(tester);
    expect(mic, findsNothing);
    store.dispose();
  });
}
