import '../../i18n/i18n.dart';
import '../engine_client.dart';
import 'connection.dart';

/// Le moteur en arrière-plan ne répond pas.
EngineException unavailableError() => EngineException.local(
  'transport.unavailable',
  t('Moteur en arrière-plan injoignable'),
);

/// Les versions du démon et de l'app ne sont pas compatibles ; le démon ferme la connexion, il faut l'arrêter à la main.
EngineException updateRequiredError() => EngineException.local(
  'host.update_required',
  t(
    'Le moteur en arrière-plan n’est pas compatible avec cette version de MIRA : arrêtez-le (pkill mira-daemon) puis relancez MIRA',
  ),
);

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
