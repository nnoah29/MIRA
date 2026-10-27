import 'package:flutter/material.dart';

import '../data/rule_import.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'rules_import_dialog.dart';
import 'widgets.dart';

/// Résumé chiffré d'un import, sans les compteurs à zéro.
String importSummary(RuleImportReport r) {
  final parts = [
    if (r.imported > 0) tn(r.imported, 'importée', 'importées'),
    if (r.replaced > 0) tn(r.replaced, 'remplacée', 'remplacées'),
    if (r.alreadyPresent > 0)
      tn(r.alreadyPresent, 'déjà présente', 'déjà présentes'),
    if (r.refused > 0) tn(r.refused, 'refusée', 'refusées'),
  ];
  return parts.isEmpty ? t('Aucune règle dans le fichier.') : parts.join(', ');
}

/// Raison, dans la langue active, du refus d'une règle du fichier.
String refusalText(RuleImportEntry e) => switch (e.reason) {
  'unknown_account' => t('Compte inconnu'),
  'unknown_mailbox' => t('Boîte inconnue'),
  'unknown_label' => t('Libellé inconnu'),
  'unknown_template' => t('Modèle inconnu'),
  _ => t(engineErrorTexts[e.code] ?? 'Règle invalide'),
};

/// Rapport d'un import : le résumé, puis chaque règle refusée avec sa raison.
class RulesImportReportView extends StatelessWidget {
  final RuleImportReport report;
  const RulesImportReportView({super.key, required this.report});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          importSummary(report),
          style: Fonts.sans(size: 14, w: FontWeight.w500, color: p.text),
        ),
        for (final e in report.entries.where((e) => e.refused))
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(
              t('Règle {0} « {1} » : {2}', [
                e.position,
                e.name,
                refusalText(e),
              ]),
              style: Fonts.sans(size: 13.5, color: p.accent),
            ),
          ),
      ],
    );
  }
}

/// Montre le rapport d'un import dans une boîte de dialogue.
Future<void> showRulesImportReport(
  BuildContext context,
  RuleImportReport report,
) => showDialog<void>(
  context: context,
  builder: (c) => ImportDialogShell(
    children: [
      Text(
        t('Résultat de l’import'),
        style: Fonts.sans(size: 16, w: FontWeight.w600, color: c.pal.text),
      ),
      const SizedBox(height: 12),
      RulesImportReportView(report: report),
      const SizedBox(height: 16),
      Align(
        alignment: Alignment.centerRight,
        child: Btn(label: t('Fermer'), onTap: () => Navigator.pop(c)),
      ),
    ],
  ),
);
