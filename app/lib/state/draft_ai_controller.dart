//! « Écrire avec l'IA » dans le composeur : instruction globale ou sur sélection, application au brouillon, annulation.

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../engine/ai_admin.dart';
import '../engine/engine_client.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';
import 'draft_ai_request.dart';
import 'draft_apply.dart';
import 'store.dart';

export 'draft_ai_request.dart' show DraftSelection;

/// Une écriture assistée en cours ou terminée : état, réponse texte de l'IA, erreur, et de quoi annuler.
class DraftAiController extends ChangeNotifier {
  DraftAiController({
    required this.admin,
    required this.connection,
    required this.compose,
    required this.onChanged,
    this.signature,
  });

  final AiAdmin? Function() admin;
  final Future<String?> Function() connection;
  final ComposeState Function() compose;
  final VoidCallback onChanged;

  /// Signature de l'expéditeur du brouillon, pour la retrouver dans un brouillon rouvert.
  final String Function()? signature;

  bool busy = false;
  String? reply;
  String? error;

  /// Ce que la dernière application a changé (objet, destinataires avec leurs adresses, corps) ; `null` sans application ou après annulation.
  String? summary;
  DraftSnapshot? _undo;
  DraftSelection? _selection;
  AiDraftProposal? _last;
  StreamSubscription<AiChatEvent>? _sub;
  Completer<void>? _finished;
  String? _operation;
  String? _stopReason;
  bool _stopped = false;
  bool _disposed = false;
  int _generation = 0;

  /// Vrai quand la dernière application peut être annulée.
  bool get canUndo => _undo != null;

  /// Envoie l'instruction ; la dernière proposition du tour est appliquée à la fin.
  Future<void> run(String instruction, {DraftSelection? selection}) async {
    final text = instruction.trim();
    if (busy || text.isEmpty) return;
    final generation = _begin(selection);
    final id = await connection();
    if (generation != _generation || _disposed) return;
    final ai = admin();
    if (ai == null || id == null) {
      return _end(
        error: t(
          'Aucune IA utilisable : connecte-la dans les réglages de l’IA.',
        ),
      );
    }
    _detectSignature();
    final request = buildDraftRequest(compose(), text, selection: selection);
    final mode = {'kind': 'composer', 'selection': selection != null};
    final finished = _finished = Completer<void>();
    _sub = ai
        .chat(request, connection: id, mode: mode)
        .listen(
          _onEvent,
          onError: (Object e) => _fail(engineErrorText(e)),
          onDone: () => _complete(finished),
          cancelOnError: true,
        );
    await finished.future;
  }

  /// Arrête la génération en gardant le brouillon tel qu'il est.
  Future<void> stop() async {
    if (!busy) return;
    _stopped = true;
    final finished = _finished;
    if (finished == null) {
      _generation++;
      return _end();
    }
    final operation = _operation;
    await _sub?.cancel();
    final ai = admin();
    if (ai != null && operation != null) {
      try {
        await ai.cancel(operation);
      } on Object {
        /* l'annulation par fermeture du flux suffit */
      }
    }
    _complete(finished);
  }

  /// Annule la dernière application de l'IA.
  void undo() {
    final snapshot = _undo;
    if (snapshot == null) return;
    snapshot.restore(compose());
    _undo = null;
    summary = null;
    onChanged();
    _notify();
  }

  int _begin(DraftSelection? selection) {
    busy = true;
    _finished = null;
    reply = null;
    error = null;
    summary = null;
    _last = null;
    _operation = null;
    _stopReason = null;
    _stopped = false;
    _selection = selection;
    _notify();
    return ++_generation;
  }

  void _detectSignature() {
    final draft = compose();
    final sig = signature?.call() ?? '';
    if (draft.sigText.isNotEmpty || sig.trim().isEmpty) return;
    if (draft.body.text.contains(sig)) draft.sigText = sig;
  }

  void _onEvent(AiChatEvent event) {
    switch (event) {
      case AiStarted():
        _operation = event.operation;
      case AiText():
        reply = '${reply ?? ''}${event.text}';
        _notify();
      case AiDraftProposal():
        _last = event;
      case AiFailed():
        _fail(engineErrorText(EngineException(event.code, event.message)));
      case AiDone() when event.failed:
        _fail(
          engineErrorText(EngineException(event.code ?? 'internal.bug', '')),
        );
      case AiDone():
        _stopReason = event.reason;
      default:
        break;
    }
  }

  void _fail(String text) {
    error = text;
    final finished = _finished;
    if (finished != null) _complete(finished);
  }

  void _complete(Completer<void> finished) {
    if (finished.isCompleted) return;
    _finishTurn();
    finished.complete();
  }

  void _finishTurn() {
    final proposal = _last;
    if (!_stopped && error == null && proposal != null) {
      final snapshot = DraftSnapshot.of(compose());
      final problem = applyProposal(compose(), proposal, _selection);
      if (problem == null) {
        _undo = snapshot;
        summary = changesOf(proposal);
        onChanged();
      } else {
        error = problem;
      }
    } else if (!_stopped && error == null) {
      error = _whyNothing();
      if (error == null && (reply ?? '').trim().isEmpty) {
        reply = t('L’IA n’a rien proposé : reformule ta demande.');
      }
    }
    _end(error: error);
  }

  /// Pourquoi l'IA n'a rien proposé quand le moteur le sait : limite de longueur atteinte, ou trop de recherches.
  String? _whyNothing() => switch (_stopReason) {
    'max_tokens' => t(
      'La réponse de l’IA a été coupée par sa limite de longueur avant d’écrire le brouillon : certains modèles « réfléchissent » longuement. Réessaie avec une demande plus courte, ou choisis un autre modèle.',
    ),
    'max_tool_rounds' => t(
      'L’IA a enchaîné trop de recherches sans écrire : indique-lui l’adresse du destinataire dans ta demande.',
    ),
    _ => null,
  };

  void _end({String? error}) {
    this.error = error;
    busy = false;
    _sub = null;
    _notify();
  }

  void _notify() {
    if (!_disposed) notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    _generation++;
    final finished = _finished;
    if (finished != null && !finished.isCompleted) finished.complete();
    unawaited(_sub?.cancel());
    super.dispose();
  }
}
