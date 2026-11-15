//! Pilote une dictée à la fois pour toute l'app : micro, reconnaissance, insertion dans le champ visé.

import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import 'dictation_inserter.dart';
import 'dictation_types.dart';
import 'transcript_text.dart';

/// État de la dictée en cours et commandes pour la démarrer, l'arrêter ou changer de langue.
class DictationController extends ChangeNotifier {
  DictationController({
    required this.audio,
    required this.recognizer,
    required this.models,
    this.language = DictationLanguage.fr,
    this.maxDuration = const Duration(minutes: 5),
  });

  final AudioSource audio;
  final SpeechRecognizer recognizer;
  final ModelAvailability models;
  final Duration maxDuration;
  DictationLanguage language;
  DictationState state = DictationState.idle;
  DictationProblem? problem;
  TextEditingController? target;
  DictationInserter? _inserter;
  StreamSubscription<DictationEvent>? _events;
  StreamSubscription<Float32List>? _samples;
  Timer? _limit;

  /// Démarre la dictée dans [field] ; une dictée en cours ailleurs est arrêtée d'abord.
  Future<void> start(TextEditingController field) async {
    if (state != DictationState.idle) {
      if (identical(target, field)) return;
      await stop();
    }
    problem = null;
    target = field;
    _set(DictationState.preparing);
    final failure = await _open(field);
    if (failure != null) await _fail(failure);
  }

  /// Ouvre reconnaisseur et micro ; rend la raison de l'échec, ou null si tout écoute.
  Future<DictationProblem?> _open(TextEditingController field) async {
    if (!models.isInstalled(language)) return DictationProblem.modelMissing;
    if (!await audio.requestPermission()) {
      return DictationProblem.microphoneDenied;
    }
    try {
      await recognizer.start(language);
    } on DictationException catch (e) {
      return e.problem;
    }
    try {
      _listen(field, await audio.start());
    } on Object {
      return DictationProblem.microphoneUnavailable;
    }
    return null;
  }

  /// Branche le micro sur le reconnaisseur et le reconnaisseur sur le champ.
  void _listen(TextEditingController field, Stream<Float32List> samples) {
    final inserter = DictationInserter(field);
    _inserter = inserter;
    _events = recognizer.events.listen(
      (e) => _onEvent(inserter, e),
      onError: (Object _) =>
          unawaited(_fail(DictationProblem.recognizerFailed)),
    );
    _samples = samples.listen(
      recognizer.feed,
      onError: (Object _) =>
          unawaited(_fail(DictationProblem.microphoneUnavailable)),
    );
    _limit = Timer(maxDuration, () => unawaited(stop()));
    _set(DictationState.listening);
  }

  /// Écrit un texte reconnu dans le champ.
  void _onEvent(DictationInserter inserter, DictationEvent event) {
    final text = normalizeTranscript(event.text);
    switch (event) {
      case DictationPartial():
        inserter.showPartial(text);
      case DictationFinal():
        inserter.commit(text);
    }
  }

  /// Arrête la dictée : le dernier texte reconnu est validé dans le champ.
  Future<void> stop() async {
    if (state == DictationState.idle || state == DictationState.stopping) {
      return;
    }
    _set(DictationState.stopping);
    await _close();
    _finish();
  }

  /// Le champ visé disparaît : arrêt immédiat, plus aucune écriture dedans.
  void release(TextEditingController field) {
    if (!identical(target, field) || state == DictationState.idle) return;
    _inserter?.release();
    unawaited(stop());
  }

  /// Change la langue ; une dictée en cours est arrêtée.
  Future<void> setLanguage(DictationLanguage next) async {
    if (next == language) return;
    await stop();
    language = next;
    notifyListeners();
  }

  /// Arrête sur échec : la raison est conservée pour l'interface.
  Future<void> _fail(DictationProblem why) async {
    problem = why;
    await _close();
    _finish();
  }

  /// Coupe le minuteur, le micro et le reconnaisseur ; sans erreur même si l'un d'eux échoue.
  Future<void> _close() async {
    _limit?.cancel();
    _limit = null;
    await _samples?.cancel();
    _samples = null;
    await _quietly(audio.stop);
    await _quietly(recognizer.stop);
    await _events?.cancel();
    _events = null;
  }

  /// Exécute [action] en ignorant son échec : on ferme, on n'a plus rien à tenter.
  Future<void> _quietly(Future<void> Function() action) async {
    try {
      await action();
    } on Object {
      return;
    }
  }

  /// Retour à l'état de repos.
  void _finish() {
    _inserter = null;
    target = null;
    _set(DictationState.idle);
  }

  /// Change l'état et prévient l'interface.
  void _set(DictationState next) {
    state = next;
    notifyListeners();
  }
}
