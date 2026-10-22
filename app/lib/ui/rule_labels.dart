import '../data/rule_models.dart';
import '../i18n/i18n.dart';

const _kindLabels = <String, String>{
  'sender': 'Expéditeur',
  'recipient': 'Destinataire',
  'me_in_to': 'Je suis en destinataire direct',
  'me_in_cc_only': 'Je suis seulement en copie',
  'recipient_count': 'Nombre de destinataires',
  'reply_to': 'Adresse de réponse',
  'subject': 'Objet',
  'preview': 'Aperçu du texte',
  'received_before': 'Reçu avant',
  'received_after': 'Reçu après',
  'older_than_days': 'Plus ancien que (jours)',
  'size_compare': 'Taille (octets)',
  'has_attachment': 'A une pièce jointe',
  'attachment_name': 'Nom de pièce jointe',
  'attachment_type': 'Type de pièce jointe',
  'is_read': 'Est lu',
  'is_flagged': 'Est favori',
  'is_answered': 'A reçu une réponse',
  'is_forwarded': 'A été transféré',
  'is_newsletter': 'Est une lettre d\'information',
  'is_reply': 'Est une réponse',
  'in_existing_thread': 'Dans une conversation existante',
  'in_mailbox': 'Est dans la boîte',
  'from_account': 'Reçu sur le compte',
  'mark_read': 'Marquer comme lu',
  'mark_flagged': 'Marquer comme favori',
  'move_to': 'Déplacer vers',
  'add_to_mailbox': 'Mettre un libellé du compte',
  'remove_from_mailbox': 'Retirer un libellé du compte',
  'has_local_label': 'A le libellé local',
  'add_local_label': 'Mettre un libellé local',
  'remove_local_label': 'Retirer un libellé local',
  'archive': 'Archiver',
  'trash': 'Mettre à la corbeille',
  'mark_spam': 'Marquer comme spam',
  'suppress_notification': 'Ne pas notifier',
  'stop_processing': 'Arrêter le traitement',
};

/// Libellé lisible d'une variante du catalogue (l'étiquette brute si inconnue).
String ruleKindLabel(String kind) {
  final fr = _kindLabels[kind];
  return fr == null ? kind : t(fr);
}

/// Valeur de départ d'un paramètre du moteur, selon son nom.
dynamic defaultParam(String field) => switch (field) {
  'field' => 'address',
  'which' => 'to',
  'cmp' => 'greater',
  'n' || 'days' => 1,
  'bytes' => 1000,
  'at' => 0,
  'read' || 'flagged' || 'spam' => true,
  'matcher' => {'op': 'contains', 'values': <String>[]},
  _ => '',
};

/// Test ou action neuve pour une variante du catalogue, avec ses paramètres par défaut.
Map<String, dynamic> defaultsFor(CatalogEntry entry) => {
  'type': entry.kind,
  for (final f in entry.fields) f: defaultParam(f),
};
