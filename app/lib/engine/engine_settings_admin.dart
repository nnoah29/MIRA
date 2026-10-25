import 'dart:convert';

import 'engine_client.dart';

/// Réglages que le moteur applique lui-même : relevé, comptes en sourdine, heures calmes.
class EngineSettings {
  final int? pollSecs;
  final Set<String> mutedAccounts;
  final (int, int)? quietHours;
  const EngineSettings({
    this.pollSecs,
    this.mutedAccounts = const {},
    this.quietHours,
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
    );
  }

  Map<String, dynamic> toJson() => {
    'poll_secs': pollSecs,
    'muted_accounts': mutedAccounts.toList()..sort(),
    'quiet_hours': quietHours == null
        ? null
        : {'from': quietHours!.$1, 'to': quietHours!.$2},
  };

  /// Mêmes réglages que `other` ?
  bool sameAs(EngineSettings other) =>
      jsonEncode(toJson()) == jsonEncode(other.toJson());

  EngineSettings withPoll(int? secs) => EngineSettings(
    pollSecs: secs,
    mutedAccounts: mutedAccounts,
    quietHours: quietHours,
  );

  EngineSettings withMuted(Set<String> accounts) => EngineSettings(
    pollSecs: pollSecs,
    mutedAccounts: accounts,
    quietHours: quietHours,
  );

  EngineSettings withQuiet((int, int)? hours) => EngineSettings(
    pollSecs: pollSecs,
    mutedAccounts: mutedAccounts,
    quietHours: hours,
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
