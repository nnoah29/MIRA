import 'package:mira/engine/engine_client.dart';

/// Moteur factice du domaine `template` : garde les requêtes et tient les modèles en mémoire.
class FakeTemplateEngine implements EngineClient {
  final calls = <Map<String, dynamic>>[];
  final templates = <Map<String, dynamic>>[];
  EngineException? failNext;
  int _n = 0;

  /// Requêtes reçues pour cette opération.
  List<Map<String, dynamic>> requestsOf(String op) => [
    for (final c in calls)
      if ((c['request'] as Map)['op'] == op)
        (c['request'] as Map).cast<String, dynamic>(),
  ];

  Map<String, dynamic> _wrap(String type, Object? data) => {
    'type': 'template',
    'data': {'type': type, 'data': data},
  };

  Map<String, dynamic> _create(Map<String, dynamic> r) {
    final made = {
      'id': 't${_n++}',
      'name': r['name'],
      'subject': r['subject'],
      'body': r['body'],
      'created_at': 1,
      'updated_at': 1,
    };
    templates.add(made);
    return _wrap('template', made);
  }

  Map<String, dynamic> _update(Map<String, dynamic> r) {
    final found = templates.firstWhere((t) => t['id'] == r['template']);
    found
      ..['name'] = r['name']
      ..['subject'] = r['subject']
      ..['body'] = r['body'];
    return _wrap('template', found);
  }

  /// Valeur d'une variable dans le contexte, avec les noms de champs du contrat du moteur.
  String? _value(String name, Map<String, dynamic> context) {
    final contact = (context['contact'] as Map?)?.cast<String, dynamic>();
    final found = switch (name) {
      'contact.nom' => contact?['name'],
      'contact.prénom' => contact?['first_name'],
      'contact.email' => contact?['email'],
      'date' => context['date'],
      'heure' => context['time'],
      'moi.nom' => context['account_name'],
      'moi.email' => context['account_email'],
      'objet' => context['subject'],
      _ => null,
    };
    return found is String && found.isNotEmpty ? found : null;
  }

  /// Rend le texte comme le moteur : variables sans valeur laissées visibles et listées.
  String _fill(String text, Map<String, dynamic> context, List<String> left) {
    return text.replaceAllMapped(RegExp(r'\{\{\s*([^}]+?)\s*\}\}'), (m) {
      final value = _value(m.group(1)!, context);
      if (value != null) return value;
      if (!left.contains(m.group(1)!)) left.add(m.group(1)!);
      return m.group(0)!;
    });
  }

  Map<String, dynamic> _render(Map<String, dynamic> r) {
    final matches = templates.where((t) => t['id'] == r['template']);
    if (matches.isEmpty) {
      throw const EngineException('not_found.template', 'inconnu');
    }
    final found = matches.first;
    final context = (r['context'] as Map).cast<String, dynamic>();
    final left = <String>[];
    final subject = found['subject'] as String?;
    return _wrap('rendered', {
      'subject': subject == null ? null : _fill(subject, context, left),
      'body': _fill('${found['body']}', context, left),
      'unresolved': left,
    });
  }

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    calls.add(request);
    final fail = failNext;
    if (fail != null) {
      failNext = null;
      throw fail;
    }
    final r = (request['request'] as Map).cast<String, dynamic>();
    switch (r['op']) {
      case 'list_templates':
        return _wrap('templates', [...templates]);
      case 'create_template':
        return _create(r);
      case 'update_template':
        return _update(r);
      case 'render_template':
        return _render(r);
      case 'delete_template':
        templates.removeWhere((t) => t['id'] == r['template']);
        return _wrap('done', null);
    }
    return _wrap('done', null);
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();

  @override
  Future<String?> originalHtml(String messageId) async => null;

  @override
  Future<void> close() async {}
}
