//! Libellé d'un appel d'outil de l'IA : ce qu'elle consulte, puis l'issue de l'appel.

import '../i18n/i18n.dart';
import '../state/chat_message.dart';

/// Ce que l'outil fait, en un groupe nominal ; la requête saisie par l'IA est citée pour la recherche.
String toolAction(ToolActivity tool) => switch (tool.name) {
  'search_messages' when tool.summary.isNotEmpty => t('Recherche « {0} »', [
    tool.summary,
  ]),
  'search_messages' => t('Recherche dans la boîte'),
  'list_messages' => t('Liste des messages'),
  'get_message' => t('Lecture d’un message'),
  'get_thread' => t('Lecture d’un fil'),
  'list_mailboxes' => t('Boîtes consultées'),
  _ => tool.name,
};

/// Libellé complet : l'action suivie de son issue (nombre de résultats, refus ou erreur).
String toolLabel(ToolActivity tool) {
  final action = toolAction(tool);
  return switch (tool.state) {
    ToolState.running => action,
    ToolState.ok => '$action · ${tn(tool.results, 'résultat', 'résultats')}',
    ToolState.denied => '$action · ${t('refusé')}',
    ToolState.error => '$action · ${t('erreur')}',
  };
}
