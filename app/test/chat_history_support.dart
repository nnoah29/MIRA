import 'dart:async';

import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/ai_history_admin.dart';
import 'package:mira/state/assistant.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_history_state.dart';
import 'package:mira/state/chat_message.dart';
import 'package:mira/state/chat_message_json.dart';

import 'fake_ai_engine.dart';

/// Contrôleur de fil, historique et moteur factice assemblés comme dans l'app.
class Rig {
  final FakeAiEngine engine;
  final ChatHistoryState history;
  final ChatController chat;
  const Rig(this.engine, this.history, this.chat);

  /// Requêtes d'enregistrement reçues par le moteur.
  List<Map<String, dynamic>> get saves =>
      engine.requestsOf('save_ai_conversation');
}

/// Moteur factice avec une connexion OpenAI par défaut ; [consent] faux : l'envoi demande d'abord l'accord.
FakeAiEngine engineWithConnection({bool consent = true}) => FakeAiEngine()
  ..providers.add({
    'id': 'openai',
    'display_name': 'OpenAI',
    'base_url': 'https://api.openai.com/v1',
    'base_url_editable': false,
  })
  ..connections.add({
    'id': 'ai-1',
    'provider': 'openai',
    'label': 'Mon OpenAI',
    'base_url': null,
    'default_model': 'm1',
    'is_default': true,
    'has_key': true,
    'consent_at': consent ? 1 : null,
    'models': null,
    'allowed_kinds': <String>[],
  });

/// Assemble le contrôleur ; [demo] vrai : aucun moteur d'IA ne répond, l'historique reste branché.
Rig rig({bool demo = false, bool consent = true}) {
  final engine = engineWithConnection(consent: consent);
  final history = ChatHistoryState(() => AiHistoryAdmin(engine));
  final chat = ChatController(
    admin: () => demo ? null : AiAdmin(engine),
    history: () => history,
    welcome: () => ChatMsg(role: 'ai', text: 'Bienvenue'),
    demoAnswer: (q) => Answer('démo : $q'),
    after: (ms, run) => Timer(Duration.zero, run),
  );
  return Rig(engine, history, chat);
}

/// Lignes de flux d'une réponse complète.
List<Map<String, dynamic>> reply(String text) => [
  {'type': 'started', 'operation': 'op-1'},
  {'type': 'text', 'text': text},
  {'type': 'done', 'reason': 'end_turn'},
];

/// Range une conversation lisible (version 1) dans le faux moteur.
void seedConversation(
  FakeAiEngine engine,
  String id,
  List<ChatMsg> messages, {
  int version = chatPayloadVersion,
  String? payload,
}) {
  engine.history.conversations.add({
    'id': id,
    'title': 'Titre $id',
    'created_at': 1000,
    'updated_at': 1000,
    'payload_version': version,
    'payload': payload ?? encodeChat(messages),
  });
}
