//! Liste défilante des derniers mails reçus pour en joindre plusieurs au message de l'assistant, dans un popover.

import 'package:flutter/material.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Sélecteur à cases : expéditeur et objet par ligne, un clic joint ou détache le mail.
class ChatMailPicker extends StatelessWidget {
  final MailStore store;
  const ChatMailPicker({super.key, required this.store});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final mails = store.recentInboxMails();
    return SizedBox(
      width: 380,
      child: Column(
        key: const ValueKey('chat-mail-picker'),
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _header(p),
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 320),
            child: mails.isEmpty
                ? _empty(p)
                : SingleChildScrollView(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [for (final m in mails) _row(p, m)],
                    ),
                  ),
          ),
        ],
      ),
    );
  }

  Widget _header(Pal p) => Padding(
    padding: const EdgeInsets.fromLTRB(14, 8, 14, 8),
    child: Mono(t('Joindre un mail'), color: p.accent, height: null),
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
