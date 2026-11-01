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
  });

  final AiAdmin? Function() admin;
  final Future<String?> Function() connection;
  final ComposeState Function() compose;
  final VoidCallback onChanged;

  bool busy = false;
  String? reply;
  String? error;
  DraftSnapshot? _undo;
  DraftSelection? _selection;
  AiDraftProposal? _last;
  StreamSubscription<AiChatEvent>? _sub;
  Completer<void>? _finished;
  String? _operation;
  bool _stopped = false;
  bool _disposed = false;

  /// Vrai quand la dernière application peut être annulée.
  bool get canUndo => _undo != null;

  /// Envoie l'instruction ; la dernière proposition du tour est appliquée à la fin.
  Future<void> run(String instruction, {DraftSelection? selection}) async {
    final text = instruction.trim();
    if (busy || text.isEmpty) return;
    _begin(selection);
    final id = await connection();
    final ai = admin();
    if (ai == null || id == null) {
      return _end(
        error: t(
          'Aucune IA utilisable : connecte-la dans les réglages de l’IA.',
        ),
      );
    }
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
    final finished = _finished;
    if (!busy || finished == null) return;
    _stopped = true;
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
    onChanged();
    _notify();
  }

  void _begin(DraftSelection? selection) {
    busy = true;
    reply = null;
    error = null;
    _last = null;
    _operation = null;
    _stopped = false;
    _selection = selection;
    _notify();
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
        onChanged();
      } else {
        error = problem;
      }
    } else if (!_stopped && error == null && (reply ?? '').trim().isEmpty) {
      reply = t('L’IA n’a rien proposé : reformule ta demande.');
    }
    _end(error: error);
  }

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
    unawaited(_sub?.cancel());
    super.dispose();
  }
}
