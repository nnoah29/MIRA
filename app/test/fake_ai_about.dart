import 'package:mira/engine/engine_client.dart';

/// Texte « ce que tu dois savoir de moi » d'un moteur factice : même contrat, rogné, borné à 1 000 caractères.
class FakeAiAbout {
  String text = '';

  Map<String, dynamic> _wrap(String value) => {
    'type': 'ai',
    'data': {'type': 'about_me', 'data': value},
  };

  Map<String, dynamic> _set(Map<String, dynamic> r) {
    final next = '${r['text']}'.trim();
    if (next.runes.length > 1000) {
      throw const EngineException('invalid_input.ai_about_text', '');
    }
    text = next;
    return _wrap(text);
  }

  /// Réponse à une requête du champ, ou `null` si l'opération n'en fait pas partie.
  Map<String, dynamic>? answer(Map<String, dynamic> r) => switch (r['op']) {
    'get_about_me' => _wrap(text),
    'set_about_me' => _set(r),
    _ => null,
  };
}
