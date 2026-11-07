import '../../i18n/i18n.dart';
import '../engine_client.dart';
import 'connection.dart';

/// Le moteur en arrière-plan ne répond pas.
EngineException unavailableError() => EngineException.local(
  'transport.unavailable',
  t('Moteur en arrière-plan injoignable'),
);

/// Le démon en marche est plus ancien que l'app et n'a pas pu être redémarré.
EngineException updateRequiredError() => EngineException.local(
  'host.update_required',
  t(
    'Le moteur en arrière-plan est trop ancien pour cette version de MIRA et n’a pas pu être redémarré : relancez votre session ou redémarrez-le (systemctl --user restart mira-daemon)',
  ),
);

/// Le démon lancé par l'app s'est arrêté avant d'être prêt ; le code dit pourquoi.
EngineException daemonExitedError(int code) =>
    EngineException.local('transport.daemon_exited', switch (code) {
      3 => t('Un autre moteur en arrière-plan utilise déjà ce profil'),
      2 => t('La configuration du moteur en arrière-plan est invalide'),
      _ => t('Le moteur en arrière-plan s’est arrêté au démarrage'),
    });

/// Refus de poignée de main : un message précis, jamais le jeton.
EngineException refusalError(HandshakeRefused refusal) =>
    switch (refusal.reason) {
      'auth' => EngineException.local(
        'host.auth_refused',
        t('Accès refusé par le moteur en arrière-plan'),
      ),
      'protocol_version' || 'contract_version' => updateRequiredError(),
      _ => unavailableError(),
    };

/// Le trousseau du système reste verrouillé : le démon reprendra seul dès qu'il sera déverrouillé.
EngineException keyringLockedError() => EngineException.local(
  'host.keyring_locked',
  t(
    'Le trousseau du système est verrouillé : déverrouillez-le puis relancez MIRA, le moteur en arrière-plan reprendra seul',
  ),
);
