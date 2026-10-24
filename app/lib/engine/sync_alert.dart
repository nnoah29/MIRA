import '../data/models.dart';
import '../i18n/i18n.dart';
import 'sync_state.dart';

/// Gravité d'un problème de synchronisation : plus petit = plus urgent.
int _rank(SyncState st) => switch (st.failed) {
  'tls.invalid_certificate' => 0,
  'server.quota_exceeded' => 1,
  'server.rate_limited' => 3,
  null => 4,
  _ => 2,
};

/// Message du bandeau pour un compte qui ne se synchronise pas normalement.
String syncAlertMessage(String email, SyncState st) => switch (st.failed) {
  'tls.invalid_certificate' => t(
    'Certificat du serveur de {0} non approuvé : synchronisation suspendue',
    [email],
  ),
  'server.quota_exceeded' => t(
    'Boîte pleine : {0}. Libère de la place sur le serveur',
    [email],
  ),
  'server.rate_limited' => t(
    '{0} : le fournisseur limite les requêtes, nouvel essai plus tard',
    [email],
  ),
  null => t('Hors ligne : {0}', [email]),
  final code => t('Synchronisation de {0} impossible ({1})', [email, code]),
};

/// Le compte le plus urgent parmi ceux en difficulté (hors ligne ou en échec), ou `null`.
(Account, SyncState)? mostUrgent(Iterable<(Account, SyncState)> issues) {
  final bad = issues.where((e) => e.$2.offline || e.$2.failed != null);
  if (bad.isEmpty) return null;
  return bad.reduce((a, b) => _rank(b.$2) < _rank(a.$2) ? b : a);
}
