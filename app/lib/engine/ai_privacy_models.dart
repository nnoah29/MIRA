//! Confidentialité de l'assistant d'IA : types de contenu autorisés, masquage, lignes du journal des envois.

/// Type de contenu « mail ouvert » tel que le moteur le nomme.
const aiKindMail = 'mail_message';

/// Type de contenu « listes de messages » (résultats de recherche) tel que le moteur le nomme.
const aiKindMailList = 'mail_list';

/// Type de contenu « boîtes et comptes consultés » tel que le moteur le nomme.
const aiKindToolResult = 'tool_result';

/// Types de contenu que l'app sait produire aujourd'hui.
const aiKnownKinds = [aiKindMail, aiKindMailList, aiKindToolResult];

/// Catégories de données sensibles que le masquage reconnaît, dans l'ordre d'affichage.
const aiMaskCategories = ['iban', 'card', 'secret', 'otp', 'email', 'phone'];

/// Catégories actives par défaut quand le moteur n'en dit rien.
const aiDefaultMaskCategories = ['iban', 'card', 'secret', 'otp'];

/// Réglage de masquage d'une connexion : `off` ou `mask`, et les catégories masquées.
class AiMasking {
  final String mode;
  final List<String> categories;
  const AiMasking(this.mode, this.categories);

  /// Réglage prudent d'une connexion distante.
  static const remoteDefault = AiMasking('mask', aiDefaultMaskCategories);

  factory AiMasking.fromJson(Object? raw) {
    if (raw is! Map) return remoteDefault;
    final list = raw['categories'];
    return AiMasking('${raw['mode'] ?? 'mask'}', [
      if (list is List)
        for (final c in list) '$c',
    ]);
  }

  /// Vrai quand le masquage est actif.
  bool get on => mode == 'mask';

  AiMasking copyWith({String? mode, List<String>? categories}) =>
      AiMasking(mode ?? this.mode, categories ?? this.categories);

  Map<String, dynamic> toJson() => {'mode': mode, 'categories': categories};
}

/// Une génération telle que le journal la garde : des comptes et des codes, jamais de texte.
class AiJournalRow {
  final String id;
  final int at;
  final String provider;
  final String model;
  final bool local;
  final List<String> kinds;
  final int chars;
  final int masked;
  final int? inputTokens;
  final int? outputTokens;
  final String outcome;
  const AiJournalRow({
    required this.id,
    required this.at,
    required this.provider,
    required this.model,
    required this.local,
    required this.kinds,
    required this.chars,
    required this.masked,
    required this.outcome,
    this.inputTokens,
    this.outputTokens,
  });

  factory AiJournalRow.fromJson(Map<String, dynamic> json) => AiJournalRow(
    id: '${json['id']}',
    at: (json['at'] as num?)?.toInt() ?? 0,
    provider: '${json['provider']}',
    model: '${json['model']}',
    local: json['local'] == true,
    kinds: [for (final k in (json['kinds'] as List? ?? const [])) '$k'],
    chars: (json['chars'] as num?)?.toInt() ?? 0,
    masked: (json['masked'] as num?)?.toInt() ?? 0,
    inputTokens: (json['input_tokens'] as num?)?.toInt(),
    outputTokens: (json['output_tokens'] as num?)?.toInt(),
    outcome: '${json['outcome']}',
  );
}
