import 'dart:convert';

/// Erreur structurée renvoyée par le moteur (catégorie.code + message, sans secret).
class EngineException implements Exception {
  final String code;
  final String message;
  const EngineException(this.code, this.message);

  @override
  String toString() => 'EngineException($code): $message';
}

/// Accès au moteur : une requête du contrat (JSON) → les données de la réponse.
abstract class EngineClient {
  /// Exécute une requête et rend l'objet `Response` (`{type, data}`).
  Future<Map<String, dynamic>> call(Map<String, dynamic> request);

  /// Événements du moteur (objets `Event`) à partir d'un numéro.
  Stream<Map<String, dynamic>> events({int? after});

  /// HTML d'origine d'un message, tel que reçu (`null` : pas de partie HTML).
  Future<String?> originalHtml(String messageId);

  /// Ferme le moteur.
  Future<void> close();
}

/// Ouvre l'enveloppe `{ok, data | error}` ; lève [EngineException] si `ok` est faux.
Map<String, dynamic> openEnvelope(String json) {
  final env = jsonDecode(json) as Map<String, dynamic>;
  if (env['ok'] == true) {
    final data = env['data'];
    return data is Map<String, dynamic> ? data : <String, dynamic>{};
  }
  final err = (env['error'] as Map?)?.cast<String, dynamic>() ?? const {};
  throw EngineException(
    '${err['code'] ?? 'internal.bug'}',
    '${err['message'] ?? 'erreur du moteur'}',
  );
}

/// Requête `{domain, request:{op, …}}` du contrat.
Map<String, dynamic> req(
  String domain,
  String op, [
  Map<String, dynamic> args = const {},
]) => {
  'domain': domain,
  'request': {'op': op, ...args},
};
