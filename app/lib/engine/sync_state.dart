/// État de synchronisation d'un compte, tel que les événements du moteur le racontent.
class SyncState {
  bool offline = false;
  bool syncing = false;
  String? failed;
  int done = 0;
  int total = 0;

  /// Avancement en pourcentage (`null` : inconnu).
  int? get percent => total > 0 ? (done * 100 ~/ total).clamp(0, 100) : null;

  /// Applique un événement ; vrai si l'affichage doit changer.
  bool apply(String type, Map<String, dynamic> kind) {
    final before = (offline, syncing, failed, percent);
    switch (type) {
      case 'went_offline':
        offline = true;
        syncing = false;
      case 'back_online':
        offline = false;
      case 'sync_started':
        syncing = true;
        failed = null;
        done = total = 0;
      case 'sync_progress':
        syncing = true;
        done = (kind['done'] as num?)?.toInt() ?? 0;
        total = (kind['total'] as num?)?.toInt() ?? 0;
      case 'sync_finished':
        syncing = false;
        offline = false;
        failed = null;
      case 'sync_failed':
        syncing = false;
        failed = '${kind['code']}';
    }
    return before != (offline, syncing, failed, percent);
  }
}
