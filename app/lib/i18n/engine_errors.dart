import '../engine/engine_client.dart';
import 'i18n.dart';

/// Texte français de chaque code d'erreur du moteur (clé de traduction) ; un test vérifie qu'aucun code n'y manque.
const engineErrorTexts = <String, String>{
  // i18n:keys-begin
  'authentication.ai_key_rejected':
      'Clé refusée par le service d’IA : vérifie-la',
  'authentication.app_password_required':
      'Ce service exige un mot de passe d’application',
  'authentication.oauth_cancelled': 'Connexion annulée',
  'authentication.oauth_state_mismatch':
      'Réponse de connexion inattendue : recommence',
  'authentication.rejected': 'Identifiants refusés par le serveur',
  'authentication.token_expired': 'Session expirée : reconnecte le compte',
  'authentication.token_revoked':
      'Accès révoqué par le service : reconnecte le compte',
  'cancelled.by_user': 'Opération annulée',
  'incompatible_state.account_deleting':
      'Le compte est en cours de suppression',
  'incompatible_state.account_paused': 'Le compte est en pause',
  'incompatible_state.action_not_cancellable':
      'Cette action ne peut plus être annulée',
  'incompatible_state.action_not_retryable':
      'Seule une action échouée peut être relancée',
  'incompatible_state.ai_consent_required':
      'Autorise d’abord l’envoi de texte à ce service d’IA',
  'incompatible_state.ai_key_missing':
      'Aucune clé enregistrée pour ce service d’IA',
  'incompatible_state.ai_model_missing': 'Choisis d’abord un modèle d’IA',
  'incompatible_state.body_unavailable_offline':
      'Corps du message indisponible hors ligne',
  'incompatible_state.draft_not_editable': 'Ce brouillon n’est plus modifiable',
  'incompatible_state.mailbox_changed':
      'La boîte a changé sur le serveur : resynchronisation nécessaire',
  'incompatible_state.multiple_labels_unsupported':
      'Ce fournisseur n’accepte qu’un libellé à la fois',
  'incompatible_state.profile_locked':
      'Le profil est déjà utilisé par une autre instance de MIRA',
  'incompatible_state.sync_disabled':
      'La synchronisation en arrière-plan est désactivée',
  'incompatible_state.system_mailbox':
      'Opération interdite sur une boîte système',
  'internal.bug': 'Erreur interne de MIRA',
  'invalid_input.address': 'Adresse e-mail invalide',
  'invalid_input.ai_input_too_large':
      'Le texte est trop long pour le service d’IA',
  'invalid_input.ai_key_invalid': 'Clé du service d’IA invalide',
  'invalid_input.ai_url_invalid': 'Adresse du service d’IA invalide',
  'invalid_input.contract_version':
      'Version incompatible entre l’app et le moteur',
  'invalid_input.cursor': 'Page demandée invalide',
  'invalid_input.export_kind_mismatch':
      'Ce fichier n’est pas un export de règles',
  'invalid_input.export_too_large':
      'Fichier d’export trop gros (2 Mo et 500 éléments au plus)',
  'invalid_input.export_version_unsupported':
      'Version du format d’export non prise en charge',
  'invalid_input.idempotency_conflict':
      'Cette opération a déjà été faite autrement',
  'invalid_input.missing_parameter': 'Un champ obligatoire est vide',
  'invalid_input.parameter': 'Valeur invalide',
  'invalid_input.rule_duplicate_in_file':
      'Cette règle apparaît déjà plus haut dans le fichier',
  'invalid_input.template_name_empty': 'Le nom du modèle est vide',
  'invalid_input.template_name_taken': 'Un modèle porte déjà ce nom',
  'invalid_input.template_too_long':
      'Le nom, l’objet ou le texte du modèle est trop long',
  'invalid_input.yaml_syntax': 'Fichier YAML invalide',
  'local_storage.corrupted': 'Base de données locale corrompue',
  'local_storage.disk_full': 'Disque plein',
  'local_storage.io': 'Erreur de lecture ou d’écriture sur le disque',
  'local_storage.migration_required':
      'Mise à jour de la base de données nécessaire',
  'network.connection_lost': 'Connexion perdue',
  'network.offline': 'Aucune connexion réseau',
  'network.timeout': 'Le serveur met trop de temps à répondre',
  'network.unreachable': 'Serveur injoignable',
  'not_found.account': 'Compte introuvable',
  'not_found.action': 'Action introuvable',
  'not_found.ai_connection': 'Connexion d’IA introuvable',
  'not_found.ai_model': 'Modèle d’IA introuvable',
  'not_found.ai_provider': 'Service d’IA inconnu',
  'not_found.attachment': 'Pièce jointe introuvable',
  'not_found.draft': 'Brouillon introuvable',
  'not_found.identity': 'Identité d’envoi introuvable',
  'not_found.label': 'Libellé introuvable',
  'not_found.local_label': 'Libellé local introuvable',
  'not_found.mailbox': 'Boîte introuvable',
  'not_found.message': 'Message introuvable',
  'not_found.operation': 'Opération introuvable',
  'not_found.provider': 'Fournisseur inconnu',
  'not_found.rule': 'Règle introuvable',
  'not_found.template': 'Modèle introuvable',
  'not_found.thread': 'Conversation introuvable',
  'secrets.keyring_locked':
      'Trousseau de mots de passe verrouillé : déverrouille-le puis réessaie',
  'secrets.keyring_unavailable': 'Trousseau de mots de passe indisponible',
  'secrets.secret_missing':
      'Mot de passe introuvable dans le trousseau : saisis-le à nouveau',
  'server.ai_bad_response': 'Réponse inattendue du service d’IA',
  'server.ai_rate_limited':
      'Le service d’IA limite le nombre de requêtes : réessaie plus tard',
  'server.ai_unavailable': 'Service d’IA indisponible',
  'server.extension_missing':
      'Le serveur ne prend pas en charge cette fonction',
  'server.mailbox_missing': 'Boîte absente du serveur',
  'server.message_too_large': 'Message trop gros pour le serveur',
  'server.protocol': 'Réponse inattendue du serveur',
  'server.quota_exceeded': 'Boîte pleine sur le serveur',
  'server.rate_limited':
      'Le serveur limite le nombre de requêtes : réessaie plus tard',
  'server.rejected': 'Commande refusée par le serveur',
  'tls.handshake': 'Connexion sécurisée impossible avec le serveur',
  'tls.invalid_certificate': 'Certificat du serveur invalide ou non approuvé',
  'tls.plaintext_refused': 'Connexion non chiffrée refusée',
  // i18n:keys-end
};

/// Texte d'une erreur dans la langue active : le message de l'app s'il est déjà traduit, sinon celui du code.
/// Un code inconnu (moteur plus récent que l'app) donne le code, jamais le message français du moteur.
String engineErrorText(Object error) {
  if (error is! EngineException) {
    return t('Erreur inattendue : {0}', ['$error']);
  }
  if (error.localized) return error.message;
  final text = engineErrorTexts[error.code];
  return text == null ? t('Erreur : {0}', [error.code]) : t(text);
}
