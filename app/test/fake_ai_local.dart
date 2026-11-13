import 'package:mira/engine/engine_client.dart';

/// Modèle local d'un moteur factice : trois paliers, une installation suivie par une opération scriptée.
class FakeAiLocal {
  bool supported = true;
  String? running;
  String? installing;
  final installed = <String>{};
  final memoryWarning = <String>{};
  final script = <Map<String, dynamic>>[];
  EngineException? failInstall;

  Map<String, dynamic> _model(String id, String name, String tier, int size) =>
      {
        'id': id,
        'display_name': name,
        'tier': tier,
        'size': size,
        'license': 'Apache-2.0',
        'supports_tools': tier == 'advanced',
        'ram_recommended_mib': tier == 'advanced' ? 7000 : 1300,
        'installed': installed.contains(id),
        'memory_warning': memoryWarning.contains(id),
      };

  Map<String, dynamic> _status() => {
    'supported': supported,
    'models': supported
        ? [
            _model('mini', 'Granite 4.0 H 1B', 'mini', 901162208),
            _model('light', 'Ministral 3 3B', 'light', 2147023008),
            _model('big', 'Granite 4.2 8B', 'advanced', 5347917952),
          ]
        : <Map<String, dynamic>>[],
    'server_installed': installed.isNotEmpty,
    'running_model': running,
    'memory_available_mib': 5000,
    'installing': installing == null
        ? null
        : {'model_id': installing, 'operation': 'op-local'},
  };

  Map<String, dynamic> _operation(Map<String, dynamic> status) => {
    'type': 'operation',
    'data': {'id': 'op-local', 'kind': 'ai_local_install', 'status': status},
  };

  Map<String, dynamic> _progress() {
    final next = script.length > 1
        ? script.removeAt(0)
        : (script.isEmpty ? {'type': 'finished'} : script.first);
    if (next['type'] == 'finished' && installing != null) {
      installed.add(installing!);
      installing = null;
    }
    return _operation(next);
  }

  Map<String, dynamic> _install(Map<String, dynamic> r) {
    final fail = failInstall;
    if (fail != null) throw fail;
    installing = '${r['model_id']}';
    return _operation({'type': 'running', 'done': 0, 'total': 100});
  }

  Map<String, dynamic> _done() => {
    'type': 'ai',
    'data': {'type': 'done', 'data': null},
  };

  /// Réponse à une requête du modèle local, ou `null` si elle n'en est pas une.
  Map<String, dynamic>? answer(Map<String, dynamic> r) => switch (r['op']) {
    'get_local_runtime_status' => {
      'type': 'ai',
      'data': {'type': 'local_runtime', 'data': _status()},
    },
    'install_local_model' => _install(r),
    'get_operation' => _progress(),
    'cancel_operation' => _cancelled(),
    'remove_local_model' => _removed(r),
    'stop_local_runtime' => _stopped(),
    _ => null,
  };

  Map<String, dynamic> _cancelled() {
    installing = null;
    return _operation({'type': 'cancelled'});
  }

  Map<String, dynamic> _removed(Map<String, dynamic> r) {
    installed.remove('${r['model_id']}');
    return _done();
  }

  Map<String, dynamic> _stopped() {
    running = null;
    return _done();
  }
}
