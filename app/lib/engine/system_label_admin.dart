import 'engine_client.dart';

/// Sortes de libellés système posées par le moteur sur les conversations.
const systemKinds = {'needs', 'waiting', 'mention'};

/// Libellés système du moteur : poses par conversation et choix manuel de l'utilisateur.
class SystemLabelAdmin {
  final EngineClient client;
  const SystemLabelAdmin(this.client);

  Future<Object?> _send(String op, [Map<String, dynamic>? data]) async {
    final r = await client.call(req('system_label', op, data ?? const {}));
    return (r['data'] as Map)['data'];
  }

  /// Sortes posées sur chaque conversation, par identifiant de conversation.
  Future<Map<String, Set<String>>> assignments() async {
    final items = await _send('list_system_label_assignments') as List;
    return {
      for (final v in items)
        '${(v as Map)['thread']}': {for (final k in v['kinds'] as List) '$k'},
    };
  }

  /// Pose (`on`) ou retire une sorte sur des conversations ; valable jusqu'au prochain message du fil.
  Future<void> set(String kind, List<String> threads, bool on) =>
      _send('set_system_label', {'kind': kind, 'threads': threads, 'on': on});
}
