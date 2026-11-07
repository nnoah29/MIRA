import 'dart:async';

import '../state/mail_notifier.dart';

/// Affiche les notifications que le moteur embarqué décide d'envoyer : aucune règle ici, le moteur a déjà tout tranché.
class NotificationFeed {
  NotificationFeed(this._notifications, this._display);

  final Stream<Map<String, dynamic>> _notifications;
  final SystemNotifications _display;
  StreamSubscription<Map<String, dynamic>>? _subscription;

  /// Commence à afficher ; un afficheur en panne ne coupe pas le flux.
  void start() {
    _subscription = _notifications.listen(_show);
  }

  Future<void> _show(Map<String, dynamic> n) async {
    try {
      await _display.show(
        (n['id'] as num).toInt(),
        '${n['title']}',
        '${n['body']}',
        payload: n['message'] as String?,
        sound: n['sound'] != false,
      );
    } catch (_) {
      /* service de notifications indisponible : rien à afficher */
    }
  }

  /// Arrête d'afficher et ferme le flux (le moteur arrête alors son notifieur).
  Future<void> stop() async {
    await _subscription?.cancel();
    _subscription = null;
  }
}
