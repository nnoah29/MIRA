//! Boîtes de dialogue de l'historique : renommer, supprimer une conversation, tout effacer.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../i18n/i18n.dart';
import '../state/chat_conversation.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Demande le nouveau titre ; rend le texte rogné, ou `null` si l'utilisateur annule.
Future<String?> showRenameDialog(BuildContext context, String current) =>
    showDialog<String>(
      context: context,
      builder: (_) => _RenameDialog(current),
    );

/// Demande de confirmer la suppression de la conversation [title].
Future<bool> showDeleteDialog(BuildContext context, String title) => _confirm(
  context,
  confirmKey: 'confirm-delete-conversation',
  title: t('Supprimer cette conversation ?'),
  body: t('« {0} » est effacée de cet appareil.', [title]),
  action: t('Supprimer'),
);

/// Demande de confirmer l'effacement de toutes les conversations.
Future<bool> showClearHistoryDialog(BuildContext context) => _confirm(
  context,
  confirmKey: 'confirm-clear-history',
  title: t('Effacer tout l’historique ?'),
  body: t(
    'Toutes les conversations enregistrées sont effacées de cet appareil.',
  ),
  action: t('Effacer'),
);

Future<bool> _confirm(
  BuildContext context, {
  required String confirmKey,
  required String title,
  required String body,
  required String action,
}) async {
  final ok = await showDialog<bool>(
    context: context,
    builder: (d) => _Shell(
      title: title,
      body: Text(body, style: Fonts.sans(size: 13.5, color: d.pal.dim)),
      actions: [
        Btn(label: t('Annuler'), onTap: () => Navigator.pop(d, false)),
        const SizedBox(width: 10),
        Btn(
          key: ValueKey(confirmKey),
          label: action,
          go: true,
          onTap: () => Navigator.pop(d, true),
        ),
      ],
    ),
  );
  return ok == true;
}

class _Shell extends StatelessWidget {
  final String title;
  final Widget body;
  final List<Widget> actions;
  const _Shell({
    required this.title,
    required this.body,
    required this.actions,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Dialog(
      backgroundColor: p.bgSide,
      shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 420),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: Fonts.sans(size: 16, w: FontWeight.w600, color: p.text),
              ),
              const SizedBox(height: 8),
              body,
              const SizedBox(height: 20),
              Row(mainAxisAlignment: MainAxisAlignment.end, children: actions),
            ],
          ),
        ),
      ),
    );
  }
}

/// Coupe le texte saisi à 80 caractères comptés comme le moteur (caractères Unicode, pas unités UTF-16).
final _titleLimit = TextInputFormatter.withFunction((old, value) {
  if (value.text.runes.length <= conversationTitleMax) return value;
  final cut = String.fromCharCodes(value.text.runes.take(conversationTitleMax));
  return TextEditingValue(
    text: cut,
    selection: TextSelection.collapsed(offset: cut.length),
  );
});

class _RenameDialog extends StatefulWidget {
  final String current;
  const _RenameDialog(this.current);

  @override
  State<_RenameDialog> createState() => _RenameDialogState();
}

class _RenameDialogState extends State<_RenameDialog> {
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
    return _Shell(
      title: t('Renommer la conversation'),
      body: TextField(
        key: const ValueKey('chat-history-rename-field'),
        controller: text,
        autofocus: true,
        inputFormatters: [_titleLimit],
        style: Fonts.sans(size: 15, color: p.text),
        cursorColor: p.accent,
        onSubmitted: (_) =>
            valid ? Navigator.pop(context, text.text.trim()) : null,
      ),
      actions: [
        Btn(label: t('Annuler'), onTap: () => Navigator.pop(context)),
        const SizedBox(width: 10),
        Btn(
          key: const ValueKey('confirm-rename-conversation'),
          label: t('Renommer'),
          go: true,
          onTap: valid ? () => Navigator.pop(context, text.text.trim()) : null,
        ),
      ],
    );
  }
}
