import 'dart:convert';

import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../engine/engine_settings_admin.dart';

/// Heures calmes écrites `de-à` en heures pleines ; `null` : jamais.
(int, int)? parseQuiet(String raw) {
  final parts = raw.split('-').map(int.tryParse).toList();
  if (parts.length != 2 || parts.contains(null)) return null;
  return (parts[0]!, parts[1]!);
}

/// Réglages des notifications de nouveaux messages : heures calmes et comptes en sourdine viennent
/// du moteur quand il les porte, des préférences sinon ; le reste reste dans les préférences.
class NotifySettings {
  final SharedPreferences prefs;
  EngineSettings? engine;
  NotifySettings(this.prefs);

  bool _flag(String k, bool byDefault) =>
      (prefs.getString(k) ?? (byDefault ? '1' : '0')) == '1';

  bool get enabled => engine?.notifications.enabled ?? _flag('notify', true);
  bool get preview =>
      engine?.notifications.preview ?? _flag('notifyPreview', true);
  bool get sound => engine?.notifications.sound ?? _flag('notifySound', true);
  bool get whenFocused =>
      engine?.notifications.whenFocused ?? _flag('notifyFocused', false);

  /// Heures calmes `(début, fin)` en heures pleines ; `null` : jamais.
  (int, int)? get quiet => engine != null
      ? engine!.quietHours
      : parseQuiet(prefs.getString('notifyQuiet') ?? '');

  /// Comptes dont les nouveaux messages ne sont pas notifiés.
  Set<String> get muted {
    if (engine != null) return engine!.mutedAccounts;
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

/// Notifications du bureau (D-Bus sous Linux, centre de notifications Windows) et d'Android.
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
        windows: const WindowsInitializationSettings(
          appName: 'MIRA',
          appUserModelId: 'com.toffanoah.mira',
          guid: '6f2b1c9e-3a7d-4e58-9b0a-4d1f7c52e8a3',
        ),
      ),
      onDidReceiveNotificationResponse: (r) {
        final p = r.payload;
        if (p != null && p.isNotEmpty) onTap(p);
      },
    );
    await _plugin
        .resolvePlatformSpecificImplementation<
          AndroidFlutterLocalNotificationsPlugin
        >()
        ?.requestNotificationsPermission();
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
