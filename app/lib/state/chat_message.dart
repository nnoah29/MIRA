//! Message du fil de l'assistant : auteur, texte (affiché au fur et à mesure), brouillon et références.

/// État d'un appel d'outil de l'IA dans la conversation.
enum ToolState { running, ok, denied, error }

/// Un appel d'outil montré sous la réponse : ce que l'IA a consulté, et comment cela s'est passé.
class ToolActivity {
  final String id;
  final String name;
  final String summary;
  ToolState state;
  int results;

  /// Type de contenu que l'utilisateur n'a pas autorisé, quand l'appel a été refusé.
  String? deniedKind;
  ToolActivity({
    required this.id,
    required this.name,
    this.summary = '',
    this.state = ToolState.running,
    this.results = 0,
    this.deniedKind,
  });
}

class ChatMsg {
  final String role; // 'me' | 'ai'
  bool done;
  String shown;
  String text;
  List<String>? draft;
  List<String>? refs;

  /// Appels d'outils de l'IA pendant cette réponse, dans l'ordre.
  final List<ToolActivity> tools = [];

  /// Identifiants des messages consultés par les outils, sans doublon.
  final List<String> sources = [];

  /// Texte venu du moteur d'IA (non traduisible) ; faux pour la démonstration et les notices.
  final bool fromEngine;

  /// Réponse en échec ou vide : jamais renvoyée au modèle comme un tour de parole.
  bool failed;

  /// Remarque affichée sous la réponse (arrêt, troncature) ; ne fait pas partie du texte renvoyé au modèle.
  String? note;

  /// Nombre de données sensibles masquées avant l'envoi (0 : aucune ou masquage coupé).
  int maskedCount;
  ChatMsg({
    required this.role,
    this.done = true,
    this.shown = '',
    required this.text,
    this.draft,
    this.refs,
    this.fromEngine = false,
    this.failed = false,
    this.note,
    this.maskedCount = 0,
  });
}
