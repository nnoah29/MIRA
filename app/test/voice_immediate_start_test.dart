import 'dart:typed_data';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/dictation_controller.dart';
import 'package:mira/voice/dictation_types.dart';

import 'voice_fakes.dart';

const load = Duration(milliseconds: 200);

class Rig {
  final audio = FakeAudioSource();
  final recognizer = FakeRecognizer();
  late final DictationController controller;
  Rig({Duration startTimeout = const Duration(seconds: 20)}) {
    controller = DictationController(
      audio: audio,
      recognizer: recognizer,
      models: FakeModels(),
      recognizerStartTimeout: startTimeout,
    );
  }
}

Float32List chunk(double v) => Float32List.fromList([v, v]);

Future<void> pause(Duration d) => Future<void>.delayed(d);

void main() {
  retryTests();
  test(
    'le micro écoute dès le clic, pendant le chargement du modèle',
    () async {
      final r = Rig();
      r.recognizer.startDelay = load;
      final starting = r.controller.start(TextEditingController());
      await pause(const Duration(milliseconds: 50));
      expect(r.audio.startCount, 1);
      expect(r.controller.state, DictationState.listening);
      expect(r.recognizer.startCount, 0);
      await starting;
      expect(r.recognizer.startCount, 1);
    },
  );

  test(
    'les premiers mots dits pendant le chargement ne sont pas perdus',
    () async {
      final r = Rig();
      r.recognizer.startDelay = load;
      final starting = r.controller.start(TextEditingController());
      await pause(const Duration(milliseconds: 50));
      r.audio.out.add(chunk(0.1));
      r.audio.out.add(chunk(0.2));
      await starting;
      r.audio.out.add(chunk(0.3));
      await pause(Duration.zero);
      expect(r.recognizer.fed.map((c) => c.first).toList(), [
        closeTo(0.1, 1e-6),
        closeTo(0.2, 1e-6),
        closeTo(0.3, 1e-6),
      ]);
    },
  );

  test(
    'un échec du chargement ferme le micro et signale le problème',
    () async {
      final r = Rig();
      r.recognizer.startDelay = load;
      r.recognizer.startError = const DictationException(
        DictationProblem.recognizerFailed,
      );
      await r.controller.start(TextEditingController());
      expect(r.controller.problem, DictationProblem.recognizerFailed);
      expect(r.controller.state, DictationState.idle);
      expect(r.audio.isOpen, isFalse);
    },
  );

  test('un chargement qui ne finit jamais est abandonné au délai', () async {
    final r = Rig(startTimeout: const Duration(milliseconds: 100));
    r.recognizer.startDelay = const Duration(seconds: 5);
    await r.controller.start(TextEditingController());
    expect(r.controller.problem, DictationProblem.recognizerFailed);
    expect(r.controller.state, DictationState.idle);
    expect(r.audio.isOpen, isFalse);
  });

  test(
    'arrêter pendant le chargement ne nourrit plus le reconnaisseur',
    () async {
      final r = Rig();
      r.recognizer.startDelay = load;
      final starting = r.controller.start(TextEditingController());
      await pause(const Duration(milliseconds: 50));
      r.audio.out.add(chunk(0.1));
      final stopping = r.controller.stop();
      await Future.wait([starting, stopping]);
      expect(r.controller.state, DictationState.idle);
      expect(r.audio.isOpen, isFalse);
      expect(r.recognizer.fed, isEmpty);
    },
  );

  test('la dictée reste utilisable au démarrage suivant', () async {
    final r = Rig();
    r.recognizer.startDelay = load;
    await r.controller.start(TextEditingController());
    await r.controller.stop();
    await r.controller.start(TextEditingController());
    expect(r.controller.state, DictationState.listening);
    expect(r.audio.startCount, 2);
  });
}

void retryTests() {
  test(
    'le modèle installé après coup : retry relance la dictée sur le même champ',
    () async {
      final models = FakeModels({});
      final controller = DictationController(
        audio: FakeAudioSource(),
        recognizer: FakeRecognizer(),
        models: models,
      );
      final field = TextEditingController();
      await controller.start(field);
      expect(controller.problem, DictationProblem.modelMissing);
      models.installed.add(DictationLanguage.fr);
      await controller.retry();
      expect(controller.state, DictationState.listening);
      expect(controller.target, same(field));
    },
  );

  test('retry ne fait rien si le champ a été fermé entre-temps', () async {
    final models = FakeModels({});
    final controller = DictationController(
      audio: FakeAudioSource(),
      recognizer: FakeRecognizer(),
      models: models,
    );
    final field = TextEditingController();
    await controller.start(field);
    controller.release(field);
    models.installed.add(DictationLanguage.fr);
    await controller.retry();
    expect(controller.state, DictationState.idle);
  });

  test('retry ne relance pas une dictée terminée normalement', () async {
    final controller = DictationController(
      audio: FakeAudioSource(),
      recognizer: FakeRecognizer(),
      models: FakeModels(),
    );
    await controller.start(TextEditingController());
    await controller.stop();
    await controller.retry();
    expect(controller.state, DictationState.idle);
  });

  test('retry sert une seule fois', () async {
    final models = FakeModels({});
    final controller = DictationController(
      audio: FakeAudioSource(),
      recognizer: FakeRecognizer(),
      models: models,
    );
    await controller.start(TextEditingController());
    models.installed.add(DictationLanguage.fr);
    await controller.retry();
    await controller.stop();
    await controller.retry();
    expect(controller.state, DictationState.idle);
  });
}
