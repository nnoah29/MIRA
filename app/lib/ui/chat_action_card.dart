//! Carte « Action proposée » sous la réponse de l'assistant : ce qui sera fait, à quels messages, et Confirmer / Ignorer.

import 'package:flutter/material.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/chat_message.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

const _shown = 5;
const _known = {
  'archive',
  'trash',
  'mark_read',
  'mark_unread',
  'star',
  'unstar',
};

/// Action que l'IA propose sur des messages : rien ne change avant « Confirmer », et l'annulation habituelle s'applique ensuite.
class ChatActionCard extends StatelessWidget {
  final ProposedAction item;
  final List<Mail> targets;
  final VoidCallback onConfirm;
  final VoidCallback onDismiss;
  const ChatActionCard(
    this.item, {
    super.key,
    required this.targets,
    required this.onConfirm,
    required this.onDismiss,
  });

  String get _title {
    final n = tn(targets.length, 'message', 'messages');
    return switch (item.proposal.action) {
      'archive' => t('Archiver : {0}', [n]),
      'trash' => t('Mettre à la corbeille : {0}', [n]),
      'mark_read' => t('Marquer comme lu : {0}', [n]),
      'mark_unread' => t('Marquer comme non lu : {0}', [n]),
      'star' => t('Mettre en favori : {0}', [n]),
      'unstar' => t('Retirer des favoris : {0}', [n]),
      _ => t('Action inconnue'),
    };
  }

  Widget _subjects(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      for (final m in targets.take(_shown))
        Text(
          '• ${m.subject}',
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: Fonts.sans(size: 13.5, color: p.dim),
        ),
      if (targets.length > _shown)
        Text(
          t('+ {0} autres', [targets.length - _shown]),
          style: Fonts.sans(size: 13.5, color: p.dim),
        ),
    ],
  );

  Widget _end(Pal p) {
    final id = item.proposal.id;
    if (item.status == 'done') {
      return Mono(
        t('Fait'),
        key: ValueKey('chat-action-done-$id'),
        color: p.accent,
        height: null,
      );
    }
    if (item.status == 'dismissed') {
      return Mono(
        t('Ignorée'),
        key: ValueKey('chat-action-dismissed-$id'),
        color: p.dim,
        height: null,
      );
    }
    return Wrap(
      spacing: 10,
      runSpacing: 8,
      children: [
        if (targets.isNotEmpty && _known.contains(item.proposal.action))
          Btn(
            key: ValueKey('chat-action-confirm-$id'),
            icon: 'check',
            label: t('Confirmer'),
            go: true,
            onTap: onConfirm,
          ),
        Btn(
          key: ValueKey('chat-action-dismiss-$id'),
          label: t('Ignorer'),
          onTap: onDismiss,
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final reason = item.proposal.reason;
    return Container(
      key: ValueKey('chat-action-${item.proposal.id}'),
      width: double.infinity,
      margin: const EdgeInsets.only(top: 12),
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: p.surface2,
        border: Border(left: BorderSide(color: p.accent, width: 2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Mono(t('Action proposée'), color: p.accent, height: null),
          const SizedBox(height: 8),
          if (targets.isEmpty)
            Text(
              t('Ces messages ne sont plus disponibles'),
              style: Fonts.sans(size: 14, color: p.muted),
            )
          else ...[
            Text(
              _title,
              style: Fonts.sans(size: 14.5, w: FontWeight.w500, color: p.text),
            ),
            if (reason != null && reason.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Text(
                  t('Raison donnée par l’IA : {0}', [reason]),
                  style: Fonts.sans(size: 14, color: p.muted),
                ),
              ),
            const SizedBox(height: 6),
            _subjects(p),
          ],
          const SizedBox(height: 12),
          _end(p),
        ],
      ),
    );
  }
}
