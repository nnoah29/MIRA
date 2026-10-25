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

  Map<String, dynamic> _render(Map<String, dynamic> r) {
    final found = templates.firstWhere((t) => t['id'] == r['template']);
    return _wrap('rendered', {
      'subject': found['subject'],
      'body': found['body'],
      'unresolved': <String>[],
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
