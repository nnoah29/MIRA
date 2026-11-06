/// Où le démon d'un profil met sa socket et son jeton.
class DaemonPaths {
  /// Chemin de la socket Unix.
  final String socket;

  /// Chemin du fichier du jeton local.
  final String token;
  const DaemonPaths({required this.socket, required this.token});
}

/// Valeur d'une variable d'environnement, `null` si absente ou vide.
String? _value(Map<String, String> env, String key) {
  final value = env[key];
  return value == null || value.isEmpty ? null : value;
}

/// Racine des profils comme le moteur : `MIRA_HOME` (ou `home`), `XDG_DATA_HOME`, puis `HOME`.
String? _profilesRoot(Map<String, String> env, String? home) {
  final miraHome = home ?? _value(env, 'MIRA_HOME');
  if (miraHome != null) return '$miraHome/profiles';
  final xdg = _value(env, 'XDG_DATA_HOME');
  if (xdg != null) return '$xdg/mira/profiles';
  final user = _value(env, 'HOME');
  return user == null ? null : '$user/.local/share/mira/profiles';
}

/// Emplacements de la socket et du jeton du démon du profil `profile`, comme `mira-daemon` les calcule.
/// `home` remplace `MIRA_HOME` (option `--home` du démon). Lève [ArgumentError] sans racine ou avec un nom de profil invalide.
DaemonPaths resolveDaemonPaths({
  required String profile,
  required Map<String, String> env,
  String? home,
}) {
  final invalid =
      profile.isEmpty ||
      profile == '.' ||
      profile == '..' ||
      profile.contains('/') ||
      profile.contains(r'\');
  if (invalid) throw ArgumentError.value(profile, 'profile', 'nom invalide');
  final root = _profilesRoot(env, home);
  if (root == null) {
    throw ArgumentError(
      'dossier de données du profil introuvable',
    ); // i18n:ignore
  }
  final profileRoot = '$root/$profile';
  final runtime = _value(env, 'XDG_RUNTIME_DIR');
  return DaemonPaths(
    socket: runtime == null
        ? '$profileRoot/daemon.sock'
        : '$runtime/mira-daemon/$profile.sock',
    token: '$profileRoot/daemon.token',
  );
}
