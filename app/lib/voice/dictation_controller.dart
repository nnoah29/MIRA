//! Pilote une dictée à la fois pour toute l'app : micro, reconnaissance, insertion dans le champ visé.

import 'dart:async';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import 'audio_level.dart';
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
    this.recognizerStartTimeout = const Duration(seconds: 20),
  });

  final AudioSource audio;
  final SpeechRecognizer recognizer;
  final ModelAvailability models;
  final Duration maxDuration;
  final Duration recognizerStartTimeout;
  DictationLanguage language;
  DictationState state = DictationState.idle;
  DictationProblem? problem;
  TextEditingController? target;
  DictationInserter? _inserter;
  StreamSubscription<DictationEvent>? _events;
  StreamSubscription<Float32List>? _samples;
  Timer? _limit;
  int _session = 0;
  Future<void>? _opening;
  Future<void>? _closing;
  final List<Float32List> _backlog = [];
  int _backlogSamples = 0;
  bool _recognizerReady = false;
  bool _micOpen = false;
  bool _stopRequested = false;
  TextEditingController? _retryTarget;

  /// Niveau sonore du micro, de 0 à 1, pour l'animation ; il retombe doucement au silence.
  final ValueNotifier<double> level = ValueNotifier<double>(0);

  /// Dit si le reconnaisseur a fini de charger son modèle (avant, le son est seulement mis de côté).
  bool get recognizerReady => _recognizerReady;

  /// Démarre la dictée dans [field] ; une dictée en cours ailleurs est arrêtée d'abord.
  Future<void> start(TextEditingController field) async {
    await _closing;
    if (state != DictationState.idle) {
      if (identical(target, field)) return;
      await stop();
    }
    problem = null;
    _stopRequested = false;
    _retryTarget = null;
    target = field;
    final session = ++_session;
    _set(DictationState.preparing);
    final run = _run(field, session);
    _opening = run;
    await run;
    if (identical(_opening, run)) _opening = null;
  }

  /// Ouvre la dictée de cette session ; une session dépassée par un arrêt est abandonnée sans bruit.
  Future<void> _run(TextEditingController field, int session) async {
    try {
      final failure = await _open(field, session);
      if (failure == DictationProblem.modelMissing) _retryTarget = field;
      if (failure != null) await _fail(failure, session);
    } on _Superseded {
      return;
    }
  }

  /// Rend la raison de l'échec d'ouverture, ou null si tout écoute ; lève [_Superseded] si un arrêt est survenu.
  Future<DictationProblem?> _open(
    TextEditingController field,
    int session,
  ) async {
    if (!models.isInstalled(language)) return DictationProblem.modelMissing;
    final allowed = await _guard(
      audio.requestPermission,
      DictationProblem.microphoneUnavailable,
    );
    _stillCurrent(session);
    _abortIfNothingSaid();
    if (allowed == false) return DictationProblem.microphoneDenied;
    if (allowed is! bool) return allowed as DictationProblem;
    final samples = await _guard(
      audio.start,
      DictationProblem.microphoneUnavailable,
    );
    _stillCurrent(session);
    if (samples is DictationProblem) return samples;
    if (_stopRequested) {
      await _quietly(audio.stop);
      throw const _Superseded();
    }
    _listen(field, samples as Stream<Float32List>);
    final failure = await _guard(
      () => recognizer.start(language).timeout(recognizerStartTimeout),
      DictationProblem.recognizerFailed,
    );
    _stillCurrent(session);
    if (failure is DictationProblem) return failure;
    _flushBacklog();
    return null;
  }

  /// Exécute [action] ; en cas d'échec rend la raison [fallback] (ou celle portée par une [DictationException]).
  Future<Object?> _guard(
    Future<Object?> Function() action,
    DictationProblem fallback,
  ) async {
    try {
      return await action();
    } on DictationException catch (e) {
      return e.problem;
    } on Object {
      return fallback;
    }
  }

  /// Un arrêt demandé avant l'ouverture du micro annule l'ouverture : il n'y a rien à garder.
  void _abortIfNothingSaid() {
    if (_stopRequested && !_micOpen) throw const _Superseded();
  }

  /// Lève [_Superseded] si un arrêt ou un autre démarrage a dépassé cette session.
  void _stillCurrent(int session) {
    if (session != _session) throw const _Superseded();
  }

  /// Branche le micro et le champ : le son est mis de côté tant que le reconnaisseur charge son modèle.
  void _listen(TextEditingController field, Stream<Float32List> samples) {
    _micOpen = true;
    _recognizerReady = false;
    final inserter = DictationInserter(field);
    _inserter = inserter;
    _events = recognizer.events.listen(
      (e) => _onEvent(inserter, e),
      onError: (Object _) =>
          unawaited(_fail(DictationProblem.recognizerFailed, _session)),
    );
    _samples = samples.listen(
      _onSamples,
      onError: (Object _) =>
          unawaited(_fail(DictationProblem.microphoneUnavailable, _session)),
    );
    _limit = Timer(maxDuration, () => unawaited(stop()));
    _set(DictationState.listening);
  }

  /// Transmet le son au reconnaisseur, ou le garde (30 s au plus) pendant qu'il charge son modèle.
  void _onSamples(Float32List chunk) {
    level.value = math.max(audioLevel(chunk), level.value * 0.6);
    if (_recognizerReady) return recognizer.feed(chunk);
    if (_backlogSamples + chunk.length > _backlogLimit) return;
    _backlogSamples += chunk.length;
    _backlog.add(chunk);
  }

  /// Le reconnaisseur est prêt : il reçoit d'abord tout ce qui a été dit pendant le chargement.
  void _flushBacklog() {
    _recognizerReady = true;
    _backlog.forEach(recognizer.feed);
    _backlog.clear();
    _backlogSamples = 0;
    notifyListeners();
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

  /// Arrête la dictée : le dernier texte reconnu est validé dans le champ ; attend la fin d'une ouverture en cours.
  Future<void> stop() async {
    if (state == DictationState.idle) return;
    if (state == DictationState.stopping) return _closing;
    _stopRequested = true;
    if (_opening == null) _session++;
    _set(DictationState.stopping);
    final closing = _shutdown();
    _closing = closing;
    await closing;
    if (identical(_closing, closing)) _closing = null;
  }

  /// Laisse finir une ouverture dépassée, puis ferme micro et reconnaisseur et revient au repos.
  Future<void> _shutdown() async {
    if (_opening != null) await _closeMic();
    await _opening;
    await _close();
    _finish();
  }

  /// Le champ visé disparaît : arrêt immédiat, plus aucune écriture dedans.
  void release(TextEditingController field) {
    if (identical(_retryTarget, field)) _retryTarget = null;
    if (!identical(target, field) || state == DictationState.idle) return;
    _inserter?.release();
    _session++;
    unawaited(stop());
  }

  /// Relance, une seule fois, la dictée que l'absence du modèle avait empêchée, si son champ existe encore.
  Future<void> retry() async {
    final field = _retryTarget;
    if (field == null || state != DictationState.idle) return;
    await start(field);
  }

  /// Change la langue ; une dictée en cours est arrêtée.
  Future<void> setLanguage(DictationLanguage next) async {
    if (next == language) return;
    await stop();
    language = next;
    notifyListeners();
  }

  /// Arrête sur échec de la session [session] : la raison est conservée pour l'interface.
  Future<void> _fail(DictationProblem why, int session) async {
    if (session != _session) return;
    _session++;
    problem = why;
    await _close();
    _finish();
  }

  /// Coupe le minuteur, le micro et le reconnaisseur ; sans erreur même si l'un d'eux échoue.
  Future<void> _close() async {
    _limit?.cancel();
    _limit = null;
    await _closeMic();
    await _quietly(recognizer.stop);
    await _events?.cancel();
    _events = null;
    level.value = 0;
    _backlog.clear();
    _backlogSamples = 0;
    _recognizerReady = false;
  }

  /// Ferme le micro une seule fois : plus aucun son n'arrive, ni en réserve ni au reconnaisseur.
  Future<void> _closeMic() async {
    if (!_micOpen) return;
    _micOpen = false;
    await _samples?.cancel();
    _samples = null;
    await _quietly(audio.stop);
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
    _stopRequested = false;
    _inserter = null;
    target = null;
    _set(DictationState.idle);
  }

  @override
  void dispose() {
    level.dispose();
    super.dispose();
  }

  /// Change l'état et prévient l'interface.
  void _set(DictationState next) {
    state = next;
    notifyListeners();
  }
}

const _backlogLimit = 16000 * 30;

/// Signal interne : la session d'ouverture a été dépassée par un arrêt ou un autre démarrage.
class _Superseded implements Exception {
  const _Superseded();
}
