//! Carte d'une conversation enregistrée, pour la grille de la vue « Historique ».

import 'package:flutter/material.dart';

import '../data/format.dart';
import '../engine/ai_history_models.dart';
import '../i18n/i18n.dart';
import '../state/chat_message_json.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Date relative de la dernière mise à jour, et marqueur « Illisible » d'un contenu d'une version inconnue.
class ChatHistoryStamp extends StatelessWidget {
  final AiConversationMeta meta;
  const ChatHistoryStamp(this.meta, {super.key});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Wrap(
      spacing: 10,
      runSpacing: 2,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        Mono(rowTime(meta.updatedAt.millisecondsSinceEpoch), height: null),
        if (meta.payloadVersion != chatPayloadVersion)
          KeyedSubtree(
            key: ValueKey('chat-history-unreadable-${meta.id}'),
            child: Mono(t('Illisible'), color: p.accent, height: null),
          ),
      ],
    );
  }
}

/// Une conversation sous forme de carte : titre sur trois lignes, date, boutons renommer et supprimer.
class ChatHistoryCard extends StatelessWidget {
  final AiConversationMeta meta;
  final bool current;
  final bool busy;
  final VoidCallback onOpen;
  final VoidCallback onRename;
  final VoidCallback onDelete;
  const ChatHistoryCard(
    this.meta, {
    super.key,
    required this.current,
    required this.busy,
    required this.onOpen,
    required this.onRename,
    required this.onDelete,
  });

  Widget _footer() => Row(
    children: [
      Expanded(child: ChatHistoryStamp(meta)),
      IBtn(
        'pen',
        key: ValueKey('chat-history-rename-${meta.id}'),
        tip: t('Renommer'),
        size: 14,
        box: 28,
        onTap: onRename,
      ),
      IBtn(
        'trash',
        key: ValueKey('chat-history-delete-${meta.id}'),
        tip: t('Supprimer'),
        size: 14,
        box: 28,
        onTap: onDelete,
      ),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      key: ValueKey('chat-history-row-${meta.id}'),
      onTap: busy ? null : onOpen,
      tip: busy ? t('Attends la fin de la réponse') : null,
      builder: (_, hover) => Opacity(
        opacity: busy ? .45 : 1,
        child: Container(
          key: current ? ValueKey('chat-history-current-${meta.id}') : null,
          padding: const EdgeInsets.fromLTRB(14, 12, 8, 6),
          decoration: BoxDecoration(
            color: hover && !busy ? p.surface2 : p.surface,
            border: Border.all(
              color: current ? p.accent : p.line,
              width: current ? 1.5 : 1,
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: Padding(
                  padding: const EdgeInsets.only(right: 6),
                  child: Text(
                    meta.title,
                    maxLines: 3,
                    overflow: TextOverflow.ellipsis,
                    style: Fonts.sans(size: 15, color: p.text),
                  ),
                ),
              ),
              _footer(),
            ],
          ),
        ),
      ),
    );
  }
}
