import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/dictation_controller.dart';
import 'package:mira/voice/dictation_types.dart';

import 'voice_fakes.dart';

class Rig {
  final audio = FakeAudioSource();
  final recognizer = FakeRecognizer();
  final models = FakeModels();
  late final DictationController controller;
  Rig({Duration maxDuration = const Duration(minutes: 5)}) {
    controller = DictationController(
      audio: audio,
      recognizer: recognizer,
      models: models,
      maxDuration: maxDuration,
    );
  }
}

TextEditingController field([String text = '']) {
  final c = TextEditingController(text: text);
  c.selection = TextSelection.collapsed(offset: text.length);
  return c;
}

void main() {
  test(
    'start passe par preparing puis listening ; les partiels écrivent',
    () async {
      final r = Rig();
      final seen = <DictationState>[];
      r.controller.addListener(() => seen.add(r.controller.state));
      final f = field();
      await r.controller.start(f);
      expect(seen.first, DictationState.preparing);
      expect(r.controller.state, DictationState.listening);
      expect(r.controller.target, same(f));
      r.recognizer.say(const DictationPartial('Bonjour Marie'));
      expect(f.text, 'Bonjour Marie');
      expect(r.recognizer.startedWith, DictationLanguage.fr);
    },
  );

  test('les échantillons du micro sont transmis au reconnaisseur', () async {
    final r = Rig();
    await r.controller.start(field());
    r.audio.out.add(Float32List.fromList([0.1, 0.2]));
    await Future<void>.delayed(Duration.zero);
    expect(r.recognizer.fed.length, 1);
  });

  test('stop valide le dernier texte et revient à idle', () async {
    final r = Rig();
    final f = field();
    r.recognizer.textOnStop = 'Bonjour Marie.';
    await r.controller.start(f);
    r.recognizer.say(const DictationPartial('Bonjour'));
    await r.controller.stop();
    expect(f.text, 'Bonjour Marie. ');
    expect(r.controller.state, DictationState.idle);
    expect(r.controller.target, isNull);
    expect(r.audio.stopCount, 1);
    expect(r.recognizer.stopCount, 1);
  });

  test('micro refusé : problème, idle, champ intact', () async {
    final r = Rig();
    r.audio.permission = false;
    final f = field('A');
    await r.controller.start(f);
    expect(r.controller.problem, DictationProblem.microphoneDenied);
    expect(r.controller.state, DictationState.idle);
    expect(f.text, 'A');
    expect(r.audio.startCount, 0);
  });

  test('micro indisponible', () async {
    final r = Rig();
    r.audio.startError = StateError('pas de micro');
    await r.controller.start(field());
    expect(r.controller.problem, DictationProblem.microphoneUnavailable);
    expect(r.controller.state, DictationState.idle);
    expect(r.recognizer.stopCount, 1);
  });

  test('modèle absent : le micro n’est pas ouvert', () async {
    final r = Rig();
    r.models.installed.remove(DictationLanguage.fr);
    await r.controller.start(field());
    expect(r.controller.problem, DictationProblem.modelMissing);
    expect(r.audio.permissionAsked, 0);
    expect(r.audio.startCount, 0);
  });

  test('un échec du reconnaisseur au démarrage est signalé', () async {
    final r = Rig();
    r.recognizer.startError = const DictationException(
      DictationProblem.recognizerFailed,
    );
    await r.controller.start(field());
    expect(r.controller.problem, DictationProblem.recognizerFailed);
    expect(r.audio.isOpen, isFalse);
    expect(r.audio.stopCount, 1);
  });

  test('une seconde cible arrête la première', () async {
    final r = Rig();
    final a = field();
    final b = field();
    await r.controller.start(a);
    r.recognizer.say(const DictationPartial('un'));
    await r.controller.start(b);
    expect(r.controller.target, same(b));
    r.recognizer.say(const DictationPartial('deux'));
    expect(a.text, 'un');
    expect(b.text, 'deux');
  });

  test('toucher deux fois la même cible ne relance rien', () async {
    final r = Rig();
    final a = field();
    await r.controller.start(a);
    await r.controller.start(a);
    expect(r.recognizer.startCount, 1);
  });

  test('release(cible) arrête sans écrire', () async {
    final r = Rig();
    final f = field();
    await r.controller.start(f);
    r.recognizer.say(const DictationPartial('un'));
    r.controller.release(f);
    r.recognizer.say(const DictationPartial('deux'));
    expect(f.text, 'un');
    await Future<void>.delayed(Duration.zero);
    expect(r.controller.state, DictationState.idle);
  });

  test('release d’une autre cible ne change rien', () async {
    final r = Rig();
    await r.controller.start(field());
    r.controller.release(field());
    expect(r.controller.state, DictationState.listening);
  });

  test('changer de langue pendant l’écoute arrête la dictée', () async {
    final r = Rig();
    await r.controller.start(field());
    await r.controller.setLanguage(DictationLanguage.en);
    expect(r.controller.state, DictationState.idle);
    expect(r.controller.language, DictationLanguage.en);
    expect(r.audio.stopCount, 1);
  });

  test('erreur du reconnaisseur en cours de route', () async {
    final r = Rig();
    await r.controller.start(field());
    r.recognizer.fail();
    await Future<void>.delayed(Duration.zero);
    expect(r.controller.problem, DictationProblem.recognizerFailed);
    expect(r.controller.state, DictationState.idle);
    expect(r.audio.stopCount, 1);
  });

  test('durée maximale : retour à idle quand elle est atteinte', () async {
    final r = Rig(maxDuration: const Duration(milliseconds: 50));
    await r.controller.start(field());
    expect(r.controller.state, DictationState.listening);
    await Future<void>.delayed(const Duration(milliseconds: 300));
    expect(r.controller.state, DictationState.idle);
    expect(r.audio.stopCount, 1);
    expect(r.recognizer.stopCount, 1);
  });

  test('une nouvelle dictée efface le problème précédent', () async {
    final r = Rig();
    r.audio.permission = false;
    await r.controller.start(field());
    r.audio.permission = true;
    await r.controller.start(field());
    expect(r.controller.problem, isNull);
    expect(r.controller.state, DictationState.listening);
  });

  test('stop pendant la préparation : rien ne reste ouvert', () async {
    final r = Rig();
    r.recognizer.startDelay = const Duration(milliseconds: 100);
    unawaited(r.controller.start(field()));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    expect(r.controller.state, DictationState.listening);
    await r.controller.stop();
    await Future<void>.delayed(const Duration(milliseconds: 300));
    expect(r.controller.state, DictationState.idle);
    expect(r.controller.target, isNull);
    expect(r.audio.isOpen, isFalse);
  });

  test(
    'champ fermé pendant la préparation : aucune écriture, micro fermé',
    () async {
      final r = Rig();
      r.recognizer.startDelay = const Duration(milliseconds: 100);
      final f = field();
      unawaited(r.controller.start(f));
      await Future<void>.delayed(const Duration(milliseconds: 20));
      r.controller.release(f);
      await Future<void>.delayed(const Duration(milliseconds: 300));
      r.recognizer.say(const DictationPartial('fantôme'));
      expect(f.text, '');
      expect(r.controller.state, DictationState.idle);
      expect(r.audio.isOpen, isFalse);
    },
  );

  test(
    'démarrer ailleurs pendant l’arrêt : la nouvelle dictée reste ouverte',
    () async {
      final r = Rig();
      r.recognizer.stopDelay = const Duration(milliseconds: 100);
      final a = field();
      final b = field();
      await r.controller.start(a);
      unawaited(r.controller.stop());
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(r.controller.state, DictationState.stopping);
      await r.controller.start(b);
      expect(r.controller.state, DictationState.listening);
      expect(identical(r.controller.target, b), isTrue);
      expect(r.audio.isOpen, isTrue);
      r.recognizer.say(const DictationPartial('bonjour'));
      expect(b.text, 'bonjour');
      await r.controller.stop();
      expect(r.audio.isOpen, isFalse);
    },
  );

  test(
    'une exception de la demande d’autorisation ne bloque pas l’état',
    () async {
      final r = Rig();
      r.audio.permissionError = StateError('plugin');
      await r.controller.start(field());
      expect(r.controller.problem, DictationProblem.microphoneUnavailable);
      expect(r.controller.state, DictationState.idle);
    },
  );

  test('une erreur inattendue du reconnaisseur ne bloque pas l’état', () async {
    final r = Rig();
    r.recognizer.startOtherError = StateError('natif');
    await r.controller.start(field());
    expect(r.controller.problem, DictationProblem.recognizerFailed);
    expect(r.controller.state, DictationState.idle);
    expect(r.audio.isOpen, isFalse);
  });
}
