//! Modèles de l'assistant d'IA tels que le moteur les décrit : fournisseurs, connexions, modèles, événements de chat.

import 'ai_privacy_models.dart';

export 'ai_privacy_models.dart';

/// Fournisseur d'IA connu du moteur : identifiant, nom affiché, adresse de base et si elle se modifie.
class AiProviderInfo {
  final String id;
  final String name;
  final String baseUrl;
  final bool editableUrl;
  final bool keyLogin;
  const AiProviderInfo(
    this.id,
    this.name,
    this.baseUrl,
    this.editableUrl, {
    this.keyLogin = false,
  });

  factory AiProviderInfo.fromJson(Map<String, dynamic> json) => AiProviderInfo(
    '${json['id']}',
    '${json['display_name']}',
    '${json['base_url']}',
    json['base_url_editable'] == true,
    keyLogin: json['key_login'] != null,
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

/// Appel d'outil annoncé : l'IA consulte la boîte ; [summary] est l'extrait de sa requête saisie par le modèle.
class AiToolCalled extends AiChatEvent {
  final String id;
  final String name;
  final String summary;
  const AiToolCalled(this.id, this.name, this.summary);
}

/// Une personne d'une proposition de brouillon : nom affiché éventuel et adresse validée par le moteur.
class AiAddressEntry {
  final String? name;
  final String address;
  const AiAddressEntry(this.name, this.address);

  factory AiAddressEntry.fromJson(Map<String, dynamic> json) =>
      AiAddressEntry(json['name'] as String?, '${json['address']}');

  Map<String, dynamic> toJson() => {'name': name, 'address': address};
}

List<AiAddressEntry>? _people(Object? raw) => raw == null
    ? null
    : [
        for (final p in raw as List)
          AiAddressEntry.fromJson((p as Map).cast<String, dynamic>()),
      ];

/// Brouillon que l'IA propose : [kind] vaut `new` (nouveau message) ou `edit` (brouillon ouvert) ; seuls les champs présents changent. L'app seule l'applique.
class AiDraftProposal extends AiChatEvent {
  final String id;
  final String kind;
  final List<AiAddressEntry>? to;
  final List<AiAddressEntry>? cc;
  final String? subject;
  final String? body;
  final String? selection;
  final String? replyTo;
  final String? identity;
  const AiDraftProposal({
    required this.id,
    required this.kind,
    this.to,
    this.cc,
    this.subject,
    this.body,
    this.selection,
    this.replyTo,
    this.identity,
  });

  /// Lit la ligne `draft_proposed` du flux : l'identifiant de l'appel et la proposition.
  factory AiDraftProposal.fromLine(Map<String, dynamic> line) =>
      AiDraftProposal.fromJson({
        ...(line['proposal'] as Map).cast<String, dynamic>(),
        'id': line['id'],
      });

  factory AiDraftProposal.fromJson(Map<String, dynamic> json) =>
      AiDraftProposal(
        id: '${json['id']}',
        kind: '${json['kind']}',
        to: _people(json['to']),
        cc: _people(json['cc']),
        subject: json['subject'] as String?,
        body: json['body'] as String?,
        selection: json['selection'] as String?,
        replyTo: json['reply_to'] as String?,
        identity: json['identity'] as String?,
      );

  Map<String, dynamic> toJson() => {
    'id': id,
    'kind': kind,
    'to': to?.map((p) => p.toJson()).toList(),
    'cc': cc?.map((p) => p.toJson()).toList(),
    'subject': subject,
    'body': body,
    'selection': selection,
    'reply_to': replyTo,
    'identity': identity,
  };
}

/// Action sur la boîte que l'IA propose : [action] vaut `archive`, `trash`, `mark_read`, `mark_unread`, `star` ou `unstar`. Rien ne change avant la confirmation de l'utilisateur.
class AiActionProposal extends AiChatEvent {
  final String id;
  final String action;
  final List<String> messages;
  final String? reason;
  const AiActionProposal({
    required this.id,
    required this.action,
    required this.messages,
    this.reason,
  });

  /// Lit la ligne `action_proposed` du flux : l'identifiant de l'appel et la proposition.
  factory AiActionProposal.fromLine(Map<String, dynamic> line) =>
      AiActionProposal.fromJson({
        ...(line['proposal'] as Map).cast<String, dynamic>(),
        'id': line['id'],
      });

  factory AiActionProposal.fromJson(Map<String, dynamic> json) =>
      AiActionProposal(
        id: '${json['id']}',
        action: '${json['action']}',
        messages: [for (final m in (json['messages'] as List)) '$m'],
        reason: json['reason'] as String?,
      );

  Map<String, dynamic> toJson() => {
    'id': id,
    'action': action,
    'messages': messages,
    'reason': reason,
  };
}

/// Fin d'un appel d'outil : [state] vaut `ok`, `denied` ou `error` ; [sources] sont les messages consultés, [deniedKind] le type de contenu refusé.
class AiToolFinished extends AiChatEvent {
  final String id;
  final String state;
  final int results;
  final List<String> sources;
  final String? deniedKind;
  const AiToolFinished(
    this.id,
    this.state,
    this.results,
    this.sources,
    this.deniedKind,
  );
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
