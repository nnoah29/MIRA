import 'ai_history_models.dart';
import 'engine_client.dart';

export 'ai_history_models.dart';

/// Historique des conversations avec l'assistant IA, porté par le moteur.
class AiHistoryAdmin {
  final EngineClient client;
  const AiHistoryAdmin(this.client);

  Future<Object?> _send(String op, [Map<String, dynamic>? data]) async {
    final r = await client.call(req('ai', op, data ?? const {}));
    return (r['data'] as Map)['data'];
  }

  AiConversationMeta _meta(Object? raw) =>
      AiConversationMeta.fromJson((raw as Map).cast<String, dynamic>());

  /// Conversations enregistrées, les plus récentes d'abord ; [before] reprend après cette conversation.
  Future<List<AiConversationMeta>> list({
    int limit = 50,
    String? before,
  }) async {
    final items = await _send('list_ai_conversations', {
      'limit': limit,
      'before': before,
    });
    return [for (final v in items as List) _meta(v)];
  }

  /// Une conversation avec son contenu.
  Future<AiConversationDoc> get(String id) async => AiConversationDoc.fromJson(
    ((await _send('get_ai_conversation', {'id': id})) as Map)
        .cast<String, dynamic>(),
  );

  /// Crée ou remplace la conversation [id] ; le moteur date et garde les plus récentes.
  Future<AiConversationMeta> save({
    required String id,
    required String title,
    required int payloadVersion,
    required String payload,
  }) async => _meta(
    await _send('save_ai_conversation', {
      'id': id,
      'title': title,
      'payload_version': payloadVersion,
      'payload': payload,
    }),
  );

  /// Change le titre sans changer le rang de la conversation dans la liste.
  Future<AiConversationMeta> rename(String id, String title) async =>
      _meta(await _send('rename_ai_conversation', {'id': id, 'title': title}));

  /// Supprime une conversation.
  Future<void> delete(String id) => _send('delete_ai_conversation', {'id': id});

  /// Supprime toutes les conversations.
  Future<void> clearAll() => _send('clear_ai_conversations');

  /// Vrai quand le moteur enregistre les conversations.
  Future<bool> enabled() async =>
      (await _send('get_ai_history_enabled')) == true;

  /// Active ou coupe l'historique et rend la valeur confirmée par le moteur.
  Future<bool> setEnabled(bool enabled) async =>
      (await _send('set_ai_history_enabled', {'enabled': enabled})) == true;
}
