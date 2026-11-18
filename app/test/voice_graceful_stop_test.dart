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
  Rig() {
    controller = DictationController(
      audio: audio,
      recognizer: recognizer,
      models: FakeModels(),
    );
  }
}

Float32List chunk(double v) => Float32List.fromList([v, v]);

Future<void> pause([int ms = 0]) =>
    Future<void>.delayed(Duration(milliseconds: ms));

void main() {
  test(
    'arrêter pendant le chargement livre quand même ce qui a été dit',
    () async {
      final r = Rig();
      r.recognizer.startDelay = load;
      r.recognizer.textOnStop = 'Bonjour Marie.';
      final field = TextEditingController();
      final starting = r.controller.start(field);
      await pause(50);
      r.audio.out.add(chunk(0.1));
      await pause();
      final stopping = r.controller.stop();
      await Future.wait([starting, stopping]);
      expect(r.recognizer.fed, hasLength(1));
      expect(r.recognizer.stopCount, 1);
      expect(field.text, 'Bonjour Marie. ');
      expect(r.controller.state, DictationState.idle);
      expect(r.audio.isOpen, isFalse);
    },
  );

  test('le micro est coupé dès l’arrêt, pas à la fin du chargement', () async {
    final r = Rig();
    r.recognizer.startDelay = load;
    final starting = r.controller.start(TextEditingController());
    await pause(50);
    final stopping = r.controller.stop();
    await pause(20);
    expect(r.audio.isOpen, isFalse);
    await Future.wait([starting, stopping]);
  });

  test(
    'fermer le champ pendant le chargement n’écrit rien et ne nourrit rien',
    () async {
      final r = Rig();
      r.recognizer.startDelay = load;
      r.recognizer.textOnStop = 'ne doit pas apparaître';
      final field = TextEditingController();
      final starting = r.controller.start(field);
      await pause(50);
      r.audio.out.add(chunk(0.1));
      await pause();
      r.controller.release(field);
      await starting;
      await pause(300);
      expect(field.text, isEmpty);
      expect(r.recognizer.fed, isEmpty);
      expect(r.controller.state, DictationState.idle);
      expect(r.audio.isOpen, isFalse);
    },
  );

  test(
    'arrêter avant l’ouverture du micro : le micro ne s’ouvre pas',
    () async {
      final r = Rig();
      r.audio.permissionDelay = const Duration(milliseconds: 100);
      final starting = r.controller.start(TextEditingController());
      await pause(20);
      final stopping = r.controller.stop();
      await Future.wait([starting, stopping]);
      expect(r.audio.isOpen, isFalse);
      expect(r.audio.startCount, 0);
      expect(r.controller.state, DictationState.idle);
    },
  );

  test(
    'un échec du chargement après l’arrêt est signalé, micro fermé',
    () async {
      final r = Rig();
      r.recognizer.startDelay = load;
      r.recognizer.startError = const DictationException(
        DictationProblem.recognizerFailed,
      );
      final starting = r.controller.start(TextEditingController());
      await pause(50);
      final stopping = r.controller.stop();
      await Future.wait([starting, stopping]);
      expect(r.controller.problem, DictationProblem.recognizerFailed);
      expect(r.controller.state, DictationState.idle);
      expect(r.audio.isOpen, isFalse);
    },
  );

  test(
    'après un arrêt pendant le chargement, une nouvelle dictée démarre',
    () async {
      final r = Rig();
      r.recognizer.startDelay = load;
      final first = r.controller.start(TextEditingController());
      await pause(50);
      await Future.wait([first, r.controller.stop()]);
      await r.controller.start(TextEditingController());
      expect(r.controller.state, DictationState.listening);
      expect(r.audio.startCount, 2);
    },
  );
}
