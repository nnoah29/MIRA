/// Ce que l'app fait d'un événement du moteur.
enum EventEffect {
  /// Relit le compte concerné (regroupé par rafale).
  reload,

  /// Met à jour le compteur de non-lus de la boîte, sans relire le compte.
  counters,

  /// Met à jour l'état de synchronisation du compte.
  sync,

  /// Relit la liste des comptes.
  accounts,

  /// Retient le verdict des règles sur une arrivée.
  verdict,

  /// Signale un changement des règles.
  rules,

  /// Signale un changement des libellés locaux.
  localLabels,

  /// Signale un changement des réglages des libellés.
  labelSettings,

  /// Signale un changement des réglages du moteur.
  settings,

  /// Signale un changement des modèles de messages.
  templates,
}

const _none = <EventEffect>{};
const _reload = {EventEffect.reload};
const _accounts = {EventEffect.accounts};
const _sync = {EventEffect.sync};

/// Traitement de chaque événement du moteur, par son `type` ; un test vérifie que la table nomme
/// exactement les variantes de `EventKind`, donc qu'aucun nouvel événement n'est oublié.
const Map<String, Set<EventEffect>> eventRoutes = {
  'engine_started': _none,
  'engine_stopping': _none,
  'engine_fatal_error': _none,
  'account_added': _accounts,
  'account_updated': _accounts,
  'account_deleted': _accounts,
  'account_state_changed': _accounts,
  'sync_started': _sync,
  'sync_progress': _sync,
  'sync_finished': {EventEffect.sync, EventEffect.reload},
  'sync_failed': _sync,
  'went_offline': _sync,
  'back_online': _sync,
  'mailbox_created': _reload,
  'mailbox_updated': _reload,
  'mailbox_deleted': _reload,
  'mailbox_counters_updated': {EventEffect.counters},
  'new_message_received': {EventEffect.reload, EventEffect.verdict},
  'message_updated': _reload,
  'message_deleted': _reload,
  'body_available': _none,
  'attachment_progress': _none,
  'attachment_ready': _none,
  'thread_updated': _reload,
  'action_queued': _none,
  'action_applied': _none,
  'action_failed': _reload,
  'action_cancelled': _reload,
  'send_scheduled': _reload,
  'send_departed': {EventEffect.sync, EventEffect.reload},
  'send_failed': _reload,
  'sent_copy_stored': _reload,
  'send_uncertain': _reload,
  'quota_full': _sync,
  'certificate_needs_approval': _none,
  'rate_limited': _sync,
  'mailbox_resynchronized': _reload,
  'diagnostic': _none,
  'operation_progress': _none,
  'operation_finished': _none,
  'operation_failed': _none,
  'operation_cancelled': _none,
  'rule_changed': {EventEffect.rules},
  'rule_applied': _none,
  'local_labels_changed': {EventEffect.localLabels},
  'label_settings_changed': {EventEffect.labelSettings},
  'settings_changed': {EventEffect.settings},
  'templates_changed': {EventEffect.templates},
};

/// Effets d'un événement ; un type inconnu (moteur plus récent que l'app) n'en a aucun.
Set<EventEffect> effectsOf(String type) => eventRoutes[type] ?? _none;
