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

/// Texte d'une réponse de l'IA sans lignes vides ni espaces en bordure de ligne : les modèles en laissent entre leurs tours, ce qui étire la barre.
String tidyReply(String text) => text
    .replaceAll('\r', '')
    .split('\n')
    .map((line) => line.trim())
    .where((line) => line.isNotEmpty)
    .join('\n');

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
  String _appliedBody = '';
  String _appliedSubject = '';
  DraftSelection? _selection;
  AiDraftProposal? _last;
  StreamSubscription<AiChatEvent>? _sub;
  Completer<void>? _finished;
  String? _operation;
  String? _stopReason;
  String _rawReply = '';
  AiAdmin? _ai;
  String? _connectionId;
  Map<String, dynamic>? _request;
  Map<String, dynamic>? _mode;
  bool _retried = false;
  bool _sawOutput = false;
  bool _stopped = false;
  bool _disposed = false;
  int _generation = 0;

  /// Vrai quand la dernière application peut être annulée.
  bool get canUndo => _undo != null;

  /// Vrai quand l'utilisateur a modifié le brouillon depuis l'application : annuler effacerait aussi ses modifications.
  bool get undoDropsEdits {
    final draft = compose();
    return canUndo &&
        (draft.body.text != _appliedBody ||
            draft.subject.text != _appliedSubject);
  }

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
    _ai = ai;
    _connectionId = id;
    _request = buildDraftRequest(compose(), text, selection: selection);
    _mode = {'kind': 'composer', 'selection': selection != null};
    final finished = _finished = Completer<void>();
    _listen(finished);
    await finished.future;
  }

  void _listen(Completer<void> finished) {
    _sub = _ai!
        .chat(_request!, connection: _connectionId, mode: _mode)
        .listen(
          _onEvent,
          onError: (Object e) => _fail(engineErrorText(e)),
          onDone: () => _complete(finished),
          cancelOnError: true,
        );
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
    _rawReply = '';
    _retried = false;
    _sawOutput = false;
    _stopped = false;
    _selection = selection;
    _notify();
    return ++_generation;
  }

  void _detectSignature() {
    final draft = compose();
    final sig = signature?.call() ?? '';
    if (draft.sigText.isNotEmpty || sig.trim().isEmpty) return;
    final body = draft.body.text;
    final ending = body.trimRight().endsWith(sig);
    if (ending || (draft.mode == 'fwd' && body.contains(sig))) {
      draft.sigText = sig;
    }
  }

  void _onEvent(AiChatEvent event) {
    switch (event) {
      case AiStarted():
        _operation = event.operation;
      case AiText():
        _sawOutput = true;
        _rawReply += event.text;
        reply = tidyReply(_rawReply);
        _notify();
      case AiToolCalled():
        _sawOutput = true;
      case AiDraftProposal():
        _sawOutput = true;
        _last = event;
      case AiFailed():
        _failOrRetry(event.code, event.message);
      case AiDone() when event.failed:
        _failOrRetry(event.code ?? 'internal.bug', '');
      case AiDone():
        _stopReason = event.reason;
      default:
        break;
    }
  }

  /// Un refus du fournisseur avant toute réponse (limite de sortie trop haute pour son modèle) relance une fois avec la limite par défaut du moteur.
  void _failOrRetry(String code, String message) {
    final request = _request;
    final finished = _finished;
    final refused = code == 'invalid_input.ai_input_too_large';
    final retry = refused && !_retried && !_sawOutput && finished != null;
    if (!retry ||
        request == null ||
        !request.containsKey('max_output_tokens')) {
      return _fail(engineErrorText(EngineException(code, message)));
    }
    _retried = true;
    unawaited(_sub?.cancel());
    _request = {...request}..remove('max_output_tokens');
    _listen(finished);
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
        _appliedBody = compose().body.text;
        _appliedSubject = compose().subject.text;
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
