//! Fichier joint au prochain message de l'assistant : nom et texte extrait.

/// Fichier dont le texte a été lu ; `id` distingue deux fichiers de même nom.
class ChatAttachedFile {
  final String id, name, text;
  const ChatAttachedFile({
    required this.id,
    required this.name,
    required this.text,
  });
}

/// Raison pour laquelle un fichier ne peut pas être joint.
enum ChatFileProblem { tooLarge, unreadable, empty }

/// Échec de lecture d'un fichier à joindre.
class ChatFileException implements Exception {
  final ChatFileProblem problem;
  const ChatFileException(this.problem);
}
