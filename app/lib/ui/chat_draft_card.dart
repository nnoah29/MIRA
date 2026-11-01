//! Carte « Brouillon proposé » sous la réponse de l'assistant : destinataires, objet, extrait, et bouton pour l'ouvrir.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Brouillon que l'IA a préparé : rien n'est ouvert ni envoyé avant le clic sur « Ouvrir le brouillon ».
class ChatDraftCard extends StatelessWidget {
  final AiDraftProposal draft;
  final VoidCallback onOpen;
  const ChatDraftCard(this.draft, {super.key, required this.onOpen});

  String get _recipients {
    final to = draft.to ?? const <AiAddressEntry>[];
    if (to.isEmpty) return t('(aucun destinataire)');
    return to.map((p) => p.name ?? p.address).join(', ');
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final subject = (draft.subject ?? '').isEmpty
        ? t('(sans objet)')
        : draft.subject!;
    return Container(
      key: ValueKey('chat-draft-${draft.id}'),
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
          Mono(t('Brouillon proposé'), color: p.accent, height: null),
          const SizedBox(height: 8),
          Text(
            '${t('À')} : $_recipients',
            style: Fonts.sans(size: 14, color: p.muted),
          ),
          Text(
            '${t('Objet')} : $subject',
            style: Fonts.sans(size: 14, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 6),
          Text(
            draft.body ?? '',
            maxLines: 3,
            overflow: TextOverflow.ellipsis,
            style: Fonts.sans(size: 14, color: p.dim, height: 1.45),
          ),
          const SizedBox(height: 12),
          Btn(
            key: ValueKey('chat-draft-open-${draft.id}'),
            icon: 'pen',
            label: t('Ouvrir le brouillon'),
            go: true,
            onTap: onOpen,
          ),
        ],
      ),
    );
  }
}
