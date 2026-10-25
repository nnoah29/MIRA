import 'dart:math';

import 'engine_client.dart';

/// Modèle de message tel que le moteur le tient : identifiant, nom, objet facultatif, corps.
class EngineTemplate {
  final String id;
  final String name;
  final String? subject;
  final String body;
  const EngineTemplate(this.id, this.name, this.subject, this.body);

  factory EngineTemplate.fromJson(Map<String, dynamic> json) => EngineTemplate(
    '${json['id']}',
    '${json['name']}',
    json['subject'] as String?,
    '${json['body']}',
  );
}

/// Modèle rendu : objet et corps remplacés, noms des variables restées visibles.
class RenderedTemplate {
  final String? subject;
  final String body;
  final List<String> unresolved;
  const RenderedTemplate(this.subject, this.body, this.unresolved);

  factory RenderedTemplate.fromJson(Map<String, dynamic> json) =>
      RenderedTemplate(json['subject'] as String?, '${json['body']}', [
        for (final v in json['unresolved'] as List) '$v',
      ]);
}

/// Modèles de messages portés par le moteur : lecture, écritures, rendu des variables.
class TemplateAdmin {
  final EngineClient client;
  int _seq = 0;
  final String _run = Random().nextInt(1 << 32).toRadixString(36);
  TemplateAdmin(this.client);

  String _key(String what) =>
      'ui-template-$what-$_run-${DateTime.now().microsecondsSinceEpoch}-${_seq++}';

  Future<Object?> _send(String op, [Map<String, dynamic>? data]) async {
    final r = await client.call(req('template', op, data ?? const {}));
    return (r['data'] as Map)['data'];
  }

  EngineTemplate _one(Object? raw) =>
      EngineTemplate.fromJson((raw as Map).cast<String, dynamic>());

  /// Tous les modèles, par nom.
  Future<List<EngineTemplate>> list() async {
    final items = await _send('list_templates') as List;
    return [for (final v in items) _one(v)];
  }

  /// Crée un modèle (le moteur choisit l'identifiant).
  Future<EngineTemplate> create(
    String name,
    String? subject,
    String body,
  ) async => _one(
    await _send('create_template', {
      'name': name,
      'subject': subject,
      'body': body,
      'idempotency_key': _key('create'),
    }),
  );

  /// Remplace nom, objet et corps d'un modèle.
  Future<EngineTemplate> update(
    String id,
    String name,
    String? subject,
    String body,
  ) async => _one(
    await _send('update_template', {
      'template': id,
      'name': name,
      'subject': subject,
      'body': body,
    }),
  );

  /// Supprime un modèle.
  Future<void> delete(String id) => _send('delete_template', {'template': id});

  /// Rend un modèle dans ce contexte (contact, identité, objet, date, heure).
  Future<RenderedTemplate> render(
    String id,
    Map<String, dynamic> context,
  ) async {
    final r = await _send('render_template', {
      'template': id,
      'context': context,
    });
    return RenderedTemplate.fromJson((r as Map).cast<String, dynamic>());
  }
}
