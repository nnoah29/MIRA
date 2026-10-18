import 'engine_client.dart';

/// Volumes du miroir local du moteur.
class StorageStats {
  final int messages, attachments, cached, rawBytes, compressedBytes;
  const StorageStats({
    required this.messages,
    required this.attachments,
    required this.cached,
    required this.rawBytes,
    required this.compressedBytes,
  });
}

/// Entretien du moteur : volumes, intégrité, purge du cache, index de recherche, exports.
class MaintenanceAdmin {
  final EngineClient client;
  const MaintenanceAdmin(this.client);

  Future<Map<String, dynamic>> _data(
    String op, [
    Map<String, dynamic>? a,
  ]) async {
    final raw = await client.call(req('maintenance', op, a ?? const {}));
    final d = raw['data'];
    return d is Map ? d.cast<String, dynamic>() : const {};
  }

  /// Mesure ce que le moteur garde sur disque.
  Future<StorageStats> stats() async {
    final d = await _data('get_storage_stats');
    int n(String k) => (d[k] as num?)?.toInt() ?? 0;
    return StorageStats(
      messages: n('messages'),
      attachments: n('attachments'),
      cached: n('attachments_cached'),
      rawBytes: n('raw_bytes'),
      compressedBytes: n('compressed_bytes'),
    );
  }

  /// Vrai si la base est saine.
  Future<bool> checkIntegrity() async =>
      (await _data('check_integrity'))['ok'] == true;

  /// Supprime les pièces jointes gardées en cache (elles seront retéléchargées au besoin).
  Future<void> purgeAttachments() => _data('purge_attachment_cache');

  /// Reconstruit l'index de recherche local.
  Future<void> rebuildIndex() => _data('rebuild_search_index');

  /// Écrit un export (`export_diagnostic` ou `export_profile`) ; rend sa taille en octets.
  Future<int> export(String op, String destination) async =>
      ((await _data(op, {'destination': destination}))['bytes'] as num?)
          ?.toInt() ??
      0;
}
