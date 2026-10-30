import 'dart:async';

import 'package:mira/engine/engine_client.dart';

/// Moteur factice du domaine `ai` : garde les requêtes, tient les connexions en mémoire, rejoue des flux de chat.
class FakeAiEngine implements EngineClient, ChatStreamingClient {
  final calls = <Map<String, dynamic>>[];
  final chatStarts = <Map<String, dynamic>>[];
  final providers = <Map<String, dynamic>>[];
  final connections = <Map<String, dynamic>>[];
  final models = <Map<String, dynamic>>[];
  final scripts = <List<Map<String, dynamic>>>[];
  final journal = <Map<String, dynamic>>[];
  StreamController<Map<String, dynamic>>? live;
  bool manual = false;
  int cancelledStreams = 0;
  EngineException? failNext;
  EngineException? failOp;
  String? failOpName;
  int _n = 0;

  /// Requêtes reçues pour cette opération.
  List<Map<String, dynamic>> requestsOf(String op) => [
    for (final c in calls)
      if ((c['request'] as Map)['op'] == op)
        (c['request'] as Map).cast<String, dynamic>(),
  ];

  Map<String, dynamic> _wrap(String type, Object? data) => {
    'type': 'ai',
    'data': {'type': type, 'data': data},
  };

  Map<String, dynamic> _find(Object? id) =>
      connections.firstWhere((c) => c['id'] == id);

  Map<String, dynamic> _add(Map<String, dynamic> r) {
    final made = {
      'id': 'ai-${_n++}',
      'provider': r['provider'],
      'label': r['label'],
      'base_url': r['base_url'],
      'default_model': null,
      'is_default': false,
      'has_key': false,
      'consent_at': null,
      'models': null,
      'allowed_kinds': <String>[],
      'masking': {
        'mode': 'mask',
        'categories': ['iban', 'card', 'secret', 'otp'],
      },
    };
    connections.add(made);
    return _wrap('connection', made);
  }

  Map<String, dynamic> _update(Map<String, dynamic> r) {
    final found = _find(r['connection']);
    found
      ..['label'] = r['label']
      ..['base_url'] = r['base_url']
      ..['default_model'] = r['default_model'];
    return _wrap('connection', found);
  }

  Map<String, dynamic> _setDefault(Object? id) {
    for (final c in connections) {
      c['is_default'] = c['id'] == id;
    }
    return _wrap('connection', _find(id));
  }

  Map<String, dynamic> _mark(Object? id, String field, Object? value) {
    final found = _find(id)..[field] = value;
    return _wrap('connection', found);
  }

  Map<String, dynamic> _answer(Map<String, dynamic> r) => switch (r['op']) {
    'list_ai_providers' => _wrap('providers', [...providers]),
    'list_ai_connections' => _wrap('connections', [...connections]),
    'add_ai_connection' => _add(r),
    'update_ai_connection' => _update(r),
    'set_default_ai_connection' => _setDefault(r['connection']),
    'set_ai_api_key' => _mark(r['connection'], 'has_key', true),
    'clear_ai_api_key' => _mark(r['connection'], 'has_key', false),
    'acknowledge_ai_consent' => _consent(r),
    'set_ai_allowed_content' => _mark(
      r['connection'],
      'allowed_kinds',
      r['kinds'],
    ),
    'set_ai_masking' => _mark(r['connection'], 'masking', r['masking']),
    'list_ai_journal' => _wrap('journal', _page(r)),
    'clear_ai_journal' => _cleared(),
    'list_ai_models' => _wrap('models', {
      'models': [...models],
      'fetched_at': 1,
    }),
    'remove_ai_connection' => _removed(r['connection']),
    _ => _wrap('done', null),
  };

  Map<String, dynamic> _consent(Map<String, dynamic> r) {
    final found = _find(r['connection'])
      ..['consent_at'] = 1
      ..['allowed_kinds'] = [...?(r['kinds'] as List?)];
    return _wrap('connection', found);
  }

  List<Map<String, dynamic>> _page(Map<String, dynamic> r) {
    final from = r['before'] == null
        ? 0
        : journal.indexWhere((e) => e['id'] == r['before']) + 1;
    return journal.skip(from).take(r['limit'] as int).toList();
  }

  Map<String, dynamic> _cleared() {
    journal.clear();
    return _wrap('done', null);
  }

  Map<String, dynamic> _removed(Object? id) {
    connections.removeWhere((c) => c['id'] == id);
    return _wrap('done', null);
  }

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    calls.add(request);
    final r = (request['request'] as Map).cast<String, dynamic>();
    final fail = failNext;
    if (fail != null) {
      failNext = null;
      throw fail;
    }
    if (failOp != null && failOpName == r['op']) throw failOp!;
    return _answer(r);
  }

  @override
  Stream<Map<String, dynamic>> chatStream(Map<String, dynamic> start) {
    chatStarts.add(start);
    if (manual) {
      final c = live = StreamController<Map<String, dynamic>>(
        onCancel: () => cancelledStreams++,
      );
      return c.stream;
    }
    final lines = scripts.isEmpty
        ? <Map<String, dynamic>>[]
        : scripts.removeAt(0);
    return Stream.fromIterable(lines);
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();

  @override
  Future<String?> originalHtml(String messageId) async => null;

  @override
  Future<void> close() async {}
}
