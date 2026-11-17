import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/voice_settings.dart';
import 'package:mira/voice/dictation_types.dart';
import 'package:mira/voice/voice_model_catalog.dart';
import 'package:mira/voice/voice_model_store.dart';
import 'package:mira/voice/voice_services.dart';

import 'voice_fakes.dart';
import 'voice_model_server.dart';

Key k(String id) => ValueKey(id);

Future<void> pumpSettings(WidgetTester tester, FakeVoice voice) async {
  tester.view.physicalSize = const Size(1200, 1600);
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
}

Future<void> waitFor(WidgetTester tester, bool Function() done) async {
  for (var i = 0; i < 100 && !done(); i++) {
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 30)),
    );
    await tester.pump();
  }
  expect(done(), isTrue);
}

void main() {
  late ModelServer server;

  setUp(() async {
    HttpOverrides.global = null;
    server = ModelServer();
    await server.start();
  });
  tearDown(() => server.stop());

  Future<FakeVoice> voiceWithServer({VoiceModelEntry? override}) =>
      FakeVoice.create(catalog: [override ?? server.entry()]);

  testWidgets('sans services de dictée, la section est vide', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: buildTheme(false),
        home: const Scaffold(body: VoiceSettings()),
      ),
    );
    expect(find.byKey(k('voice-settings')), findsNothing);
  });

  testWidgets('une carte par langue du catalogue, avec taille et état', (
    tester,
  ) async {
    final voice = await FakeVoice.create();
    await pumpSettings(tester, voice);
    expect(find.byKey(k('voice-model-fr')), findsOneWidget);
    expect(find.byKey(k('voice-model-en')), findsOneWidget);
    expect(find.textContaining('57'), findsWidgets);
    expect(find.byKey(k('voice-install-fr')), findsOneWidget);
    await tester.runAsync(voice.dispose);
  });

  testWidgets('télécharger demande confirmation et ne lance rien avant', (
    tester,
  ) async {
    final voice = await voiceWithServer();
    await pumpSettings(tester, voice);
    await tester.tap(find.byKey(k('voice-install-fr')));
    await tester.pumpAndSettle();
    expect(find.byKey(k('voice-confirm-accept')), findsOneWidget);
    expect(
      voice.services.models.statusOf(DictationLanguage.fr),
      isA<VoiceModelNotInstalled>(),
    );
    await tester.tap(find.byKey(k('voice-confirm-cancel')));
    await tester.pumpAndSettle();
    expect(
      voice.services.models.statusOf(DictationLanguage.fr),
      isA<VoiceModelNotInstalled>(),
    );
    expect(find.byKey(k('voice-install-fr')), findsOneWidget);
    await tester.runAsync(voice.dispose);
  });

  testWidgets(
    'confirmer télécharge puis affiche Installé, supprimer le retire',
    (tester) async {
      final voice = await voiceWithServer();
      await pumpSettings(tester, voice);
      await tester.tap(find.byKey(k('voice-install-fr')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(k('voice-confirm-accept')));
      await tester.pump();
      final store = voice.services.models;
      await waitFor(
        tester,
        () => store.statusOf(DictationLanguage.fr) is VoiceModelInstalled,
      );
      expect(find.byKey(k('voice-remove-fr')), findsOneWidget);
      await tester.tap(find.byKey(k('voice-remove-fr')));
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 100)),
      );
      await tester.pump();
      expect(
        store.statusOf(DictationLanguage.fr),
        isA<VoiceModelNotInstalled>(),
      );
      await tester.runAsync(voice.dispose);
    },
  );

  testWidgets(
    'un téléchargement qui échoue affiche la raison et propose de réessayer',
    (tester) async {
      final good = server.entry();
      final bad = VoiceModelEntry(
        language: DictationLanguage.fr,
        url: good.url,
        archiveBytes: good.archiveBytes,
        sha256: '0' * 64,
        folder: good.folder,
        files: good.files,
      );
      final voice = await voiceWithServer(override: bad);
      await pumpSettings(tester, voice);
      await tester.tap(find.byKey(k('voice-install-fr')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(k('voice-confirm-accept')));
      await tester.pump();
      await waitFor(
        tester,
        () =>
            voice.services.models.statusOf(DictationLanguage.fr)
                is VoiceModelFailed,
      );
      expect(find.byKey(k('voice-failed-fr')), findsOneWidget);
      expect(find.byKey(k('voice-install-fr')), findsOneWidget);
      await tester.runAsync(voice.dispose);
    },
  );

  testWidgets('changer de langue la mémorise et la donne au contrôleur', (
    tester,
  ) async {
    final voice = await FakeVoice.create();
    await pumpSettings(tester, voice);
    await tester.tap(find.byKey(k('voice-lang-en')));
    await tester.pump();
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 50)),
    );
    await tester.pump();
    expect(voice.services.dictation.language, DictationLanguage.en);
    expect(
      voice.services.preferences.language(uiLocale: 'fr'),
      DictationLanguage.en,
    );
    await tester.runAsync(voice.dispose);
  });

  testWidgets('une installation en cours se suit et s’annule', (tester) async {
    server.mode = ServeMode.hangHalfway;
    final voice = await voiceWithServer();
    await pumpSettings(tester, voice);
    await tester.tap(find.byKey(k('voice-install-fr')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(k('voice-confirm-accept')));
    await tester.pump();
    await waitFor(tester, () => server.halfway.isCompleted);
    await waitFor(
      tester,
      () =>
          voice.services.models.statusOf(DictationLanguage.fr)
              is VoiceModelDownloading,
    );
    expect(find.byKey(k('voice-cancel-fr')), findsOneWidget);
    await tester.tap(find.byKey(k('voice-cancel-fr')));
    await waitFor(
      tester,
      () =>
          voice.services.models.statusOf(DictationLanguage.fr)
              is VoiceModelNotInstalled,
    );
    await tester.runAsync(voice.dispose);
  });
}
