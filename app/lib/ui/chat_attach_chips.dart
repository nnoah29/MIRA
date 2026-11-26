//! Vignettes des mails et fichiers joints : avec croix avant l'envoi, sans croix dans le message envoyé.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Une vignette : icône, libellé sur une ligne et, si [onRemove] est donné, une croix de retrait.
class ChatAttachChip extends StatelessWidget {
  final String id, icon, label;
  final VoidCallback? onRemove;
  const ChatAttachChip({
    super.key,
    required this.id,
    required this.icon,
    required this.label,
    this.onRemove,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      key: ValueKey('chat-attach-chip-$id'),
      constraints: const BoxConstraints(maxWidth: 280),
      padding: EdgeInsets.fromLTRB(10, 6, onRemove == null ? 10 : 4, 6),
      decoration: BoxDecoration(
        color: p.bg,
        border: Border.all(color: p.line),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Ico(icon, size: 14, color: p.muted),
          const SizedBox(width: 8),
          Flexible(
            child: Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: Fonts.sans(size: 13, color: p.text),
            ),
          ),
          if (onRemove != null) ...[
            const SizedBox(width: 4),
            IBtn(
              'x',
              key: ValueKey('chat-attach-remove-$id'),
              tip: t('Retirer'),
              onTap: onRemove!,
            ),
          ],
        ],
      ),
    );
  }
}

/// Rangée de vignettes au-dessus du champ de saisie ; vide quand rien n'est joint.
class ChatAttachChips extends StatelessWidget {
  final MailStore store;
  const ChatAttachChips({super.key, required this.store});

  @override
  Widget build(BuildContext context) {
    final mails = store.chatAttached;
    final files = store.chatFiles;
    if (mails.isEmpty && files.isEmpty) return const SizedBox.shrink();
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
      child: Wrap(
        spacing: 8,
        runSpacing: 8,
        children: [
          for (final m in mails)
            ChatAttachChip(
              id: m.id,
              icon: 'inbox',
              label: '${m.from.name} · ${m.subject}',
              onRemove: () => store.detachChatMail(m.id),
            ),
          for (final f in files)
            ChatAttachChip(
              id: f.id,
              icon: 'file',
              label: f.name,
              onRemove: () => store.detachChatFile(f.id),
            ),
        ],
      ),
    );
  }
}

/// Vignettes sans croix des pièces jointes d'un message déjà envoyé.
class ChatSentAttachments extends StatelessWidget {
  final List<ChatAttachmentRef> attachments;
  const ChatSentAttachments({super.key, required this.attachments});

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 10),
    child: Wrap(
      key: const ValueKey('chat-sent-attachments'),
      spacing: 8,
      runSpacing: 8,
      children: [
        for (var i = 0; i < attachments.length; i++)
          ChatAttachChip(
            id: 'sent-$i',
            icon: attachments[i].kind == 'file' ? 'file' : 'inbox',
            label: attachments[i].label,
          ),
      ],
    ),
  );
}
