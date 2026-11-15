import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/step_animator.dart';
import 'package:mira/voice/dictation_types.dart';

import 'ai_panel_harness.dart';
import 'app_test.dart' show pumpApp, teardown;
import 'composer_wiring_test.dart' show pumpComposer;
import 'voice_fakes.dart';

Finder get mic => find.byKey(const ValueKey('voice-mic-button'));

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

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

  testWidgets('modèle absent : le micro du chat propose le téléchargement', (
    tester,
  ) async {
    final voice = await FakeVoice.create(installed: {});
    final store = await pumpPanel(
      tester,
      withDefaultConnection(),
      voice: voice.services,
    );
    await tester.tap(mic);
    for (var i = 0; i < 6; i++) {
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 10)),
      );
      await tester.pump();
    }
    expect(find.byKey(const ValueKey('voice-confirm')), findsOneWidget);
    await close(tester, store);
    await tester.runAsync(voice.dispose);
  });

  testWidgets('réglages : la section de dictée est dans l’onglet Messagerie', (
    tester,
  ) async {
    final voice = await FakeVoice.create();
    final store = await pumpApp(tester, voice: voice.services);
    await tester.tap(find.text('PARAMÈTRES').first);
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byKey(const ValueKey('voice-settings')), findsNothing);
    await tester.tap(find.text('MESSAGERIE'));
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byKey(const ValueKey('voice-settings')), findsOneWidget);
    await teardown(tester, store);
    await tester.runAsync(voice.dispose);
  });

  testWidgets('réglages sans dictée : aucune section', (tester) async {
    final store = await pumpApp(tester);
    await tester.tap(find.text('PARAMÈTRES').first);
    await tester.pump(const Duration(milliseconds: 100));
    await tester.tap(find.text('MESSAGERIE'));
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byKey(const ValueKey('voice-settings')), findsNothing);
    await teardown(tester, store);
  });

  testWidgets(
    'composeur : un brouillon dicté est marqué modifié et enregistré à la fermeture',
    (tester) async {
      final voice = await FakeVoice.create();
      final (store, _) = await pumpComposer(tester, voice: voice.services);
      int drafts() => store.mails.where((m) => m.folder == 'drafts').length;
      final before = drafts();
      await tester.tap(mic);
      await tester.pump();
      voice.recognizer.say(
        const DictationFinal('Bonjour Marie, je confirme le rendez-vous.'),
      );
      await tester.pump();
      expect(store.c!.dirty, isTrue);
      await tester.runAsync(voice.services.dictation.stop);
      for (var i = 0; i < 6; i++) {
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 10)),
        );
        await tester.pump();
      }
      store.closeCompose();
      await tester.pump();
      expect(drafts(), before + 1);
      await tester.pump(const Duration(seconds: 5));
      store.dispose();
      await tester.pumpWidget(const SizedBox());
      await tester.runAsync(voice.dispose);
    },
  );

  testWidgets('chat : envoyer le message arrête la dictée', (tester) async {
    final voice = await FakeVoice.create();
    final store = await pumpPanel(
      tester,
      withDefaultConnection(),
      voice: voice.services,
    );
    await tester.tap(mic);
    await tester.pump();
    voice.recognizer.say(const DictationFinal('Résume mon dernier mail.'));
    await tester.pump();
    await tester.tap(find.byKey(const ValueKey('chat-send')));
    for (var i = 0; i < 6; i++) {
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 10)),
      );
      await tester.pump();
    }
    expect(voice.services.dictation.state, DictationState.idle);
    expect(voice.audio.isOpen, isFalse);
    await close(tester, store);
    await tester.runAsync(voice.dispose);
  });
}
