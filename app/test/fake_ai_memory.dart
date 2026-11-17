import 'package:mira/engine/engine_client.dart';

/// Mémoire d'un moteur factice : même contrat que le moteur, en mémoire, avec un ordre de souvenirs récents d'abord.
class FakeAiMemory {
  final records = <Map<String, dynamic>>[];
  Map<String, dynamic> settings = {
    'enabled': false,
    'embedder': {'kind': 'local'},
  };
  bool modelInstalled = true;
  int stale = 0;
  EngineException? failInstall;
  int _clock = 1000;
  int _ids = 0;

  Map<String, dynamic> _wrap(String type, Object? data) => {
    'type': 'ai',
    'data': {'type': type, 'data': data},
  };

  Map<String, dynamic> _find(Object? id) {
    final at = records.indexWhere((r) => r['id'] == id);
    if (at < 0) throw const EngineException('not_found.ai_memory', '');
    return records[at];
  }

  void _requireEnabled() {
    if (settings['enabled'] != true) {
      throw const EngineException('incompatible_state.ai_memory_disabled', '');
    }
  }

  Map<String, dynamic> _status() => _wrap('memory_status', {
    'settings': settings,
    'model_installed': modelInstalled,
    'count': records.length,
    'stale': stale,
  });

  Map<String, dynamic> _remember(Map<String, dynamic> r) {
    _requireEnabled();
    final row = {
      'id': 'mem-${_ids += 1}',
      'text': r['text'],
      'kind': r['kind'],
      'source': 'user',
      'created_at': _clock += 1,
      'model_id': 'embed',
    };
    records.add(row);
    return _wrap('memory', row);
  }

  Map<String, dynamic> _list(Map<String, dynamic> r) {
    final sorted = [...records]
      ..sort((a, b) => (b['created_at'] as int) - (a['created_at'] as int));
    final from = r['after'] == null
        ? 0
        : sorted.indexWhere((x) => x['id'] == r['after']) + 1;
    return _wrap(
      'memories',
      sorted.skip(from).take(r['limit'] as int).toList(),
    );
  }

  Map<String, dynamic> _search(Map<String, dynamic> r) {
    _requireEnabled();
    final word = '${r['query']}'.toLowerCase();
    final hits = records.where(
      (x) => '${x['text']}'.toLowerCase().contains(word),
    );
    return _wrap('memory_matches', [
      for (final x in hits) {'record': x, 'score_permille': 900},
    ]);
  }

  Map<String, dynamic> _update(Map<String, dynamic> r) {
    _requireEnabled();
    return _wrap('memory', _find(r['id'])..['text'] = r['text']);
  }

  Map<String, dynamic> _forgetAll() {
    final n = records.length;
    records.clear();
    return _wrap('memory_forgotten', n);
  }

  Map<String, dynamic> _setSettings(Map<String, dynamic> r) {
    settings = (r['settings'] as Map).cast<String, dynamic>();
    return _status();
  }

  Map<String, dynamic> _forget(Map<String, dynamic> r) {
    records.remove(_find(r['id']));
    return _wrap('done', null);
  }

  Map<String, dynamic> _reindex() {
    final done = stale;
    stale = 0;
    return _wrap('memory_reindexed', {'done': done, 'failed': 0});
  }

  Map<String, dynamic> _install() {
    final fail = failInstall;
    if (fail != null) throw fail;
    modelInstalled = true;
    return {
      'type': 'operation',
      'data': {
        'id': 'op-local',
        'kind': 'ai_memory_install',
        'status': {'type': 'running', 'done': 0, 'total': 100},
      },
    };
  }

  /// Réponse à une requête de la mémoire, ou `null` si l'opération n'en fait pas partie.
  Map<String, dynamic>? answer(Map<String, dynamic> r) => switch (r['op']) {
    'get_memory_status' => _status(),
    'set_memory_settings' => _setSettings(r),
    'list_memories' => _list(r),
    'search_memories' => _search(r),
    'remember_memory' => _remember(r),
    'update_memory' => _update(r),
    'forget_memory' => _forget(r),
    'forget_all_memories' => _forgetAll(),
    'reindex_memories' => _reindex(),
    'install_memory_model' => _install(),
    _ => null,
  };
}
