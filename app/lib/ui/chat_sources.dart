//! Sources d'une réponse de l'IA : les messages que ses outils ont consultés, cliquables ; un identifiant inconnu est ignoré.

import 'package:flutter/material.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'mail_ref.dart';
import 'widgets.dart';

/// Titre « Sources » puis une ligne par message connu du magasin.
class ChatSources extends StatelessWidget {
  final List<Mail> mails;
  const ChatSources(this.mails, {super.key});
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.only(top: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.only(bottom: 6),
            child: Mono(t('Sources'), color: p.dim, height: null),
          ),
          for (final m in mails)
            KeyedSubtree(
              key: ValueKey('chat-source-${m.id}'),
              child: MailRef(m),
            ),
        ],
      ),
    );
  }
}
