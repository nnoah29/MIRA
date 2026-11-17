//! Requêtes du modèle d'IA local : état, installation (opération longue), retrait, arrêt.

import 'ai_admin.dart';
import 'ai_local_models.dart';
import 'engine_client.dart';

export 'ai_local_models.dart';

/// Modèle local du moteur ; l'installation est une opération que l'on suit puis qu'on peut annuler.
extension AiLocalAdmin on AiAdmin {
  Future<Map<String, dynamic>> _call(
    String domain,
    String op, [
    Map<String, dynamic>? data,
  ]) async {
    final r = await client.call(req(domain, op, data ?? const {}));
    return (r['data'] as Map).cast<String, dynamic>();
  }

  /// État du modèle local ; lecture seule, rien n'est téléchargé ni lancé.
  Future<LocalRuntimeStatus> localStatus() async {
    final r = await _call('ai', 'get_local_runtime_status');
    return LocalRuntimeStatus.fromJson(
      (r['data'] as Map).cast<String, dynamic>(),
    );
  }

  /// Lance l'installation d'un modèle ; rend l'identifiant de l'opération qui la porte.
  Future<String> installLocal(String modelId) async {
    final r = await _call('ai', 'install_local_model', {'model_id': modelId});
    return '${r['id']}';
  }

  /// Supprime un modèle installé.
  Future<void> removeLocal(String modelId) =>
      _call('ai', 'remove_local_model', {'model_id': modelId});

  /// Arrête le serveur local pour rendre la mémoire.
  Future<void> stopLocal() => _call('ai', 'stop_local_runtime');

  /// Avancement d'une opération du moteur.
  Future<OperationProgress> operationProgress(String operation) async {
    final r = await _call('operation', 'get_operation', {
      'operation': operation,
    });
    return OperationProgress.fromJson(r);
  }
}
