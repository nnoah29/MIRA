import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/attachment_save.dart';
import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Fait choisir un dossier ; `null` si l'on annule.
typedef PickDirectory = Future<String?> Function();

/// Boîte de choix de dossier du système.
Future<String?> pickDirectoryWithPicker() => FilePicker.getDirectoryPath();

/// Enregistre toutes les pièces d'un message dans un dossier au choix ; occupé tant que cela dure.
class DownloadAllButton extends StatefulWidget {
  final List<Attachment> list;
  final PickDirectory pickDirectory;
  const DownloadAllButton(
    this.list, {
    super.key,
    this.pickDirectory = pickDirectoryWithPicker,
  });

  @override
  State<DownloadAllButton> createState() => _DownloadAllButtonState();
}

class _DownloadAllButtonState extends State<DownloadAllButton> {
  bool busy = false;

  /// Choisit le dossier, enregistre les pièces et dit le résultat ; sans effet si déjà occupé.
  Future<void> _run() async {
    if (busy) return;
    final store = context.read<MailStore>();
    setState(() => busy = true);
    try {
      final path = await widget.pickDirectory();
      if (path == null) return;
      final report = await saveAttachments(
        widget.list,
        Directory(path),
        (a) async => a.bytes ?? await store.source?.attachmentBytes(a),
      );
      store.toast(_summary(report), err: report.failed.isNotEmpty);
    } on Object {
      store.toast(t('Enregistrement impossible'), err: true);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  /// Phrase de résultat : nombre de pièces enregistrées, puis les noms de celles qui ont échoué.
  String _summary(SaveReport report) {
    final done = tn(
      report.saved.length,
      'pièce enregistrée',
      'pièces enregistrées',
    );
    if (report.failed.isEmpty) return done;
    return '$done · ${t('échec : {0}', [report.failed.join(', ')])}';
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      tip: t('Tout télécharger'),
      onTap: busy ? null : _run,
      builder: (_, h) => DashedBox(
        color: h ? p.accent : p.line,
        fill: p.bg,
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Ico('download', color: h ? p.text : p.muted),
            const SizedBox(width: 12),
            Text(
              t('Tout télécharger'),
              style: Fonts.sans(size: 15, color: h ? p.text : p.muted),
            ),
          ],
        ),
      ),
    );
  }
}
