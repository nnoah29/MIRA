import 'dart:convert';

import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../data/models.dart';

/// Réglages des notifications de nouveaux messages, gardés dans les préférences.
class NotifySettings {
  final SharedPreferences prefs;
  NotifySettings(this.prefs);

  bool _flag(String k, bool byDefault) =>
      (prefs.getString(k) ?? (byDefault ? '1' : '0')) == '1';

  bool get enabled => _flag('notify', true);
  bool get preview => _flag('notifyPreview', true);
  bool get sound => _flag('notifySound', true);
  bool get whenFocused => _flag('notifyFocused', false);

  /// Heures calmes `(début, fin)` en heures pleines ; `null` : jamais.
  (int, int)? get quiet {
    final raw = prefs.getString('notifyQuiet') ?? '';
    final parts = raw.split('-').map(int.tryParse).toList();
    if (parts.length != 2 || parts.contains(null)) return null;
    return (parts[0]!, parts[1]!);
  }

  /// Comptes dont les nouveaux messages ne sont pas notifiés.
  Set<String> get muted {
    try {
      return {
        for (final a
            in jsonDecode(prefs.getString('notifyMuted') ?? '[]') as List)
          '$a',
      };
    } catch (_) {
      return {};
    }
  }

  void setMuted(String account, bool mute) {
    final next = muted;
    mute ? next.add(account) : next.remove(account);
    prefs.setString('notifyMuted', jsonEncode(next.toList()..sort()));
  }

  /// Vrai si l'heure `hour` tombe dans les heures calmes (plage pouvant passer minuit).
  bool isQuiet(int hour) {
    final q = quiet;
    if (q == null) return false;
    final (from, to) = q;
    return from <= to ? hour >= from && hour < to : hour >= from || hour < to;
  }
}

/// Messages arrivés en réception depuis la lecture précédente : non lus, récents, inconnus avant.
List<Mail> newArrivals(List<Mail>? before, List<Mail> fresh, int nowMs) {
  if (before == null || before.isEmpty) return const [];
  final known = {
    for (final m in before)
      for (final x in m.msgs) ?x.ref,
  };
  const day = 24 * 3600 * 1000;
  return [
    for (final m in fresh)
      if (m.folder == 'inbox' &&
          m.unread &&
          nowMs - m.date < day &&
          m.msgs.any((x) => x.ref != null && !known.contains(x.ref)))
        m,
  ];
}

/// Affichage des notifications par le système (remplaçable dans les tests).
abstract class SystemNotifications {
  /// Prépare le service ; `onTap` reçoit l'identifiant du message cliqué.
  Future<void> init(void Function(String payload) onTap);

  Future<void> show(
    int id,
    String title,
    String body, {
    String? payload,
    bool sound = true,
  });
}

/// Notifications du bureau (D-Bus sous Linux) et d'Android.
class PluginNotifications implements SystemNotifications {
  final _plugin = FlutterLocalNotificationsPlugin();

  @override
  Future<void> init(void Function(String payload) onTap) async {
    await _plugin.initialize(
      settings: InitializationSettings(
        linux: LinuxInitializationSettings(
          defaultActionName: 'Ouvrir',
          defaultIcon: AssetsLinuxIcon('assets/icon/mira.png'),
        ),
        android: const AndroidInitializationSettings('@mipmap/ic_launcher'),
      ),
      onDidReceiveNotificationResponse: (r) {
        final p = r.payload;
        if (p != null && p.isNotEmpty) onTap(p);
      },
    );
  }

  @override
  Future<void> show(
    int id,
    String title,
    String body, {
    String? payload,
    bool sound = true,
  }) => _plugin.show(
    id: id,
    title: title,
    body: body,
    payload: payload,
    notificationDetails: NotificationDetails(
      linux: LinuxNotificationDetails(
        category: LinuxNotificationCategory.emailArrived,
        sound: sound ? ThemeLinuxSound('message-new-email') : null,
        suppressSound: !sound,
      ),
      android: AndroidNotificationDetails(
        'mail',
        'Nouveaux messages',
        playSound: sound,
      ),
    ),
  );
}
