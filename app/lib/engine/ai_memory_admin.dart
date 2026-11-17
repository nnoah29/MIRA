//! Requêtes de la mémoire de l'IA : réglages, installation du modèle, souvenirs (liste, recherche, ajout, modification, oubli).

import 'ai_admin.dart';
import 'ai_memory_models.dart';
import 'engine_client.dart';

export 'ai_memory_models.dart';

/// Mémoire du moteur ; oublier et lister marchent même désactivée, retenir et chercher non.
extension AiMemoryAdmin on AiAdmin {
  Future<Object?> _ask(String op, [Map<String, dynamic>? data]) async {
    final r = await client.call(req('ai', op, data ?? const {}));
    return (r['data'] as Map)['data'];
  }

  MemoryItem _item(Object? raw) =>
      MemoryItem.fromJson((raw as Map).cast<String, dynamic>());

  /// Réglages, modèle local présent, compteurs ; lecture seule.
  Future<MemoryStatus> memoryStatus() async => MemoryStatus.fromJson(
    ((await _ask('get_memory_status')) as Map).cast<String, dynamic>(),
  );

  /// Enregistre les réglages et rend le statut qui en résulte.
  Future<MemoryStatus> setMemorySettings(MemorySettings settings) async =>
      MemoryStatus.fromJson(
        ((await _ask('set_memory_settings', {
          'settings': settings.toJson(),
        })) as Map).cast<String, dynamic>(),
      );

  /// Lance l'installation du modèle d'embeddings ; rend l'identifiant de l'opération qui la porte.
  Future<String> installMemoryModel() async {
    final r = await client.call(req('ai', 'install_memory_model', const {}));
    return '${(r['data'] as Map)['id']}';
  }

  /// Souvenirs, les plus récents d'abord ; [after] reprend après ce souvenir.
  Future<List<MemoryItem>> listMemories({int limit = 50, String? after}) async {
    final items = await _ask('list_memories', {'limit': limit, 'after': after});
    return [for (final v in items as List) _item(v)];
  }

  /// Souvenirs proches du sens de [query], les plus proches d'abord.
  Future<List<MemoryHit>> searchMemories(String query, {int limit = 5}) async {
    final items = await _ask('search_memories', {
      'query': query,
      'limit': limit,
    });
    return [
      for (final v in items as List)
        MemoryHit.fromJson((v as Map).cast<String, dynamic>()),
    ];
  }

  /// Retient un texte à la demande de l'utilisateur ; [kind] vaut `fact`, `preference` ou `action`.
  Future<MemoryItem> remember(String text, {String kind = 'fact'}) async =>
      _item(await _ask('remember_memory', {'text': text, 'kind': kind}));

  /// Remplace le texte d'un souvenir.
  Future<MemoryItem> updateMemory(String id, String text) async =>
      _item(await _ask('update_memory', {'id': id, 'text': text}));

  /// Oublie un souvenir.
  Future<void> forgetMemory(String id) => _ask('forget_memory', {'id': id});

  /// Oublie tous les souvenirs ; rend leur nombre.
  Future<int> forgetAllMemories() async =>
      ((await _ask('forget_all_memories')) as num).toInt();

  /// Recalcule les souvenirs périmés avec le modèle courant.
  Future<MemoryReindex> reindexMemories() async {
    final r = ((await _ask('reindex_memories')) as Map).cast<String, dynamic>();
    return MemoryReindex(
      (r['done'] as num).toInt(),
      (r['failed'] as num).toInt(),
    );
  }
}
