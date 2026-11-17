//! Boîtes de dialogue de la mémoire : modifier un souvenir, oublier un souvenir, tout oublier.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'chat_history_dialogs.dart';
import 'widgets.dart';

/// Taille maximale d'un souvenir, en caractères, comme le moteur.
const memoryTextMax = 1000;

/// Demande le nouveau texte du souvenir ; rend le texte rogné, ou `null` si l'utilisateur annule.
Future<String?> showEditMemoryDialog(BuildContext context, String current) =>
    showDialog<String>(
      context: context,
      builder: (_) => _EditMemoryDialog(current),
    );

/// Demande de confirmer l'oubli d'un souvenir.
Future<bool> showForgetMemoryDialog(BuildContext context, String text) =>
    showConfirmDialog(
      context,
      confirmKey: 'confirm-forget-memory',
      title: t('Oublier ce souvenir ?'),
      body: t('« {0} » est effacé de cet appareil.', [text]),
      action: t('Oublier'),
    );

/// Demande de confirmer l'oubli de tous les souvenirs.
Future<bool> showForgetAllMemoriesDialog(BuildContext context) =>
    showConfirmDialog(
      context,
      confirmKey: 'confirm-forget-all-memories',
      title: t('Tout oublier ?'),
      body: t('Tous les souvenirs sont effacés de cet appareil.'),
      action: t('Tout oublier'),
    );

class _EditMemoryDialog extends StatefulWidget {
  final String current;
  const _EditMemoryDialog(this.current);

  @override
  State<_EditMemoryDialog> createState() => _EditMemoryDialogState();
}

class _EditMemoryDialogState extends State<_EditMemoryDialog> {
  late final TextEditingController text = TextEditingController(
    text: widget.current,
  )..addListener(() => setState(() {}));

  @override
  void dispose() {
    text.dispose();
    super.dispose();
  }

  bool get valid => text.text.trim().isNotEmpty;

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return DialogShell(
      title: t('Modifier le souvenir'),
      body: TextField(
        key: const ValueKey('ai-memory-edit-field'),
        controller: text,
        autofocus: true,
        maxLines: 4,
        minLines: 2,
        maxLength: memoryTextMax,
        style: Fonts.sans(size: 15, color: p.text),
        cursorColor: p.accent,
      ),
      actions: [
        Btn(label: t('Annuler'), onTap: () => Navigator.pop(context)),
        const SizedBox(width: 10),
        Btn(
          key: const ValueKey('confirm-edit-memory'),
          label: t('Enregistrer'),
          go: true,
          onTap: valid ? () => Navigator.pop(context, text.text.trim()) : null,
        ),
      ],
    );
  }
}
