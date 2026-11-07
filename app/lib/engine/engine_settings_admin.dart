import 'dart:convert';

import 'engine_client.dart';

/// Réglages des notifications portés par le moteur (mêmes défauts que le moteur).
class NotificationPrefs {
  final bool enabled;
  final bool preview;
  final bool sound;
  final bool whenFocused;
  const NotificationPrefs({
    this.enabled = true,
    this.preview = true,
    this.sound = true,
    this.whenFocused = false,
  });

  factory NotificationPrefs.fromJson(Map<String, dynamic>? j) =>
      NotificationPrefs(
        enabled: j?['enabled'] as bool? ?? true,
        preview: j?['preview'] as bool? ?? true,
        sound: j?['sound'] as bool? ?? true,
        whenFocused: j?['when_focused'] as bool? ?? false,
      );

  Map<String, dynamic> toJson() => {
    'enabled': enabled,
    'preview': preview,
    'sound': sound,
    'when_focused': whenFocused,
  };

  /// Copie avec le champ donné changé.
  NotificationPrefs with_({
    bool? enabled,
    bool? preview,
    bool? sound,
    bool? whenFocused,
  }) => NotificationPrefs(
    enabled: enabled ?? this.enabled,
    preview: preview ?? this.preview,
    sound: sound ?? this.sound,
    whenFocused: whenFocused ?? this.whenFocused,
  );
}

/// Réglages que le moteur applique lui-même : relevé, comptes en sourdine, heures calmes, notifications.
class EngineSettings {
  final int? pollSecs;
  final Set<String> mutedAccounts;
  final (int, int)? quietHours;
  final NotificationPrefs notifications;
  const EngineSettings({
    this.pollSecs,
    this.mutedAccounts = const {},
    this.quietHours,
    this.notifications = const NotificationPrefs(),
  });

  factory EngineSettings.fromJson(Map<String, dynamic> j) {
    final quiet = (j['quiet_hours'] as Map?)?.cast<String, dynamic>();
    return EngineSettings(
      pollSecs: (j['poll_secs'] as num?)?.toInt(),
      mutedAccounts: {
        for (final a in (j['muted_accounts'] as List?) ?? []) '$a',
      },
      quietHours: quiet == null
          ? null
          : ((quiet['from'] as num).toInt(), (quiet['to'] as num).toInt()),
      notifications: NotificationPrefs.fromJson(
        (j['notifications'] as Map?)?.cast<String, dynamic>(),
      ),
    );
  }

  Map<String, dynamic> toJson() => {
    'poll_secs': pollSecs,
    'muted_accounts': mutedAccounts.toList()..sort(),
    'quiet_hours': quietHours == null
        ? null
        : {'from': quietHours!.$1, 'to': quietHours!.$2},
    'notifications': notifications.toJson(),
  };

  /// Mêmes réglages que `other` ?
  bool sameAs(EngineSettings other) =>
      jsonEncode(toJson()) == jsonEncode(other.toJson());

  EngineSettings withPoll(int? secs) => EngineSettings(
    pollSecs: secs,
    mutedAccounts: mutedAccounts,
    quietHours: quietHours,
    notifications: notifications,
  );

  EngineSettings withMuted(Set<String> accounts) => EngineSettings(
    pollSecs: pollSecs,
    mutedAccounts: accounts,
    quietHours: quietHours,
    notifications: notifications,
  );

  EngineSettings withQuiet((int, int)? hours) => EngineSettings(
    pollSecs: pollSecs,
    mutedAccounts: mutedAccounts,
    quietHours: hours,
    notifications: notifications,
  );

  EngineSettings withNotifications(NotificationPrefs next) => EngineSettings(
    pollSecs: pollSecs,
    mutedAccounts: mutedAccounts,
    quietHours: quietHours,
    notifications: next,
  );
}

/// Lecture et remplacement des réglages du moteur.
class EngineSettingsAdmin {
  final EngineClient client;
  EngineSettingsAdmin(this.client);

  Future<EngineSettings> _send(String op, [Map<String, dynamic>? data]) async {
    final r = await client.call(req('settings', op, data ?? const {}));
    final inner = ((r['data'] as Map)['data'] as Map).cast<String, dynamic>();
    return EngineSettings.fromJson(inner);
  }

  /// Réglages en vigueur.
  Future<EngineSettings> read() => _send('get_settings');

  /// Remplace l'ensemble des réglages ; le moteur les applique tout de suite.
  Future<EngineSettings> write(EngineSettings settings) =>
      _send('set_settings', {'settings': settings.toJson()});
}
