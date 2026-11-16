import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/mic_button.dart';
import 'package:mira/voice/audio_level.dart';
import 'package:mira/voice/dictation_controller.dart';
import 'package:mira/voice/voice_services.dart';

import 'voice_fakes.dart';

Float32List tone(double amplitude, [int n = 1600]) => Float32List.fromList([
  for (var i = 0; i < n; i++) amplitude * math.sin(i * 0.3),
]);

Future<void> pause([int ms = 0]) =>
    Future<void>.delayed(Duration(milliseconds: ms));

final bars = find.byKey(const ValueKey('voice-mic-bars'));

List<double> heights(WidgetTester t) => [
  for (final e
      in find
          .descendant(
            of: bars,
            matching: find.byKey(const ValueKey('voice-mic-bar')),
          )
          .evaluate())
    t.getSize(find.byWidget(e.widget)).height,
];

void main() {
  group('audioLevel', () {
    test('le silence vaut 0', () {
      expect(audioLevel(Float32List(1600)), 0);
      expect(audioLevel(Float32List(0)), 0);
    });

    test('une voix forte approche 1, une voix douce reste au milieu', () {
      expect(audioLevel(tone(0.8)), greaterThan(0.85));
      final soft = audioLevel(tone(0.03));
      expect(soft, inExclusiveRange(0.2, 0.7));
    });

    test('plus fort donne un niveau plus haut', () {
      final levels = [0.005, 0.02, 0.1, 0.5].map((a) => audioLevel(tone(a)));
      expect(levels.toList(), orderedEquals(levels.toList()..sort()));
      expect(levels.toSet().length, 4);
    });

    test('toujours entre 0 et 1', () {
      expect(audioLevel(tone(5)), lessThanOrEqualTo(1));
      expect(audioLevel(tone(1e-9)), greaterThanOrEqualTo(0));
    });
  });

  group('niveau du contrôleur', () {
    late FakeAudioSource audio;
    late FakeRecognizer recognizer;
    late DictationController controller;
    setUp(() {
      audio = FakeAudioSource();
      recognizer = FakeRecognizer();
      controller = DictationController(
        audio: audio,
        recognizer: recognizer,
        models: FakeModels(),
      );
    });

    test(
      'monte avec la voix, retombe doucement au silence, revient à 0 à l’arrêt',
      () async {
        await controller.start(TextEditingController());
        audio.out.add(tone(0.5));
        await pause();
        final loud = controller.level.value;
        expect(loud, greaterThan(0.7));
        audio.out.add(Float32List(1600));
        await pause();
        expect(controller.level.value, lessThan(loud));
        expect(controller.level.value, greaterThan(0));
        await controller.stop();
        expect(controller.level.value, 0);
      },
    );

    test(
      'le niveau suit le son même pendant le chargement du modèle',
      () async {
        recognizer.startDelay = const Duration(milliseconds: 150);
        final starting = controller.start(TextEditingController());
        await pause(30);
        expect(controller.recognizerReady, isFalse);
        audio.out.add(tone(0.5));
        await pause();
        expect(controller.level.value, greaterThan(0.7));
        await starting;
        expect(controller.recognizerReady, isTrue);
      },
    );
  });

  group('barres du bouton micro', () {
    Widget host(Widget child, FakeVoice voice) => VoiceScope(
      services: voice.services,
      child: MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(body: Center(child: child)),
      ),
    );

    testWidgets('pas de barres au repos, cinq barres pendant l’écoute', (
      t,
    ) async {
      final voice = await FakeVoice.create();
      await t.pumpWidget(
        host(MicButton(target: TextEditingController()), voice),
      );
      expect(bars, findsNothing);
      await t.tap(find.byKey(const ValueKey('voice-mic-button')));
      await t.pump();
      expect(bars, findsOneWidget);
      expect(heights(t), hasLength(5));
      await t.tap(find.byKey(const ValueKey('voice-mic-button')));
      for (var i = 0; i < 6; i++) {
        await t.runAsync(() => pause(10));
        await t.pump();
      }
      expect(bars, findsNothing);
      await t.runAsync(voice.dispose);
    });

    testWidgets('les barres montent avec la voix', (t) async {
      final voice = await FakeVoice.create();
      await t.pumpWidget(
        host(MicButton(target: TextEditingController()), voice),
      );
      await t.tap(find.byKey(const ValueKey('voice-mic-button')));
      await t.pump();
      await t.pump(const Duration(milliseconds: 300));
      final quiet = heights(t).reduce(math.max);
      for (var i = 0; i < 5; i++) {
        voice.audio.out.add(tone(0.5));
        await t.runAsync(() => pause());
        await t.pump(const Duration(milliseconds: 200));
      }
      final loud = heights(t).reduce(math.max);
      expect(loud, greaterThan(quiet + 4));
      await t.runAsync(voice.dispose);
    });
  });
}
