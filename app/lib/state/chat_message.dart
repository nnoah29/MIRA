//! Message du fil de l'assistant : auteur, texte (affiché au fur et à mesure), brouillon et références.

class ChatMsg {
  final String role; // 'me' | 'ai'
  bool done;
  String shown;
  String text;
  List<String>? draft;
  List<String>? refs;

  /// Texte venu du moteur d'IA (non traduisible) ; faux pour la démonstration et les notices.
  final bool fromEngine;

  /// Réponse en échec ou vide : jamais renvoyée au modèle comme un tour de parole.
  bool failed;
  ChatMsg({
    required this.role,
    this.done = true,
    this.shown = '',
    required this.text,
    this.draft,
    this.refs,
    this.fromEngine = false,
    this.failed = false,
  });
}
