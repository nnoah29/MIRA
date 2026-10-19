import 'dart:convert';

import '../data/models.dart';

/// Préfixe des identifiants de libellés locaux, qui les distingue des boîtes du serveur.
const localLabelPrefix = 'local:';

/// Couleurs proposées tour à tour aux nouveaux libellés locaux.
const localLabelColors = [
  0xFFe8542b,
  0xFF6aa8d9,
  0xFF8bc48a,
  0xFFd9b46a,
  0xFFc98bb9,
  0xFF9a8be0,
];

bool isLocalLabel(String key) => key.startsWith(localLabelPrefix);

/// Libellés propres à l'application (communs à tous les comptes) et leur pose sur les conversations.
class LocalLabels {
  final Map<String, LabelDef> defs = {};
  final Map<String, Set<String>> _tags = {};

  LocalLabels();

  /// Relit l'état enregistré ; un contenu illisible donne un état vide.
  factory LocalLabels.decode(String? raw) {
    final out = LocalLabels();
    try {
      final data = jsonDecode(raw ?? '{}') as Map;
      for (final e in (data['defs'] as Map? ?? {}).entries) {
        final v = e.value as List;
        out.defs['${e.key}'] = LabelDef(
          '${v[0]}',
          (v[1] as num).toInt(),
          local: true,
        );
      }
      for (final e in (data['tags'] as Map? ?? {}).entries) {
        final keys = {for (final k in e.value as List) '$k'}
          ..removeWhere((k) => !out.defs.containsKey(k));
        if (keys.isNotEmpty) out._tags['${e.key}'] = keys;
      }
    } catch (_) {
      out.defs.clear();
      out._tags.clear();
    }
    return out;
  }

  String encode() => jsonEncode({
    'defs': {
      for (final e in defs.entries) e.key: [e.value.n, e.value.c],
    },
    'tags': {for (final e in _tags.entries) e.key: e.value.toList()},
  });

  /// Crée un libellé ; rend sa clé, ou `null` si le nom est vide ou déjà pris.
  String? create(String name) {
    final n = name.trim();
    if (n.isEmpty || _taken(n, null)) return null;
    var i = defs.length;
    var key = '$localLabelPrefix$i';
    while (defs.containsKey(key)) {
      key = '$localLabelPrefix${++i}';
    }
    defs[key] = LabelDef(
      n,
      localLabelColors[defs.length % localLabelColors.length],
      local: true,
    );
    return key;
  }

  bool _taken(String name, String? except) => defs.entries.any(
    (e) => e.key != except && e.value.n.toLowerCase() == name.toLowerCase(),
  );

  /// Renomme un libellé ; `false` si le nom est vide, déjà pris ou la clé inconnue.
  bool rename(String key, String name) {
    final def = defs[key];
    final n = name.trim();
    if (def == null || n.isEmpty || _taken(n, key)) return false;
    defs[key] = LabelDef(n, def.c, local: true);
    return true;
  }

  /// Passe à la couleur suivante de la palette.
  void recolor(String key) {
    final def = defs[key];
    if (def == null) return;
    final next =
        (localLabelColors.indexOf(def.c) + 1) % localLabelColors.length;
    defs[key] = LabelDef(def.n, localLabelColors[next], local: true);
  }

  /// Supprime un libellé et le retire de toutes les conversations.
  void delete(String key) {
    defs.remove(key);
    _tags.removeWhere((_, keys) => keys.remove(key) && keys.isEmpty);
  }

  /// Pose ou retire un libellé sur des conversations.
  void set(Iterable<String> mailIds, String key, bool on) {
    if (!defs.containsKey(key)) return;
    for (final id in mailIds) {
      final keys = _tags.putIfAbsent(id, () => {});
      on ? keys.add(key) : keys.remove(key);
      if (keys.isEmpty) _tags.remove(id);
    }
  }

  /// Reporte les libellés locaux dans `labels` des conversations données.
  void apply(Iterable<Mail> mails) {
    for (final m in mails) {
      final keys = _tags[m.id];
      m.labels = [
        for (final k in m.labels)
          if (!isLocalLabel(k)) k,
        ...?keys,
      ];
    }
  }

  /// Nombre de conversations portant ce libellé.
  int count(String key) => _tags.values.where((k) => k.contains(key)).length;
}
