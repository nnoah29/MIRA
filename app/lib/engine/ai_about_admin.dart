//! Requêtes du texte « ce que tu dois savoir de moi » : lecture et enregistrement.

import 'ai_admin.dart';
import 'engine_client.dart';

/// Taille maximale du texte, en caractères, comme le moteur.
const aboutMaxChars = 1000;

/// Texte libre de l'utilisateur, ajouté à la consigne de l'assistant à chaque discussion.
extension AiAboutAdmin on AiAdmin {
  Future<String> _about(String op, [Map<String, dynamic>? data]) async {
    final r = await client.call(req('ai', op, data ?? const {}));
    return '${(r['data'] as Map)['data']}';
  }

  /// Texte enregistré ; vide tant que l'utilisateur n'a rien écrit.
  Future<String> aboutMe() => _about('get_about_me');

  /// Enregistre le texte ; rend ce que le moteur a gardé (rogné). Un texte vide efface le champ.
  Future<String> setAboutMe(String text) =>
      _about('set_about_me', {'text': text});
}
