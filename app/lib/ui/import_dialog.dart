import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

export 'import_report.dart';

/// Cadre commun des boîtes de dialogue d'import : fond du panneau, filet, largeur bornée.
class ImportDialogShell extends StatelessWidget {
  final List<Widget> children;
  const ImportDialogShell({super.key, required this.children});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Dialog(
      backgroundColor: p.bgSide,
      shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 460),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: children,
          ),
        ),
      ),
    );
  }
}

/// Demande s'il faut remplacer les éléments existants ; rend `null` si l'on annule.
Future<bool?> showImportOptionsDialog(
  BuildContext context, {
  required String title,
  required String note,
  required String replaceLabel,
}) => showDialog<bool>(
  context: context,
  builder: (_) =>
      _ImportOptions(title: title, note: note, replaceLabel: replaceLabel),
);

/// Boîte d'import des règles ; rend l'état de la case « Remplacer », `null` si l'on annule.
Future<bool?> showRulesImportDialog(BuildContext context) =>
    showImportOptionsDialog(
      context,
      title: t('Importer des règles'),
      note: t(
        'Les règles déjà présentes sont laissées telles quelles, sauf si tu coches la case.',
      ),
      replaceLabel: t('Remplacer les règles existantes'),
    );

/// Boîte d'import des libellés ; rend l'état de la case « Remplacer », `null` si l'on annule.
Future<bool?> showLabelsImportDialog(BuildContext context) =>
    showImportOptionsDialog(
      context,
      title: t('Importer des libellés'),
      note: t(
        'Les libellés déjà présents sont laissés tels quels, sauf si tu coches la case.',
      ),
      replaceLabel: t('Remplacer les libellés existants'),
    );

class _ImportOptions extends StatefulWidget {
  final String title, note, replaceLabel;
  const _ImportOptions({
    required this.title,
    required this.note,
    required this.replaceLabel,
  });

  @override
  State<_ImportOptions> createState() => _ImportOptionsState();
}

class _ImportOptionsState extends State<_ImportOptions> {
  bool replace = false;

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return ImportDialogShell(
      children: [
        Text(
          widget.title,
          style: Fonts.sans(size: 16, w: FontWeight.w600, color: p.text),
        ),
        const SizedBox(height: 10),
        Text(widget.note, style: Fonts.sans(size: 13.5, color: p.dim)),
        const SizedBox(height: 12),
        Row(
          children: [
            Checkbox(
              key: const ValueKey('import-replace'),
              value: replace,
              onChanged: (v) => setState(() => replace = v ?? false),
            ),
            Expanded(
              child: Text(
                widget.replaceLabel,
                style: Fonts.sans(size: 14, color: p.text),
              ),
            ),
          ],
        ),
        const SizedBox(height: 16),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            Btn(
              key: const ValueKey('import-cancel'),
              label: t('Annuler'),
              onTap: () => Navigator.pop(context),
            ),
            const SizedBox(width: 10),
            Btn(
              key: const ValueKey('import-confirm'),
              label: t('Importer'),
              go: true,
              onTap: () => Navigator.pop(context, replace),
            ),
          ],
        ),
      ],
    );
  }
}
