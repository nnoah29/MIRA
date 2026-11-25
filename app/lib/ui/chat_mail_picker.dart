//! Liste défilante des derniers mails reçus pour en joindre un ou plusieurs au message de l'assistant.

import 'package:flutter/material.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Sélecteur à cases : expéditeur et objet par ligne, un clic joint ou détache le mail.
class ChatMailPicker extends StatelessWidget {
  final MailStore store;
  final VoidCallback onDone;
  const ChatMailPicker({super.key, required this.store, required this.onDone});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final mails = store.recentInboxMails();
    return Container(
      key: const ValueKey('chat-mail-picker'),
      margin: const EdgeInsets.fromLTRB(16, 0, 16, 8),
      decoration: BoxDecoration(
        color: p.bg,
        border: Border.all(color: p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _header(p),
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 260),
            child: mails.isEmpty
                ? _empty(p)
                : ListView(
                    shrinkWrap: true,
                    children: [for (final m in mails) _row(p, m)],
                  ),
          ),
        ],
      ),
    );
  }

  Widget _header(Pal p) => Padding(
    padding: const EdgeInsets.fromLTRB(14, 8, 8, 8),
    child: Row(
      children: [
        Mono(t('Joindre un mail'), color: p.accent, height: null),
        const Spacer(),
        Btn(
          key: const ValueKey('chat-mail-picker-done'),
          label: t('Terminé'),
          fontSize: 9.5,
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
          onTap: onDone,
        ),
      ],
    ),
  );

  Widget _empty(Pal p) => Padding(
    padding: const EdgeInsets.all(14),
    child: Text(
      t('Aucun mail reçu.'),
      style: Fonts.sans(size: 14, color: p.muted),
    ),
  );

  Widget _row(Pal p, Mail m) {
    final on = store.chatAttachedIds.contains(m.id);
    return Hov(
      onTap: () => store.toggleChatAttachment(m.id),
      builder: (_, h) => Container(
        key: ValueKey('chat-mail-row-${m.id}'),
        color: h ? p.surface : null,
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
        child: Row(
          children: [
            Icon(
              on ? Icons.check_box : Icons.check_box_outline_blank,
              size: 18,
              color: on ? p.accent : p.dim,
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Text.rich(
                TextSpan(
                  children: [
                    TextSpan(
                      text: m.from.name,
                      style: Fonts.sans(size: 14, color: p.text),
                    ),
                    TextSpan(
                      text: '  ${m.subject}',
                      style: Fonts.sans(size: 14, color: p.muted),
                    ),
                  ],
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
