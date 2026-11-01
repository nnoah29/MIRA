//! Vue « Historique » du panneau assistant : conversations enregistrées, reprise, renommage, suppression.

import 'package:flutter/material.dart';

import '../engine/ai_history_models.dart';
import '../i18n/i18n.dart';
import '../state/chat_history_state.dart';
import '../theme/app_theme.dart';
import 'chat_history_card.dart';
import 'chat_history_dialogs.dart';
import 'widgets.dart';

/// Liste des conversations enregistrées, plus récentes d'abord, avec « Afficher plus » et « Tout effacer ».
class ChatHistoryView extends StatelessWidget {
  final ChatHistoryState state;
  final String? currentId;
  final bool busy;

  /// Vrai : grille de cartes ; faux : liste.
  final bool grid;

  /// Choisit l'affichage en grille (`true`) ou en liste (`false`).
  final ValueChanged<bool> onLayout;

  /// Reprend la conversation choisie (la vue ne sait pas où elle s'affiche ensuite).
  final Future<void> Function(String id) onOpen;

  /// La conversation ouverte vient d'être supprimée ou effacée : le fil doit repartir à neuf.
  final VoidCallback onCurrentRemoved;

  /// Ouvre les réglages (lien de l'état « historique désactivé »).
  final VoidCallback? onSettings;
  const ChatHistoryView({
    super.key,
    required this.state,
    required this.currentId,
    required this.busy,
    required this.grid,
    required this.onLayout,
    required this.onOpen,
    required this.onCurrentRemoved,
    this.onSettings,
  });

  Future<void> _rename(BuildContext context, AiConversationMeta m) async {
    final title = await showRenameDialog(context, m.title);
    if (title != null && title != m.title) await state.rename(m.id, title);
  }

  Future<void> _delete(BuildContext context, AiConversationMeta m) async {
    if (!await showDeleteDialog(context, m.title)) return;
    await state.delete(m.id);
    if (m.id == currentId) onCurrentRemoved();
  }

  Future<void> _clear(BuildContext context) async {
    if (!await showClearHistoryDialog(context)) return;
    await state.clearAll();
    onCurrentRemoved();
  }

  Widget _empty(BuildContext context) {
    final p = context.pal;
    return Center(
      key: const ValueKey('chat-history-empty'),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            state.enabled
                ? t('Aucune conversation enregistrée')
                : t('L’historique est désactivé'),
            style: Fonts.sans(size: 14, color: p.dim),
          ),
          if (!state.enabled && onSettings != null) ...[
            const SizedBox(height: 12),
            Btn(label: t('Réglages'), onTap: onSettings),
          ],
        ],
      ),
    );
  }

  Widget _footer(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 16),
    child: Row(
      children: [
        if (state.hasMore)
          Btn(
            key: const ValueKey('chat-history-more'),
            label: t('Afficher plus'),
            onTap: state.more,
          ),
        const Spacer(),
        Btn(
          key: const ValueKey('chat-history-clear'),
          label: t('Tout effacer'),
          onTap: () => _clear(context),
        ),
      ],
    ),
  );

  Widget _layoutToggle() => Padding(
    padding: const EdgeInsets.only(bottom: 8),
    child: Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        IBtn(
          'list',
          key: const ValueKey('chat-history-layout-list'),
          tip: t('Liste'),
          on: !grid,
          onTap: () => onLayout(false),
        ),
        const SizedBox(width: 4),
        IBtn(
          'grid',
          key: const ValueKey('chat-history-layout-grid'),
          tip: t('Grille'),
          on: grid,
          onTap: () => onLayout(true),
        ),
      ],
    ),
  );

  Widget _grid(BuildContext context) => GridView.builder(
    shrinkWrap: true,
    physics: const NeverScrollableScrollPhysics(),
    gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
      maxCrossAxisExtent: 240,
      mainAxisExtent: 132,
      crossAxisSpacing: 12,
      mainAxisSpacing: 12,
    ),
    itemCount: state.items.length,
    itemBuilder: (_, i) {
      final m = state.items[i];
      return ChatHistoryCard(
        m,
        current: m.id == currentId,
        busy: busy,
        onOpen: () => onOpen(m.id),
        onRename: () => _rename(context, m),
        onDelete: () => _delete(context, m),
      );
    },
  );

  List<Widget> _rows(BuildContext context) => [
    for (final m in state.items)
      _Row(
        m,
        current: m.id == currentId,
        busy: busy,
        onOpen: () => onOpen(m.id),
        onRename: () => _rename(context, m),
        onDelete: () => _delete(context, m),
      ),
  ];

  Widget _list(BuildContext context, double hp) => ListView(
    padding: EdgeInsets.fromLTRB(hp, 8, hp, 24),
    children: [
      _layoutToggle(),
      if (state.error != null)
        Padding(
          padding: const EdgeInsets.only(bottom: 8),
          child: Text(
            state.error!,
            key: const ValueKey('chat-history-error'),
            style: Fonts.sans(size: 13, color: context.pal.dim),
          ),
        ),
      if (grid) _grid(context) else ..._rows(context),
      _footer(context),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final hp = MediaQuery.sizeOf(context).width <= 1100 ? 16.0 : 32.0;
    return ListenableBuilder(
      listenable: state,
      builder: (context, _) {
        if (!state.loaded) return const SizedBox.shrink();
        if (state.items.isEmpty && state.error == null) return _empty(context);
        return _list(context, hp);
      },
    );
  }
}

class _Row extends StatelessWidget {
  final AiConversationMeta meta;
  final bool current;
  final bool busy;
  final VoidCallback onOpen;
  final VoidCallback onRename;
  final VoidCallback onDelete;
  const _Row(
    this.meta, {
    required this.current,
    required this.busy,
    required this.onOpen,
    required this.onRename,
    required this.onDelete,
  });

  Widget _titles(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        meta.title,
        maxLines: 2,
        overflow: TextOverflow.ellipsis,
        style: Fonts.sans(size: 15, color: p.text),
      ),
      const SizedBox(height: 4),
      ChatHistoryStamp(meta),
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
          constraints: const BoxConstraints(minHeight: 64),
          margin: const EdgeInsets.only(bottom: 4),
          padding: const EdgeInsets.symmetric(vertical: 12),
          decoration: BoxDecoration(
            color: current
                ? p.surface
                : (hover && !busy ? p.surface.withValues(alpha: .5) : null),
            border: Border(
              bottom: BorderSide(color: p.lineSoft),
              left: BorderSide(
                color: current ? p.accent : Colors.transparent,
                width: 2,
              ),
            ),
          ),
          child: Row(
            children: [
              const SizedBox(width: 12),
              Expanded(child: _titles(p)),
              IBtn(
                'pen',
                key: ValueKey('chat-history-rename-${meta.id}'),
                tip: t('Renommer'),
                onTap: onRename,
              ),
              IBtn(
                'trash',
                key: ValueKey('chat-history-delete-${meta.id}'),
                tip: t('Supprimer'),
                onTap: onDelete,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
