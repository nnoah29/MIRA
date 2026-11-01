//! Conversations enregistrées par le moteur : ce que la liste montre, et la conversation complète.

/// Titre et dates d'une conversation enregistrée ; le contenu n'est pas dans la liste.
class AiConversationMeta {
  final String id;
  final String title;
  final DateTime createdAt;
  final DateTime updatedAt;
  final int payloadVersion;
  const AiConversationMeta({
    required this.id,
    required this.title,
    required this.createdAt,
    required this.updatedAt,
    required this.payloadVersion,
  });

  factory AiConversationMeta.fromJson(Map<String, dynamic> json) =>
      AiConversationMeta(
        id: '${json['id']}',
        title: '${json['title']}',
        createdAt: _date(json['created_at']),
        updatedAt: _date(json['updated_at']),
        payloadVersion: (json['payload_version'] as num?)?.toInt() ?? 0,
      );

  static DateTime _date(Object? millis) =>
      DateTime.fromMillisecondsSinceEpoch((millis as num?)?.toInt() ?? 0);
}

/// Une conversation avec son contenu, texte JSON que l'app a écrit.
class AiConversationDoc {
  final AiConversationMeta meta;
  final String payload;
  const AiConversationDoc({required this.meta, required this.payload});

  factory AiConversationDoc.fromJson(Map<String, dynamic> json) =>
      AiConversationDoc(
        meta: AiConversationMeta.fromJson(json),
        payload: '${json['payload']}',
      );
}
