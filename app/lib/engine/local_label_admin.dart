import 'dart:math';

import '../data/file_import.dart';
import 'engine_client.dart';

/// Libellé local tel que le moteur le tient : identifiant, nom, couleur ARGB.
class EngineLocalLabel {
  final String id;
  final String name;
  final int color;
  const EngineLocalLabel(this.id, this.name, this.color);

  factory EngineLocalLabel.fromJson(Map<String, dynamic> json) =>
      EngineLocalLabel(
        '${json['id']}',
        '${json['name']}',
        (json['color'] as num).toInt(),
      );
}

/// Un libellé de l'app à reprendre dans le moteur, avec les conversations qui le portent.
class LocalLabelImport {
  final String name;
  final int color;
  final List<String> conversations;
  const LocalLabelImport(this.name, this.color, this.conversations);

  Map<String, dynamic> toJson() => {
    'name': name,
    'color': color,
    'conversations': conversations,
  };
}

/// Bilan d'une reprise : libellés créés ou réutilisés, poses reprises, clés ignorées.
class ImportReport {
  final int labels;
  final int assignments;
  final int ignored;
  const ImportReport(this.labels, this.assignments, this.ignored);
}

/// Nombre maximal de conversations que le moteur accepte dans une seule pose.
const maxThreadsPerCall = 1000;

/// Libellés locaux portés par le moteur : lecture, écritures idempotentes, poses, reprise.
class LocalLabelAdmin {
  final EngineClient client;
  int _seq = 0;
  final String _run = Random().nextInt(1 << 32).toRadixString(36);
  LocalLabelAdmin(this.client);

  String _key(String what) =>
      'ui-label-$what-$_run-${DateTime.now().microsecondsSinceEpoch}-${_seq++}';

  Future<Object?> _send(String op, [Map<String, dynamic>? data]) async {
    final r = await client.call(req('local_label', op, data ?? const {}));
    return (r['data'] as Map)['data'];
  }

  /// Tous les libellés locaux, par nom.
  Future<List<EngineLocalLabel>> list() async {
    final items = await _send('list_local_labels') as List;
    return [
      for (final v in items)
        EngineLocalLabel.fromJson((v as Map).cast<String, dynamic>()),
    ];
  }

  /// Libellés posés sur chaque conversation, par identifiant de conversation.
  Future<Map<String, Set<String>>> assignments() async {
    final items = await _send('list_local_label_assignments') as List;
    return {
      for (final v in items)
        '${(v as Map)['thread']}': {for (final l in v['labels'] as List) '$l'},
    };
  }

  /// Crée un libellé (le moteur choisit l'identifiant).
  Future<EngineLocalLabel> create(String name, int color) async {
    final r = await _send('create_local_label', {
      'name': name,
      'color': color,
      'idempotency_key': _key('create'),
    });
    return EngineLocalLabel.fromJson((r as Map).cast<String, dynamic>());
  }

  /// Renomme un libellé.
  Future<void> rename(String id, String name) =>
      _send('rename_local_label', {'label': id, 'name': name});

  /// Change la couleur d'un libellé.
  Future<void> recolor(String id, int color) =>
      _send('recolor_local_label', {'label': id, 'color': color});

  /// Supprime un libellé et toutes ses poses.
  Future<void> delete(String id) => _send('delete_local_label', {'label': id});

  /// Pose (`on`) ou retire un libellé sur des conversations, par lots que le moteur accepte (tout ou rien par lot).
  Future<void> set(String id, List<String> threads, bool on) async {
    for (var i = 0; i < threads.length; i += maxThreadsPerCall) {
      final end = min(i + maxThreadsPerCall, threads.length);
      await _send('set_local_label', {
        'label': id,
        'threads': threads.sublist(i, end),
        'on': on,
        'idempotency_key': _key('set'),
      });
    }
  }

  /// Reprend les libellés de l'app et leurs poses ; rend le bilan.
  Future<ImportReport> importLabels(List<LocalLabelImport> labels) async {
    final r = await _send('import_local_labels', {
      'labels': [for (final l in labels) l.toJson()],
      'idempotency_key': _key('import'),
    });
    final m = (r as Map).cast<String, dynamic>();
    return ImportReport(
      (m['labels'] as num).toInt(),
      (m['assignments'] as num).toInt(),
      (m['ignored'] as num).toInt(),
    );
  }

  /// Les libellés en texte YAML portable (nom et couleur), pour un fichier.
  Future<String> exportLabelsFile() async {
    final r = await _send('export_local_labels_file') as Map;
    return r['yaml'] as String;
  }

  /// Importe le texte d'un fichier YAML ; les libellés déjà présents sont ignorés sauf [replaceExisting].
  Future<FileImportReport> importLabelsFile(
    String yaml, {
    bool replaceExisting = false,
  }) async {
    final r = await _send('import_local_labels_file', {
      'yaml': yaml,
      'replace_existing': replaceExisting,
      'idempotency_key': _key('file-import'),
    });
    return FileImportReport.fromJson((r as Map).cast<String, dynamic>());
  }
}
