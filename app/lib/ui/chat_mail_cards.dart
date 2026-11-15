//! Cartes de messages que l'IA montre dans le chat : un titre facultatif, puis une carte cliquable par message avec sa raison.

import 'package:flutter/material.dart';

import '../data/models.dart';
import '../engine/ai_models.dart';
import '../theme/app_theme.dart';
import 'mail_ref.dart';

/// Liste de cartes d'un appel de l'outil ; [find] rend le message connu d'un identifiant, ou rien (la carte est alors omise).
class ChatMailCards extends StatelessWidget {
  final AiMailCards list;
  final Mail? Function(String id) find;
  const ChatMailCards(this.list, {super.key, required this.find});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final title = list.title;
    return Padding(
      padding: const EdgeInsets.only(top: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (title != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 6),
              child: Text(
                title,
                style: Fonts.sans(size: 14, w: FontWeight.w600, color: p.text),
              ),
            ),
          for (final card in list.cards)
            if (find(card.id) case final mail?)
              KeyedSubtree(
                key: ValueKey('chat-card-${card.id}'),
                child: MailRef(mail, reason: card.reason),
              ),
        ],
      ),
    );
  }
}
