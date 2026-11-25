//! Vignettes des mails joints au prochain message de l'assistant, chacune avec sa croix de retrait.

import 'package:flutter/material.dart';

import '../data/models.dart';
import '../state/store.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Rangée de vignettes au-dessus du champ de saisie ; vide quand rien n'est joint.
class ChatAttachChips extends StatelessWidget {
  final MailStore store;
  const ChatAttachChips({super.key, required this.store});

  @override
  Widget build(BuildContext context) {
    final mails = store.chatAttached;
    if (mails.isEmpty) return const SizedBox.shrink();
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
      child: Wrap(
        spacing: 8,
        runSpacing: 8,
        children: [for (final m in mails) _chip(context, m)],
      ),
    );
  }

  Widget _chip(BuildContext context, Mail m) {
    final p = context.pal;
    return Container(
      key: ValueKey('chat-attach-chip-${m.id}'),
      constraints: const BoxConstraints(maxWidth: 280),
      padding: const EdgeInsets.fromLTRB(10, 6, 4, 6),
      decoration: BoxDecoration(
        color: p.bg,
        border: Border.all(color: p.line),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Ico('inbox', size: 14, color: p.muted),
          const SizedBox(width: 8),
          Flexible(
            child: Text(
              '${m.from.name} · ${m.subject}',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: Fonts.sans(size: 13, color: p.text),
            ),
          ),
          const SizedBox(width: 4),
          IBtn(
            'x',
            key: ValueKey('chat-attach-remove-${m.id}'),
            tip: t('Retirer ce mail'),
            onTap: () => store.detachChatMail(m.id),
          ),
        ],
      ),
    );
  }
}
