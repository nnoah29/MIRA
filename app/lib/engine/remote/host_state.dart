/// État de l'hôte annoncé par le démon dans `hello_ok`.
enum HostState {
  /// Le moteur est ouvert et répond.
  ready,

  /// Le démon ouvre encore son moteur.
  starting,

  /// Le trousseau du système est verrouillé et le démon n'a aucun écran pour le déverrouiller.
  keyringLocked,

  /// Le démon et l'app ne sont pas compatibles : le démon doit être redémarré.
  updateRequired,

  /// Valeur inconnue (démon plus récent que l'app).
  unknown;

  /// Lit la valeur du fil ; toute valeur inconnue donne [unknown].
  static HostState parse(String? wire) => switch (wire) {
    'ready' => ready,
    'starting' => starting,
    'keyring_locked' => keyringLocked,
    'update_required' => updateRequired,
    _ => unknown,
  };
}
