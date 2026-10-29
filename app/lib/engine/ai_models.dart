//! Modèles de l'assistant d'IA tels que le moteur les décrit : fournisseurs, connexions, modèles, événements de chat.

import 'ai_privacy_models.dart';

export 'ai_privacy_models.dart';

/// Fournisseur d'IA connu du moteur : identifiant, nom affiché, adresse de base et si elle se modifie.
class AiProviderInfo {
  final String id;
  final String name;
  final String baseUrl;
  final bool editableUrl;
  const AiProviderInfo(this.id, this.name, this.baseUrl, this.editableUrl);

  factory AiProviderInfo.fromJson(Map<String, dynamic> json) => AiProviderInfo(
    '${json['id']}',
    '${json['display_name']}',
    '${json['base_url']}',
    json['base_url_editable'] == true,
  );
}

/// Modèle annoncé par un service : identifiant et date de création déclarée (secondes).
class AiModelInfo {
  final String id;
  final int? createdAt;
  const AiModelInfo(this.id, this.createdAt);

  factory AiModelInfo.fromJson(Map<String, dynamic> json) =>
      AiModelInfo('${json['id']}', (json['created_at'] as num?)?.toInt());
}

/// Connexion à un service d'IA ; la clé n'est jamais portée, seulement `hasKey`.
class AiConnectionInfo {
  final String id;
  final String provider;
  final String label;
  final String? baseUrl;
  final String? defaultModel;
  final bool isDefault;
  final bool hasKey;
  final bool consentGiven;
  final List<AiModelInfo>? models;
  final List<String> allowedKinds;
  final AiMasking masking;
  const AiConnectionInfo({
    required this.id,
    required this.provider,
    required this.label,
    this.baseUrl,
    this.defaultModel,
    this.isDefault = false,
    this.hasKey = false,
    this.consentGiven = false,
    this.models,
    this.allowedKinds = const [],
    this.masking = AiMasking.remoteDefault,
  });

  factory AiConnectionInfo.fromJson(Map<String, dynamic> json) {
    final list = (json['models'] as Map?)?['models'] as List?;
    return AiConnectionInfo(
      id: '${json['id']}',
      provider: '${json['provider']}',
      label: '${json['label']}',
      baseUrl: json['base_url'] as String?,
      defaultModel: json['default_model'] as String?,
      isDefault: json['is_default'] == true,
      hasKey: json['has_key'] == true,
      consentGiven: json['consent_at'] != null,
      models: list == null
          ? null
          : [
              for (final m in list)
                AiModelInfo.fromJson((m as Map).cast<String, dynamic>()),
            ],
      allowedKinds: [
        for (final k in (json['allowed_kinds'] as List? ?? const [])) '$k',
      ],
      masking: AiMasking.fromJson(json['masking']),
    );
  }
}

/// Événement d'une génération, dans l'ordre : départ, textes, usage, fin ou échec.
sealed class AiChatEvent {
  const AiChatEvent();
}

/// Génération lancée ; l'opération sert à l'annuler.
class AiStarted extends AiChatEvent {
  final String operation;
  const AiStarted(this.operation);
}

/// Morceau de texte de la réponse.
class AiText extends AiChatEvent {
  final String text;
  const AiText(this.text);
}

/// Nombre de données sensibles masquées avant l'envoi (annoncé avant le premier texte).
class AiMasked extends AiChatEvent {
  final int count;
  const AiMasked(this.count);
}

/// Nombre de jetons lus et produits.
class AiUsage extends AiChatEvent {
  final int input;
  final int output;
  const AiUsage(this.input, this.output);
}

/// Fin de la génération : raison (`end_turn`, `max_tokens`, `cancelled`, `failed`), code d'échec et délai conseillé.
class AiDone extends AiChatEvent {
  final String reason;
  final String? code;
  final int? retryAfterSecs;
  const AiDone(this.reason, {this.code, this.retryAfterSecs});

  /// Vrai quand la génération a échoué.
  bool get failed => reason == 'failed';
}

/// Échec avant tout flux (connexion, clé, modèle, consentement) : code stable et message du moteur.
class AiFailed extends AiChatEvent {
  final String code;
  final String message;
  const AiFailed(this.code, this.message);
}
