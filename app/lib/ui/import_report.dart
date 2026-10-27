import 'package:flutter/material.dart';

import '../data/file_import.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'import_dialog.dart';
import 'widgets.dart';

/// Mots d'un rapport d'import, accordés à ce qu'on importe : règles ou libellés.
class ImportWording {
  final String Function(int position, String name, String reason) entry;
  final String Function(int n) imported, replaced, present, refused;
  const ImportWording(
    this.entry,
    this.imported,
    this.replaced,
    this.present,
    this.refused,
  );
}

/// Mots du rapport d'un import de règles (féminin).
ImportWording rulesWording() => ImportWording(
  (position, name, reason) =>
      t('Règle {0} « {1} » : {2}', [position, name, reason]),
  (n) => tn(n, 'importée', 'importées'),
  (n) => tn(n, 'remplacée', 'remplacées'),
  (n) => tn(n, 'déjà présente', 'déjà présentes'),
  (n) => tn(n, 'refusée', 'refusées'),
);

/// Mots du rapport d'un import de libellés (masculin).
ImportWording labelsWording() => ImportWording(
  (position, name, reason) =>
      t('Libellé {0} « {1} » : {2}', [position, name, reason]),
  (n) => tn(n, 'importé', 'importés'),
  (n) => tn(n, 'remplacé', 'remplacés'),
  (n) => tn(n, 'déjà présent', 'déjà présents'),
  (n) => tn(n, 'refusé', 'refusés'),
);

/// Résumé chiffré d'un import, sans les compteurs à zéro.
String importSummary(FileImportReport r, ImportWording w) {
  final parts = [
    if (r.imported > 0) w.imported(r.imported),
    if (r.replaced > 0) w.replaced(r.replaced),
    if (r.alreadyPresent > 0) w.present(r.alreadyPresent),
    if (r.refused > 0) w.refused(r.refused),
  ];
  return parts.isEmpty
      ? t('Rien à importer dans le fichier.')
      : parts.join(', ');
}

/// Raison, dans la langue active, du refus d'un libellé du fichier.
String labelsRefusalText(FileImportEntry e) => switch (e.reason) {
  'invalid_name' => t('Nom invalide'),
  'invalid_color' => t('Couleur invalide'),
  'invalid_label' => t('Libellé illisible'),
  'duplicate_in_file' => t('Nom déjà présent plus haut dans le fichier'),
  'too_many_labels' => t('Trop de libellés'),
  _ => t('Libellé refusé'),
};

/// Raison, dans la langue active, du refus d'une règle du fichier.
String rulesRefusalText(FileImportEntry e) => switch (e.reason) {
  'unknown_account' => t('Compte inconnu'),
  'unknown_mailbox' => t('Boîte inconnue'),
  'unknown_label' => t('Libellé inconnu'),
  'unknown_template' => t('Modèle inconnu'),
  _ => t(engineErrorTexts[e.code] ?? 'Règle invalide'),
};

/// Rapport d'un import : le résumé, puis chaque entrée refusée avec la raison que dit [refusal].
class FileImportReportView extends StatelessWidget {
  final FileImportReport report;
  final String Function(FileImportEntry) refusal;
  final ImportWording wording;
  const FileImportReportView({
    super.key,
    required this.report,
    required this.refusal,
    required this.wording,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          importSummary(report, wording),
          style: Fonts.sans(size: 14, w: FontWeight.w500, color: p.text),
        ),
        for (final e in report.entries.where((e) => e.refused))
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(
              wording.entry(e.position, e.name, refusal(e)),
              style: Fonts.sans(size: 13.5, color: p.accent),
            ),
          ),
      ],
    );
  }
}

/// Montre le rapport d'un import dans une boîte de dialogue.
Future<void> showFileImportReport(
  BuildContext context,
  FileImportReport report,
  String Function(FileImportEntry) refusal,
  ImportWording wording,
) => showDialog<void>(
  context: context,
  builder: (c) => ImportDialogShell(
    children: [
      Text(
        t('Résultat de l’import'),
        style: Fonts.sans(size: 16, w: FontWeight.w600, color: c.pal.text),
      ),
      const SizedBox(height: 12),
      FileImportReportView(report: report, refusal: refusal, wording: wording),
      const SizedBox(height: 16),
      Align(
        alignment: Alignment.centerRight,
        child: Btn(label: t('Fermer'), onTap: () => Navigator.pop(c)),
      ),
    ],
  ),
);
