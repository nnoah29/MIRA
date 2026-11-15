//! Sources d'une réponse de l'IA : les messages que ses outils ont consultés, cliquables ; cinq d'abord, le reste sur demande.

import 'package:flutter/material.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'mail_ref.dart';
import 'widgets.dart';

const _firstShown = 5;

/// Titre « Sources » puis une ligne par message connu ; au-delà de cinq, un bouton déplie les autres.
class ChatSources extends StatefulWidget {
  final List<Mail> mails;
  const ChatSources(this.mails, {super.key});
  @override
  State<ChatSources> createState() => _ChatSourcesState();
}

class _ChatSourcesState extends State<ChatSources> {
  bool _all = false;

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final hidden = widget.mails.length - _firstShown;
    final shown = _all ? widget.mails : widget.mails.take(_firstShown);
    return Padding(
      padding: const EdgeInsets.only(top: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.only(bottom: 6),
            child: Mono(t('Sources'), color: p.dim, height: null),
          ),
          for (final m in shown)
            KeyedSubtree(
              key: ValueKey('chat-source-${m.id}'),
              child: MailRef(m),
            ),
          if (hidden > 0 && !_all)
            Hov(
              key: const ValueKey('chat-sources-more'),
              onTap: () => setState(() => _all = true),
              builder: (_, h) => Mono(
                t('Voir les {0} autres', [hidden]),
                color: h ? p.accent : p.dim,
                height: null,
              ),
            ),
        ],
      ),
    );
  }
}
