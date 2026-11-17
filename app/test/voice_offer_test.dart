import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/voice_model_dialog.dart';
import 'package:mira/ui/widgets.dart';
import 'package:mira/voice/dictation_types.dart';
import 'package:mira/voice/voice_model_catalog.dart';
import 'package:mira/voice/voice_model_store.dart';
import 'package:mira/voice/voice_services.dart';

import 'voice_fakes.dart';
import 'voice_model_server.dart';

Future<void> waitFor(WidgetTester tester, bool Function() done) async {
  for (var i = 0; i < 100 && !done(); i++) {
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 30)),
    );
    await tester.pump();
  }
  expect(done(), isTrue);
}

Future<void> pumpOffer(
  WidgetTester tester,
  FakeVoice voice,
  List<String> notes,
) async {
  await tester.pumpWidget(
    VoiceScope(
      services: voice.services,
      child: MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(
          body: Builder(
            builder: (context) => TextButton(
              key: const ValueKey('offer'),
              onPressed: () => offerVoiceModelInstall(
                context,
                voice.services,
                notify: (m, {bool err = false}) =>
                    notes.add(err ? 'ERR $m' : m),
              ),
              child: const Text('offre'),
            ),
          ),
        ),
      ),
    ),
  );
}

void main() {
  late ModelServer server;
  setUp(() async {
    HttpOverrides.global = null;
    server = ModelServer();
    await server.start();
  });
  tearDown(() => server.stop());

  testWidgets('confirmer annonce le lancement puis la fin', (tester) async {
    final voice = await FakeVoice.create(catalog: [server.entry()]);
    final notes = <String>[];
    await pumpOffer(tester, voice, notes);
    await tester.tap(find.byKey(const ValueKey('offer')));
    await tester.pump(const Duration(milliseconds: 500));
    tester
        .widget<Btn>(find.byKey(const ValueKey('voice-confirm-accept')))
        .onTap!();
    await tester.pump();
    await waitFor(
      tester,
      () =>
          voice.services.models.statusOf(DictationLanguage.fr)
              is VoiceModelInstalled,
    );
    await waitFor(tester, () => notes.length >= 2);
    expect(notes.first, contains('lancé'));
    expect(notes.last, contains('prêt'));
    await tester.runAsync(voice.dispose);
  });

  testWidgets('un échec de téléchargement est annoncé comme une erreur', (
    tester,
  ) async {
    final good = server.entry();
    final bad = VoiceModelEntry(
      language: DictationLanguage.fr,
      url: good.url,
      archiveBytes: good.archiveBytes,
      sha256: '0' * 64,
      folder: good.folder,
      files: good.files,
    );
    final voice = await FakeVoice.create(catalog: [bad]);
    final notes = <String>[];
    await pumpOffer(tester, voice, notes);
    await tester.tap(find.byKey(const ValueKey('offer')));
    await tester.pump(const Duration(milliseconds: 500));
    tester
        .widget<Btn>(find.byKey(const ValueKey('voice-confirm-accept')))
        .onTap!();
    await tester.pump();
    await waitFor(tester, () => notes.length >= 2);
    expect(notes.last, startsWith('ERR'));
    await tester.runAsync(voice.dispose);
  });

  testWidgets(
    'pendant un téléchargement, on montre la progression au lieu de reproposer',
    (tester) async {
      server.mode = ServeMode.hangHalfway;
      final voice = await FakeVoice.create(catalog: [server.entry()]);
      final notes = <String>[];
      await pumpOffer(tester, voice, notes);
      voice.services.models.install(DictationLanguage.fr);
      await waitFor(
        tester,
        () =>
            voice.services.models.statusOf(DictationLanguage.fr)
                is VoiceModelDownloading,
      );
      await tester.tap(find.byKey(const ValueKey('offer')));
      await tester.pump(const Duration(milliseconds: 500));
      expect(find.byKey(const ValueKey('voice-confirm')), findsNothing);
      expect(notes.single, contains('%'));
      voice.services.models.cancel(DictationLanguage.fr);
      await waitFor(
        tester,
        () =>
            voice.services.models.statusOf(DictationLanguage.fr)
                is VoiceModelNotInstalled,
      );
      await tester.runAsync(voice.dispose);
    },
  );
}
