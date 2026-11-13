//! État du modèle d'IA local tel que le moteur le décrit : modèles proposés, installation en cours, mémoire.

/// Un modèle proposé : palier, taille, licence, ce qu'il sait faire, et son état sur cette machine.
class LocalModelStatus {
  final String id;
  final String name;
  final String tier;
  final int size;
  final String license;
  final bool supportsTools;
  final int ramRecommendedMib;
  final bool installed;
  final bool memoryWarning;
  const LocalModelStatus({
    required this.id,
    required this.name,
    required this.tier,
    required this.size,
    required this.license,
    required this.supportsTools,
    required this.ramRecommendedMib,
    required this.installed,
    required this.memoryWarning,
  });

  factory LocalModelStatus.fromJson(Map<String, dynamic> json) =>
      LocalModelStatus(
        id: '${json['id']}',
        name: '${json['display_name']}',
        tier: '${json['tier']}',
        size: (json['size'] as num?)?.toInt() ?? 0,
        license: '${json['license']}',
        supportsTools: json['supports_tools'] == true,
        ramRecommendedMib: (json['ram_recommended_mib'] as num?)?.toInt() ?? 0,
        installed: json['installed'] == true,
        memoryWarning: json['memory_warning'] == true,
      );
}

/// Installation en cours : le modèle visé et l'opération du moteur qui la porte.
class LocalInstallInfo {
  final String modelId;
  final String operation;
  const LocalInstallInfo(this.modelId, this.operation);
}

/// État complet : `supported` est faux sur les plateformes que le moteur ne décrit pas.
class LocalRuntimeStatus {
  final bool supported;
  final List<LocalModelStatus> models;
  final bool serverInstalled;
  final String? runningModel;
  final int? memoryAvailableMib;
  final LocalInstallInfo? installing;
  const LocalRuntimeStatus({
    required this.supported,
    required this.models,
    required this.serverInstalled,
    this.runningModel,
    this.memoryAvailableMib,
    this.installing,
  });

  /// État d'une machine qui ne prend pas en charge le modèle local.
  static const unsupported = LocalRuntimeStatus(
    supported: false,
    models: [],
    serverInstalled: false,
  );

  factory LocalRuntimeStatus.fromJson(Map<String, dynamic> json) {
    final installing = json['installing'] as Map?;
    return LocalRuntimeStatus(
      supported: json['supported'] == true,
      models: [
        for (final m in (json['models'] as List? ?? const []))
          LocalModelStatus.fromJson((m as Map).cast<String, dynamic>()),
      ],
      serverInstalled: json['server_installed'] == true,
      runningModel: json['running_model'] as String?,
      memoryAvailableMib: (json['memory_available_mib'] as num?)?.toInt(),
      installing: installing == null
          ? null
          : LocalInstallInfo(
              '${installing['model_id']}',
              '${installing['operation']}',
            ),
    );
  }
}

/// Avancement d'une opération longue du moteur.
class OperationProgress {
  final String status;
  final int done;
  final int total;
  final String? code;
  const OperationProgress(this.status, this.done, this.total, [this.code]);

  /// Dit si l'opération court encore.
  bool get running => status == 'running';

  /// Part faite, entre 0 et 1 (0 si le total est inconnu).
  double get fraction => total <= 0 ? 0 : (done / total).clamp(0, 1).toDouble();

  factory OperationProgress.fromJson(Map<String, dynamic> info) {
    final s = (info['status'] as Map).cast<String, dynamic>();
    return OperationProgress(
      '${s['type']}',
      (s['done'] as num?)?.toInt() ?? 0,
      (s['total'] as num?)?.toInt() ?? 0,
      s['code'] as String?,
    );
  }
}
