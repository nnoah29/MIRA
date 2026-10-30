import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';

import '../data/format.dart';
import '../data/seed.dart';
import '../engine/ai_admin.dart';
import '../engine/engine_client.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';
import 'assistant.dart';
import 'chat_context.dart';
import 'chat_message.dart';
import 'chat_pending.dart';
import 'chat_tools.dart';

/// Code d'erreur du moteur : l'utilisateur n'a pas encore autorisé l'envoi vers ce service.
const consentRequiredCode = 'incompatible_state.ai_consent_required';

/// Adresse de la machine locale : un service sans clé y reste utilisable.
bool isLoopbackUrl(String? url) {
  final host = Uri.tryParse(url ?? '')?.host ?? '';
  return host == 'localhost' ||
      InternetAddress.tryParse(host)?.isLoopback == true;
}

/// Contexte vide : la question part sans le mail ouvert.
const noContext = ChatContextInput();

/// Une génération en cours : sa réponse, son abonnement, son opération et une annulation demandée trop tôt.
class _Run {
  final ChatMsg reply;
  final Completer<void> finished = Completer<void>();
  StreamSubscription<AiChatEvent>? sub;
  String? operation;
  bool cancelPending = false;
  _Run(this.reply);
}

/// Fil de l'assistant : réponses du moteur d'IA, ou repli de démonstration sans connexion utilisable.
class ChatController extends ChangeNotifier {
  ChatController({
    required this.admin,
    required this.welcome,
    required this.demoAnswer,
    required this.after,
  }) {
    chat = [welcome()];
  }

  final AiAdmin? Function() admin;
  final ChatMsg Function() welcome;
  final Answer Function(String question) demoAnswer;
  final Timer Function(int ms, VoidCallback run) after;

  late List<ChatMsg> chat;
  bool busy = false;
  AiConnectionInfo? _connection;
  ChatPending? _consent;
  ChatPending? _content;
  ChatContextInput _lastInput = noContext;
  _Run? _run;
  final Set<_Run> _orphans = {};
  Timer? _demoTimer;
  Map<String, String> _providerNames = const {};

  /// Vrai quand aucune connexion utilisable ne répond : l'assistant de démonstration parle.
  bool get demo => admin() == null || _connection == null;

  /// Libellé de la connexion qui répond (`null` en démonstration).
  String? get connectionLabel => _connection?.label;

  /// Vrai quand l'envoi attend l'accord de l'utilisateur pour cette connexion.
  bool needsConsent(String connectionId) =>
      _consent?.connection.id == connectionId;

  /// Connexion en attente d'accord (`null` : aucune).
  String? get consentConnectionId => _consent?.connection.id;

  /// Nom du service qui attend l'accord (`null` : aucun).
  String? get consentProviderName => _consent?.providerName;

  /// Machine qui recevra le texte quand l'adresse est libre (`null` : adresse fixe du service).
  String? get consentHost {
    final host = Uri.tryParse(_consent?.connection.baseUrl ?? '')?.host ?? '';
    return host.isEmpty ? null : host;
  }

  /// Connexion qui attend l'autorisation d'un type de contenu (`null` : aucune).
  String? get contentConnectionId => _content?.connection.id;

  /// Nom du service qui attend l'autorisation d'un type de contenu (`null` : aucun).
  String? get contentProviderName => _content?.providerName;

  /// Types de contenu en attente d'autorisation.
  List<String> get contentKinds => _content?.kinds ?? const [];

  /// Relit les connexions pour savoir si une IA est utilisable.
  Future<void> refresh() async {
    final ai = admin();
    final before = _connection?.id;
    _connection = ai == null ? null : await _usable(ai);
    if (before != _connection?.id) notifyListeners();
  }

  Future<AiConnectionInfo?> _usable(AiAdmin ai) async {
    try {
      _providerNames = {for (final p in await ai.providers()) p.id: p.name};
      for (final c in await ai.connections()) {
        if (c.isDefault && (c.hasKey || isLoopbackUrl(c.baseUrl))) return c;
      }
    } on Object {
      return null;
    }
    return null;
  }

  /// Envoie une question : réponse du moteur d'IA, ou de la démonstration sans connexion utilisable.
  Future<void> send(String text, ChatContextInput input) async {
    final q = text.trim();
    final ai = admin();
    if (q.isEmpty || busy) return;
    _consent = _content = null;
    if (ai == null) return _demoSend(q);
    busy = true;
    notifyListeners();
    final connection = _connection = await _usable(ai);
    if (connection == null) {
      busy = false;
      return _demoSend(q);
    }
    await _generate(ai, connection, q, input, 0);
  }

  void _demoSend(String q) {
    chat.add(ChatMsg(role: 'me', text: q));
    final ans = demoAnswer(q);
    final msg = ChatMsg(
      role: 'ai',
      done: false,
      text: ans.text,
      draft: ans.draft,
      refs: ans.refs,
    );
    chat.add(msg);
    busy = true;
    notifyListeners();
    var i = 0;
    void tick() {
      i = (i + rand(2, 5).round()).clamp(0, msg.text.length);
      msg.shown = clipText(msg.text, i);
      if (i >= msg.text.length) _endDemo(msg);
      notifyListeners();
      if (!msg.done) _demoTimer = after(22, tick);
    }

    _demoTimer = after(rand(500, 900).round(), tick);
  }

  void _endDemo(ChatMsg msg) {
    msg.done = true;
    busy = false;
    _demoTimer = null;
  }

  Future<void> _generate(
    AiAdmin ai,
    AiConnectionInfo connection,
    String q,
    ChatContextInput input,
    int stage,
  ) async {
    _lastInput = input;
    final user = ChatMsg(role: 'me', text: q);
    final reply = ChatMsg(role: 'ai', done: false, text: '', fromEngine: true);
    final request = buildChatStart(history: [...chat, user], input: input);
    final origin = ChatPending(connection, q, input, '', stage);
    chat
      ..add(user)
      ..add(reply);
    busy = true;
    final run = _run = _Run(reply);
    notifyListeners();
    run.sub = ai
        .chat(request, connection: connection.id)
        .listen(
          (e) => _onEvent(e, run, origin),
          onError: (Object e) => _ended(run, engineErrorText(e)),
          onDone: () => _ended(run),
          cancelOnError: true,
        );
    await run.finished.future;
  }

  void _onEvent(AiChatEvent event, _Run run, ChatPending origin) {
    if (event is AiStarted) return _started(run, event.operation);
    if (_run != run) return;
    final reply = run.reply;
    switch (event) {
      case AiStarted():
        break;
      case AiText():
        reply.text += event.text;
        reply.shown = reply.text;
        notifyListeners();
      case AiMasked():
        reply.maskedCount = event.count;
        notifyListeners();
      case AiUsage():
        break;
      case AiToolCalled() || AiToolFinished():
        applyToolEvent(reply, event);
        notifyListeners();
      case AiDone():
        _onDone(run, event);
      case AiFailed()
          when event.code == consentRequiredCode && origin.stage == 0:
        _askConsent(run, origin);
      case AiFailed()
          when event.code == contentNotAllowedCode && origin.stage < 2:
        _askContent(run, origin);
      case AiFailed():
        final error = EngineException(event.code, event.message);
        _fail(run, engineErrorText(error));
    }
  }

  void _ended(_Run run, [String? error]) {
    _orphans.remove(run);
    if (error == null) return _finish(run);
    _fail(run, error);
  }

  void _started(_Run run, String operation) {
    run.operation = operation;
    if (run.cancelPending) unawaited(_cancelOperation(run));
  }

  void _onDone(_Run run, AiDone done) {
    final reply = run.reply;
    if (done.failed) {
      final error = EngineException(done.code ?? 'internal.bug', '');
      return _fail(run, engineErrorText(error), retry: done.retryAfterSecs);
    }
    if (done.reason == 'cancelled' && reply.text.isEmpty) _stoppedNote(reply);
    if (done.reason == 'max_tokens') {
      reply.note = t('Réponse tronquée : la limite de longueur est atteinte.');
    }
    _finish(run);
  }

  void _stoppedNote(ChatMsg reply) {
    reply.note = t('Génération arrêtée.');
    reply.failed = true;
  }

  void _fail(_Run run, String error, {int? retry}) {
    if (_run != run) return;
    final reply = run.reply;
    final wait = retry == null ? '' : ' ${t('Réessaie dans {0} s.', [retry])}';
    final head = reply.text.isEmpty ? '' : '${reply.text}\n\n';
    reply.text = '$head$error$wait';
    reply.failed = true;
    _finish(run);
  }

  void _finish(_Run run) {
    if (_run != run) return;
    final reply = run.reply;
    if (reply.text.isEmpty && reply.note == null) {
      reply.text = t('Le modèle n’a rien répondu.');
      reply.failed = true;
    }
    reply.shown = reply.text;
    reply.done = true;
    busy = false;
    _run = null;
    if (!run.cancelPending) unawaited(run.sub?.cancel());
    if (!run.finished.isCompleted) run.finished.complete();
    notifyListeners();
  }

  Future<void> _cancelOperation(_Run run) async {
    final ai = admin();
    final operation = run.operation;
    if (ai != null && operation != null) {
      try {
        await ai.cancel(operation);
      } on Object {
        /* l'annulation par fermeture du flux suffit */
      }
    }
    await run.sub?.cancel();
    run.sub = null;
    _orphans.remove(run);
  }

  Future<void> _abort(_Run run) {
    if (run.operation != null) return _cancelOperation(run);
    run.cancelPending = true;
    _orphans.add(run);
    return Future.value();
  }

  ChatPending _withdraw(_Run run, ChatPending origin) {
    final c = origin.connection;
    chat.remove(run.reply);
    chat.removeLast();
    final name = _providerNames[c.provider] ?? c.label;
    return ChatPending(c, origin.question, origin.input, name, origin.stage);
  }

  void _askConsent(_Run run, ChatPending origin) {
    _consent = _withdraw(run, origin);
    _finish(run);
  }

  void _askContent(_Run run, ChatPending origin) {
    _content = _withdraw(run, origin);
    _finish(run);
  }

  /// L'utilisateur accepte : le moteur enregistre l'accord pour les types [kinds], puis la question part.
  Future<void> grantConsent([List<String> kinds = const [aiKindMail]]) async {
    final pending = _consent;
    final ai = admin();
    if (pending == null || ai == null || busy) return;
    _consent = null;
    busy = true;
    notifyListeners();
    try {
      await ai.acknowledgeConsent(pending.connection.id, kinds: kinds);
    } on Object catch (e) {
      return _grantFailed(pending, e);
    }
    if (!busy) return;
    final input = kinds.contains(aiKindMail) ? pending.input : noContext;
    await _generate(ai, pending.connection, pending.question, input, 1);
  }

  void _grantFailed(ChatPending pending, Object error) {
    busy = false;
    _note(pending.question, engineErrorText(error));
  }

  /// L'utilisateur autorise le type de contenu : il s'ajoute aux types de la connexion, puis la question repart une fois.
  Future<void> grantContent() async {
    final pending = _content;
    final ai = admin();
    if (pending == null || ai == null || busy) return;
    _content = null;
    busy = true;
    notifyListeners();
    final kinds = {...pending.connection.allowedKinds, ...pending.kinds};
    final replaced = pending.replacing;
    try {
      await ai.setAllowedContent(pending.connection.id, kinds.toList());
    } on Object catch (e) {
      return _grantFailed(pending, e);
    }
    if (!busy) return;
    if (replaced != null) _dropExchange(replaced);
    await _generate(ai, pending.connection, pending.question, pending.input, 2);
  }

  /// Vrai quand la réponse est la dernière du fil et que rien ne tourne : un type refusé peut être autorisé.
  bool canAuthorize(ChatMsg reply) =>
      !busy && _connection != null && chat.isNotEmpty && chat.last == reply;

  /// L'utilisateur veut autoriser le type refusé à un outil : la boîte d'autorisation s'ouvre, et sa question repart si l'accord est donné.
  void askToolAuthorization(ChatMsg reply, ToolActivity tool) {
    final connection = _connection;
    final kind = tool.deniedKind;
    if (kind == null || connection == null || !canAuthorize(reply)) return;
    final at = chat.indexOf(reply);
    if (at < 1) return;
    final name = _providerNames[connection.provider] ?? connection.label;
    final question = chat[at - 1].text;
    _content = ChatPending(connection, question, _lastInput, name, 2, [
      kind,
    ], reply);
    notifyListeners();
  }

  void _dropExchange(ChatMsg reply) {
    final at = chat.indexOf(reply);
    if (at < 1) return;
    chat.removeRange(at - 1, at + 1);
  }

  /// L'utilisateur refuse le type de contenu : sa question reste dans le fil, rien n'a été envoyé.
  void declineContent() {
    final pending = _content;
    if (pending == null) return;
    _content = null;
    if (pending.replacing != null) return notifyListeners();
    _note(pending.question, t('Non envoyé : contenu non autorisé.'));
  }

  /// L'utilisateur refuse : sa question reste dans le fil, rien n'a été envoyé.
  void declineConsent() {
    final pending = _consent;
    if (pending == null) return;
    _consent = null;
    _note(
      pending.question,
      t('Envoi annulé : rien n’a été transmis au service d’IA.'),
    );
  }

  void _note(String question, String text) {
    chat.add(ChatMsg(role: 'me', text: question));
    chat.add(ChatMsg(role: 'ai', text: text, fromEngine: true, failed: true));
    notifyListeners();
  }

  /// Arrête la génération en cours en gardant le texte déjà reçu ; l'annulation part dès que l'opération est connue.
  Future<void> stop() async {
    final run = _run;
    if (run == null) return _stopDemo();
    if (run.reply.text.isEmpty) _stoppedNote(run.reply);
    final cancelling = _abort(run);
    _finish(run);
    await cancelling;
  }

  void _stopDemo() {
    final timer = _demoTimer;
    final last = chat.isEmpty ? null : chat.last;
    if (timer == null || last == null) return;
    timer.cancel();
    last.shown = last.text;
    _endDemo(last);
    notifyListeners();
  }

  void _halt() {
    final run = _run;
    _run = null;
    _demoTimer?.cancel();
    _demoTimer = null;
    _consent = null;
    _content = null;
    busy = false;
    if (run == null) return;
    if (!run.finished.isCompleted) run.finished.complete();
    unawaited(_abort(run));
  }

  /// Repart d'un fil neuf : arrête ce qui tourne et remet l'accueil.
  void reset() {
    _halt();
    chat = [welcome()];
    notifyListeners();
  }

  /// Vide le fil de la conversation, en gardant l'accueil.
  void clear() {
    final hello = chat.take(1).toList();
    _halt();
    chat = hello;
    notifyListeners();
  }

  @override
  void dispose() {
    final run = _run;
    _run = null;
    unawaited(run?.sub?.cancel());
    for (final orphan in _orphans) {
      unawaited(orphan.sub?.cancel());
    }
    _demoTimer?.cancel();
    super.dispose();
  }
}
