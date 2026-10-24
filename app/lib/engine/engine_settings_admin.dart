import 'engine_client.dart';

/// Réglages que le moteur applique lui-même : source de vérité unique, lue et remplacée par l'app.
class EngineSettingsAdmin {
  final EngineClient client;
  EngineSettingsAdmin(this.client);

  Future<Map<String, dynamic>> _send(
    String op, [
    Map<String, dynamic>? data,
  ]) async {
    final r = await client.call(req('settings', op, data ?? const {}));
    return ((r['data'] as Map)['data'] as Map).cast<String, dynamic>();
  }

  /// Relevé périodique en secondes, `null` : délai du fournisseur.
  Future<int?> pollSecs() async =>
      (await _send('get_settings'))['poll_secs'] as int?;

  /// Remplace le relevé périodique ; le moteur l'applique tout de suite.
  Future<void> setPollSecs(int? secs) => _send('set_settings', {
    'settings': {'poll_secs': secs},
  });
}
