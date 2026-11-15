//! Mémoire de l'IA côté app : souvenirs, réglages, statut, résultats de recherche et proposition de l'assistant.

/// Un souvenir durable : [kind] vaut `fact`, `preference` ou `action` ; [source] vaut `user` ou `ai`.
class MemoryItem {
  final String id;
  final String text;
  final String kind;
  final String source;
  final DateTime createdAt;
  const MemoryItem({
    required this.id,
    required this.text,
    required this.kind,
    required this.source,
    required this.createdAt,
  });

  /// Dit si l'IA l'a proposé (avec l'accord de l'utilisateur) plutôt que l'utilisateur lui-même.
  bool get byAi => source == 'ai';

  factory MemoryItem.fromJson(Map<String, dynamic> json) => MemoryItem(
    id: '${json['id']}',
    text: '${json['text']}',
    kind: '${json['kind']}',
    source: '${json['source']}',
    createdAt: DateTime.fromMillisecondsSinceEpoch(
      (json['created_at'] as num).toInt(),
    ),
  );
}

/// Résultat d'une recherche par le sens : le souvenir et sa proximité, entre -1 et 1.
class MemoryHit {
  final MemoryItem item;
  final double score;
  const MemoryHit(this.item, this.score);

  factory MemoryHit.fromJson(Map<String, dynamic> json) => MemoryHit(
    MemoryItem.fromJson((json['record'] as Map).cast<String, dynamic>()),
    ((json['score_permille'] as num).toInt()) / 1000,
  );
}

/// Réglages de la mémoire : activée ou non, calcul sur l'appareil ([connection] nulle) ou chez le service d'une connexion.
class MemorySettings {
  final bool enabled;
  final String? connection;
  final String? model;
  const MemorySettings({this.enabled = false, this.connection, this.model});

  /// Dit si les embeddings sont calculés sur l'appareil.
  bool get local => connection == null;

  factory MemorySettings.fromJson(Map<String, dynamic> json) {
    final e = (json['embedder'] as Map?)?.cast<String, dynamic>() ?? const {};
    final remote = e['kind'] == 'provider';
    return MemorySettings(
      enabled: json['enabled'] == true,
      connection: remote ? e['connection'] as String? : null,
      model: remote ? e['model'] as String? : null,
    );
  }

  Map<String, dynamic> toJson() => {
    'enabled': enabled,
    'embedder': local
        ? {'kind': 'local'}
        : {'kind': 'provider', 'connection': connection, 'model': model},
  };
}

/// Ce que montre l'écran de la mémoire : réglages, modèle local présent, nombre de souvenirs et de périmés.
class MemoryStatus {
  final MemorySettings settings;
  final bool modelInstalled;
  final int count;
  final int stale;
  const MemoryStatus(
    this.settings,
    this.modelInstalled,
    this.count,
    this.stale,
  );

  factory MemoryStatus.fromJson(Map<String, dynamic> json) => MemoryStatus(
    MemorySettings.fromJson((json['settings'] as Map).cast<String, dynamic>()),
    json['model_installed'] == true,
    (json['count'] as num).toInt(),
    (json['stale'] as num).toInt(),
  );
}

/// Bilan d'une réindexation : souvenirs recalculés et souvenirs en échec.
class MemoryReindex {
  final int done;
  final int failed;
  const MemoryReindex(this.done, this.failed);
}
