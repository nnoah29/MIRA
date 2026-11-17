//! Carte « Retenir ceci ? » ou « Oublier ceci ? » sous la réponse de l'assistant : le texte concerné et Confirmer / Ignorer.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../state/chat_message.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Proposition de l'IA sur la mémoire : rien n'est retenu ni oublié avant le clic de l'utilisateur.
class ChatMemoryCard extends StatelessWidget {
  final ProposedMemory item;
  final VoidCallback onConfirm;
  final VoidCallback onDismiss;
  const ChatMemoryCard(
    this.item, {
    super.key,
    required this.onConfirm,
    required this.onDismiss,
  });

  bool get _forget => item.proposal.action == 'forget';

  String get _kind => switch (item.proposal.kind) {
    'preference' => t('Préférence'),
    'action' => t('Action menée'),
    _ => t('Fait'),
  };

  Widget _end(Pal p) {
    final id = item.proposal.id;
    if (item.status == 'done') {
      return Mono(
        _forget ? t('Oublié') : t('Retenu'),
        key: ValueKey('chat-memory-done-$id'),
        color: p.accent,
        height: null,
      );
    }
    if (item.status == 'dismissed') {
      return Mono(
        t('Ignorée'),
        key: ValueKey('chat-memory-dismissed-$id'),
        color: p.dim,
        height: null,
      );
    }
    return Wrap(
      spacing: 10,
      runSpacing: 8,
      children: [
        if (!item.missing)
          Btn(
            key: ValueKey('chat-memory-confirm-$id'),
            icon: 'check',
            label: _forget ? t('Oublier') : t('Retenir'),
            go: true,
            onTap: onConfirm,
          ),
        Btn(
          key: ValueKey('chat-memory-dismiss-$id'),
          label: t('Ignorer'),
          onTap: onDismiss,
        ),
      ],
    );
  }

  Widget _body(Pal p) {
    if (item.missing) {
      return Text(
        t('Ce souvenir n’existe plus'),
        style: Fonts.sans(size: 14, color: p.muted),
      );
    }
    final shown = _forget ? item.target : item.proposal.text;
    final reason = _forget ? item.proposal.text : '';
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          shown ?? '…',
          style: Fonts.sans(size: 14.5, w: FontWeight.w500, color: p.text),
        ),
        if (reason.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Text(
              t('Raison donnée par l’IA : {0}', [reason]),
              style: Fonts.sans(size: 14, color: p.muted),
            ),
          ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final head = _forget ? t('Oublier ceci ?') : t('Retenir ceci ?');
    return Container(
      key: ValueKey('chat-memory-${item.proposal.id}'),
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
          Mono(
            _forget ? head : '$head · $_kind',
            color: p.accent,
            height: null,
          ),
          const SizedBox(height: 8),
          _body(p),
          if (item.error != null)
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: Text(
                item.error!,
                style: Fonts.sans(size: 13.5, color: p.accent),
              ),
            ),
          const SizedBox(height: 10),
          _end(p),
        ],
      ),
    );
  }
}
