import 'engine_client.dart';

/// Ce que le compteur d'un libellé affiche dans la barre latérale.
enum CounterMode {
  unread,
  total,
  none;

  static CounterMode parse(Object? raw) => CounterMode.values.firstWhere(
    (m) => m.name == '$raw',
    orElse: () => CounterMode.unread,
  );
}

/// Réglages d'un libellé tels que le moteur les garde.
class LabelPrefs {
  final bool hidden;
  final bool hideFromInbox;
  final bool notifications;
  final CounterMode counter;
  final int? color;
  final int? position;
  const LabelPrefs({
    this.hidden = false,
    this.hideFromInbox = false,
    this.notifications = true,
    this.counter = CounterMode.unread,
    this.color,
    this.position,
  });

  factory LabelPrefs.fromJson(Map<String, dynamic> j) => LabelPrefs(
    hidden: j['hidden'] == true,
    hideFromInbox: j['hide_from_inbox'] == true,
    notifications: j['notifications'] != false,
    counter: CounterMode.parse(j['counter']),
    color: (j['color'] as num?)?.toInt(),
    position: (j['position'] as num?)?.toInt(),
  );
}

/// Un libellé (serveur ou local) avec son volume et ses réglages.
class LabelOverview {
  final bool local;
  final String id;
  final String? account;
  final String name;
  final int color;
  final int messages;
  final bool unused;
  final LabelPrefs prefs;
  const LabelOverview({
    required this.local,
    required this.id,
    required this.account,
    required this.name,
    required this.color,
    required this.messages,
    required this.unused,
    required this.prefs,
  });

  /// Clé du libellé dans l'interface : l'identifiant de la boîte, ou `local:` + l'identifiant du libellé local.
  String get key => local ? 'local:$id' : id;

  factory LabelOverview.fromJson(Map<String, dynamic> j) {
    final label = (j['label'] as Map).cast<String, dynamic>();
    return LabelOverview(
      local: label['kind'] == 'local',
      id: '${label['id']}',
      account: j['account'] as String?,
      name: '${j['name']}',
      color: (j['color'] as num).toInt(),
      messages: (j['messages'] as num).toInt(),
      unused: j['unused'] == true,
      prefs: LabelPrefs.fromJson(
        (j['settings'] as Map).cast<String, dynamic>(),
      ),
    );
  }
}

/// Réglages des libellés portés par le moteur : liste, réglages partiels, ordre.
class LabelSettingsAdmin {
  final EngineClient client;
  LabelSettingsAdmin(this.client);

  Future<Object?> _send(String op, [Map<String, dynamic>? data]) async {
    final r = await client.call(req('label_settings', op, data ?? const {}));
    return (r['data'] as Map)['data'];
  }

  Map<String, dynamic> _ref(LabelOverview l) => {
    'kind': l.local ? 'local' : 'server',
    'id': l.id,
  };

  /// Tous les libellés, dans l'ordre d'affichage.
  Future<List<LabelOverview>> list() async {
    final items = await _send('list_labels') as List;
    return [
      for (final v in items)
        LabelOverview.fromJson((v as Map).cast<String, dynamic>()),
    ];
  }

  /// Modifie les seuls réglages fournis.
  Future<void> set(
    LabelOverview label, {
    bool? hidden,
    bool? hideFromInbox,
    bool? notifications,
    CounterMode? counter,
    int? color,
  }) => _send('set_label_settings', {
    'label': _ref(label),
    'hidden': hidden,
    'hide_from_inbox': hideFromInbox,
    'notifications': notifications,
    'counter': counter?.name,
    'color': color,
  });

  /// Fixe l'ordre d'affichage (la liste doit nommer chaque libellé une fois).
  Future<void> reorder(List<LabelOverview> order) => _send('reorder_labels', {
    'order': [for (final l in order) _ref(l)],
  });
}
