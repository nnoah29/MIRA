import 'dart:async';

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

/// Code d'erreur du moteur : l'utilisateur n'a pas encore autorisé l'envoi vers ce service.
const consentRequiredCode = 'incompatible_state.ai_consent_required';

class _Pending {
  final AiConnectionInfo connection;
  final String question;
  final ChatContextInput input;
  final String providerName;
  const _Pending(this.connection, this.question, this.input, this.providerName);
}

/// Adresse de la machine locale : un service sans clé y reste utilisable.
bool isLoopbackUrl(String? url) {
  final host = Uri.tryParse(url ?? '')?.host ?? '';
  return host == 'localhost' || host == '::1' || host.startsWith('127.');
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
  _Pending? _consent;
  StreamSubscription<AiChatEvent>? _sub;
  Completer<void>? _finished;
  ChatMsg? _current;
  String? _operation;
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
    if (ai == null) return _demoSend(q);
    busy = true;
    notifyListeners();
    final connection = _connection = await _usable(ai);
    if (connection == null) {
      busy = false;
      return _demoSend(q);
    }
    await _generate(ai, connection, q, input);
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
  ) async {
    final user = ChatMsg(role: 'me', text: q);
    final reply = ChatMsg(role: 'ai', done: false, text: '', fromEngine: true);
    final request = buildChatStart(history: [...chat, user], input: input);
    final origin = _Pending(connection, q, input, '');
    chat
      ..add(user)
      ..add(reply);
    busy = true;
    _current = reply;
    _operation = null;
    final finished = _finished = Completer<void>();
    notifyListeners();
    _sub = ai
        .chat(request, connection: connection.id)
        .listen(
          (e) => _onEvent(e, reply, origin),
          onError: (Object e) => _fail(reply, engineErrorText(e)),
          onDone: () => _finish(reply),
          cancelOnError: true,
        );
    await finished.future;
  }

  void _onEvent(AiChatEvent event, ChatMsg reply, _Pending origin) {
    switch (event) {
      case AiStarted():
        _operation = event.operation;
      case AiText():
        reply.text += event.text;
        reply.shown = reply.text;
        notifyListeners();
      case AiUsage():
        break;
      case AiDone():
        _onDone(reply, event);
      case AiFailed() when event.code == consentRequiredCode:
        _askConsent(reply, origin);
      case AiFailed():
        _fail(
          reply,
          engineErrorText(EngineException(event.code, event.message)),
        );
    }
  }

  void _onDone(ChatMsg reply, AiDone done) {
    if (done.failed) {
      final error = EngineException(done.code ?? 'internal.bug', '');
      return _fail(reply, engineErrorText(error), retry: done.retryAfterSecs);
    }
    if (done.reason == 'cancelled' && reply.text.isEmpty) {
      reply.text = t('Génération arrêtée.');
    }
    if (done.reason == 'max_tokens') {
      reply.text +=
          '\n\n${t('Réponse tronquée : la limite de longueur est atteinte.')}';
    }
    _finish(reply);
  }

  void _fail(ChatMsg reply, String error, {int? retry}) {
    final wait = retry == null ? '' : ' ${t('Réessaie dans {0} s.', [retry])}';
    final head = reply.text.isEmpty ? '' : '${reply.text}\n\n';
    reply.text = '$head$error$wait';
    reply.failed = true;
    _finish(reply);
  }

  void _finish(ChatMsg reply) {
    if (_current != reply) return;
    if (reply.text.isEmpty) {
      reply.text = t('Le modèle n’a rien répondu.');
      reply.failed = true;
    }
    reply.shown = reply.text;
    reply.done = true;
    busy = false;
    _current = null;
    _operation = null;
    _sub?.cancel();
    _sub = null;
    final finished = _finished;
    if (finished != null && !finished.isCompleted) finished.complete();
    notifyListeners();
  }

  void _askConsent(ChatMsg reply, _Pending origin) {
    final c = origin.connection;
    chat.remove(reply);
    chat.removeLast();
    final name = _providerNames[c.provider] ?? c.label;
    _consent = _Pending(c, origin.question, origin.input, name);
    _finish(reply);
  }

  /// L'utilisateur accepte : le moteur enregistre l'accord, puis la question part.
  Future<void> grantConsent() async {
    final pending = _consent;
    final ai = admin();
    if (pending == null || ai == null || busy) return;
    _consent = null;
    try {
      await ai.acknowledgeConsent(pending.connection.id);
    } on Object catch (e) {
      return _note(pending.question, engineErrorText(e));
    }
    await _generate(ai, pending.connection, pending.question, pending.input);
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

  /// Arrête la génération en cours en gardant le texte déjà reçu.
  Future<void> stop() async {
    final reply = _current;
    if (reply == null) return _stopDemo();
    final ai = admin();
    final operation = _operation;
    if (ai != null && operation != null) {
      try {
        await ai.cancel(operation);
      } on Object {
        /* l'annulation par fermeture du flux suffit */
      }
    }
    if (reply.text.isEmpty) reply.text = t('Génération arrêtée.');
    _finish(reply);
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
    _sub?.cancel();
    _sub = null;
    _demoTimer?.cancel();
    _demoTimer = null;
    final finished = _finished;
    if (finished != null && !finished.isCompleted) finished.complete();
    _current = null;
    _consent = null;
    busy = false;
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
    _sub?.cancel();
    _demoTimer?.cancel();
    super.dispose();
  }
}
