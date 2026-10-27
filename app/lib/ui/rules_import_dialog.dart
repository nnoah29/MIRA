import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

export 'rules_import_report.dart';

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

/// Demande s'il faut remplacer les règles existantes ; rend `null` si l'on annule.
Future<bool?> showRulesImportDialog(BuildContext context) =>
    showDialog<bool>(context: context, builder: (_) => const _ImportOptions());

class _ImportOptions extends StatefulWidget {
  const _ImportOptions();

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
          t('Importer des règles'),
          style: Fonts.sans(size: 16, w: FontWeight.w600, color: p.text),
        ),
        const SizedBox(height: 10),
        Text(
          t(
            'Les règles déjà présentes sont laissées telles quelles, sauf si tu coches la case.',
          ),
          style: Fonts.sans(size: 13.5, color: p.dim),
        ),
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
                t('Remplacer les règles existantes'),
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
