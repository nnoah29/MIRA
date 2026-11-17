//! Liste des souvenirs : recherche par le sens, modification et oubli de chacun.

import 'package:flutter/material.dart';

import '../engine/ai_memory_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_memory_state.dart';
import '../theme/app_theme.dart';
import 'ai_memory_dialogs.dart';
import 'widgets.dart';

/// Souvenirs enregistrés (ou résultats de la recherche) avec leurs actions.
class AiMemoryList extends StatefulWidget {
  final AiMemoryState state;
  const AiMemoryList({super.key, required this.state});

  @override
  State<AiMemoryList> createState() => _AiMemoryListState();
}

class _AiMemoryListState extends State<AiMemoryList> {
  final TextEditingController query = TextEditingController();

  @override
  void dispose() {
    query.dispose();
    super.dispose();
  }

  String _kind(MemoryItem m) => switch (m.kind) {
    'preference' => t('Préférence'),
    'action' => t('Action menée'),
    _ => t('Fait'),
  };

  Future<void> _edit(MemoryItem m) async {
    final text = await showEditMemoryDialog(context, m.text);
    if (text != null && text != m.text) await widget.state.edit(m.id, text);
  }

  Future<void> _forget(MemoryItem m) async {
    if (await showForgetMemoryDialog(context, m.text)) {
      await widget.state.delete(m.id);
    }
  }

  Widget _row(Pal p, MemoryItem m) => Padding(
    key: ValueKey('ai-memory-row-${m.id}'),
    padding: const EdgeInsets.symmetric(vertical: 8),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(m.text, style: Fonts.sans(size: 14, color: p.text)),
              const SizedBox(height: 2),
              Text(
                m.byAi ? '${_kind(m)} · ${t('proposé par l’IA')}' : _kind(m),
                style: Fonts.sans(size: 12.5, color: p.dim),
              ),
            ],
          ),
        ),
        const SizedBox(width: 10),
        Btn(
          key: ValueKey('ai-memory-edit-${m.id}'),
          label: t('Modifier'),
          onTap: () => _edit(m),
        ),
        const SizedBox(width: 8),
        Btn(
          key: ValueKey('ai-memory-delete-${m.id}'),
          label: t('Oublier'),
          onTap: () => _forget(m),
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = widget.state;
    final shown = s.hits == null ? s.items : [for (final h in s.hits!) h.item];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        TextField(
          key: const ValueKey('ai-memory-search-field'),
          controller: query,
          textInputAction: TextInputAction.search,
          style: Fonts.sans(size: 14, color: p.text),
          cursorColor: p.accent,
          decoration: InputDecoration(
            hintText: t('Chercher dans les souvenirs'),
          ),
          onSubmitted: s.search,
        ),
        if (shown.isEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 12),
            child: Text(
              s.hits == null
                  ? t('Aucun souvenir pour l’instant.')
                  : t('Aucun souvenir ne correspond.'),
              key: const ValueKey('ai-memory-empty'),
              style: Fonts.sans(size: 13.5, color: p.dim),
            ),
          ),
        for (final m in shown) _row(p, m),
        if (s.hits == null && s.hasMore)
          Align(
            alignment: Alignment.centerLeft,
            child: Btn(
              key: const ValueKey('ai-memory-more'),
              label: t('Afficher la suite'),
              onTap: s.more,
            ),
          ),
      ],
    );
  }
}
