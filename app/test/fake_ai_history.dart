import 'package:mira/engine/engine_client.dart';

/// Historique des conversations d'un moteur factice : même contrat que le moteur, en mémoire.
class FakeAiHistory {
  final conversations = <Map<String, dynamic>>[];
  bool enabled = true;
  int _clock = 1000;

  Map<String, dynamic> _wrap(String type, Object? data) => {
    'type': 'ai',
    'data': {'type': type, 'data': data},
  };

  Map<String, dynamic> _meta(Map<String, dynamic> row) => {
    for (final key in ['id', 'title', 'created_at', 'updated_at'])
      key: row[key],
    'payload_version': row['payload_version'],
  };

  Map<String, dynamic> _find(Object? id) {
    final at = conversations.indexWhere((c) => c['id'] == id);
    if (at < 0) throw const EngineException('not_found.ai_conversation', '');
    return conversations[at];
  }

  Map<String, dynamic> _save(Map<String, dynamic> r) {
    if (!enabled) {
      throw const EngineException('incompatible_state.ai_history_disabled', '');
    }
    final now = _clock += 1;
    final at = conversations.indexWhere((c) => c['id'] == r['id']);
    final row = {
      'id': r['id'],
      'title': r['title'],
      'created_at': at < 0 ? now : conversations[at]['created_at'],
      'updated_at': now,
      'payload_version': r['payload_version'],
      'payload': r['payload'],
    };
    at < 0 ? conversations.add(row) : conversations[at] = row;
    return _wrap('conversation_meta', _meta(row));
  }

  Map<String, dynamic> _list(Map<String, dynamic> r) {
    final sorted = [...conversations]
      ..sort((a, b) => (b['updated_at'] as int) - (a['updated_at'] as int));
    final from = r['before'] == null
        ? 0
        : sorted.indexWhere((c) => c['id'] == r['before']) + 1;
    final page = sorted.skip(from).take(r['limit'] as int);
    return _wrap('conversations', [for (final c in page) _meta(c)]);
  }

  Map<String, dynamic> _rename(Map<String, dynamic> r) {
    final row = _find(r['id'])..['title'] = r['title'];
    return _wrap('conversation_meta', _meta(row));
  }

  /// Réponse à une requête de l'historique, ou `null` si l'opération n'en fait pas partie.
  Map<String, dynamic>? answer(Map<String, dynamic> r) => switch (r['op']) {
    'save_ai_conversation' => _save(r),
    'list_ai_conversations' => _list(r),
    'get_ai_conversation' => _wrap('conversation', _find(r['id'])),
    'rename_ai_conversation' => _rename(r),
    'delete_ai_conversation' => _delete(r),
    'clear_ai_conversations' => _clear(),
    'get_ai_history_enabled' => _wrap('history_enabled', enabled),
    'set_ai_history_enabled' => _setEnabled(r),
    _ => null,
  };

  Map<String, dynamic> _delete(Map<String, dynamic> r) {
    conversations.remove(_find(r['id']));
    return _wrap('done', null);
  }

  Map<String, dynamic> _clear() {
    conversations.clear();
    return _wrap('done', null);
  }

  Map<String, dynamic> _setEnabled(Map<String, dynamic> r) {
    enabled = r['enabled'] == true;
    return _wrap('history_enabled', enabled);
  }
}
