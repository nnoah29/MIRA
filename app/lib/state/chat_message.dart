//! Message du fil de l'assistant : auteur, texte (affiché au fur et à mesure), brouillon et références.

import '../engine/ai_models.dart';

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

/// Action sur la boîte proposée par l'IA et son état : `pending`, `done` (confirmée et appliquée) ou `dismissed` (ignorée).
class ProposedAction {
  final AiActionProposal proposal;
  String status;
  ProposedAction(this.proposal, {this.status = 'pending'});
}

/// Proposition de l'IA sur la mémoire et son état : `pending`, `done` (confirmée et appliquée) ou `dismissed` (ignorée).
class ProposedMemory {
  final AiMemoryProposal proposal;
  String status;

  /// Texte du souvenir à oublier, retrouvé par son identifiant ; absent tant qu'il n'est pas lu.
  String? target;

  /// Vrai quand le souvenir à oublier n'existe plus : la proposition ne peut qu'être ignorée.
  bool missing;

  /// Dernier échec d'une confirmation, texte déjà traduit ; non enregistré avec la conversation.
  String? error;
  ProposedMemory(
    this.proposal, {
    this.status = 'pending',
    this.target,
    this.missing = false,
  });
}

/// Pièce jointe d'un message envoyé : `kind` vaut `mail` ou `file`, `label` est ce qu'on lit sur la vignette.
class ChatAttachmentRef {
  final String kind, label;
  const ChatAttachmentRef(this.kind, this.label);
}

class ChatMsg {
  final String role; // 'me' | 'ai'
  bool done;
  String shown;
  String text;
  List<String>? draft;
  List<String>? refs;

  /// Mails et fichiers joints à ce message de l'utilisateur, montrés au-dessus du texte.
  final List<ChatAttachmentRef> attachments;

  /// Appels d'outils de l'IA pendant cette réponse, dans l'ordre.
  final List<ToolActivity> tools = [];

  /// Identifiants des messages consultés par les outils, sans doublon.
  final List<String> sources = [];

  /// Brouillons proposés par l'IA pendant cette réponse, un par appel d'outil.
  final List<AiDraftProposal> drafts = [];

  /// Listes de cartes de messages montrées par l'IA pendant cette réponse, une par appel d'outil.
  final List<AiMailCards> cards = [];

  /// Actions sur la boîte proposées par l'IA pendant cette réponse, une par appel d'outil.
  final List<ProposedAction> actions = [];

  /// Propositions de mémoire (retenir ou oublier) faites par l'IA pendant cette réponse, une par appel d'outil.
  final List<ProposedMemory> memories = [];

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
    this.attachments = const [],
  });
}
