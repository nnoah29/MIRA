import 'package:flutter/material.dart';

import '../engine/local_label_admin.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'import_dialog.dart';
import 'rules_file_flow.dart';
import 'widgets.dart';

/// Exporter et importer les libellés locaux dans un fichier YAML ; occupé tant qu'une opération dure.
class LabelsFileBar extends StatefulWidget {
  final LocalLabelAdmin admin;
  final SaveFile saveFile;
  final PickYaml pickFile;
  final DateTime Function() clock;
  const LabelsFileBar({
    super.key,
    required this.admin,
    required this.saveFile,
    required this.pickFile,
    required this.clock,
  });

  @override
  State<LabelsFileBar> createState() => _LabelsFileBarState();
}

class _LabelsFileBarState extends State<LabelsFileBar> {
  bool busy = false;
  String? error;

  /// Lance l'opération si aucune autre n'est en cours ; une erreur est dite à l'utilisateur.
  Future<void> _guard(Future<void> Function() op) async {
    if (busy) return;
    setState(() => busy = true);
    try {
      await op();
      if (mounted) setState(() => error = null);
    } catch (e) {
      if (mounted) setState(() => error = fileErrorText(e));
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  /// Enregistre les libellés dans un fichier YAML choisi par l'utilisateur.
  Future<void> _export() => _guard(() async {
    final yaml = await widget.admin.exportLabelsFile();
    await saveYamlText(widget.saveFile, labelsExportName(widget.clock()), yaml);
  });

  /// Lit un fichier YAML, demande si l'on remplace les libellés existants, importe puis montre le rapport.
  Future<void> _import() => _guard(() async {
    final yaml = await widget.pickFile();
    if (yaml == null || !mounted) return;
    final replace = await showLabelsImportDialog(context);
    if (replace == null) return;
    final report = await widget.admin.importLabelsFile(
      yaml,
      replaceExisting: replace,
    );
    if (!mounted) return;
    await showFileImportReport(
      context,
      report,
      labelsRefusalText,
      labelsWording(),
    );
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Wrap(
          spacing: 10,
          runSpacing: 10,
          children: [
            Btn(
              key: const ValueKey('import-labels'),
              label: t('Importer'),
              onTap: busy ? null : _import,
            ),
            Btn(
              key: const ValueKey('export-labels'),
              label: t('Exporter'),
              onTap: busy ? null : _export,
            ),
          ],
        ),
        if (error != null)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(error!, style: Fonts.sans(size: 13.5, color: p.accent)),
          ),
      ],
    );
  }
}
