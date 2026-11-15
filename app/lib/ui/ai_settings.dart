//! Réglages « IA » : connexions, clés, modèles, essai, connexion par défaut.

import 'package:flutter/material.dart';

import '../engine/ai_admin.dart';
import '../engine/ai_history_admin.dart';
import '../i18n/i18n.dart';
import '../state/ai_journal_state.dart';
import '../state/ai_local_state.dart';
import '../state/ai_settings_state.dart';
import '../state/chat_history_state.dart';
import '../theme/app_theme.dart';
import 'ai_history_section.dart';
import 'ai_journal_view.dart';
import 'ai_local_panel.dart';
import 'ai_provider_family.dart';
import 'ai_provider_panel.dart';
import 'ai_provider_tabs.dart';
import 'rule_widgets.dart';

/// Section des réglages qui gère les connexions aux services d'IA.
class AiSettings extends StatefulWidget {
  final AiAdmin admin;

  /// Historique partagé avec l'assistant ; sans lui, la section tient le sien.
  final ChatHistoryState? history;

  /// Appelé après l'effacement total de l'historique.
  final VoidCallback? onHistoryCleared;
  const AiSettings({
    super.key,
    required this.admin,
    this.history,
    this.onHistoryCleared,
  });

  @override
  State<AiSettings> createState() => _AiSettingsState();
}

class _AiSettingsState extends State<AiSettings> {
  late final AiSettingsState state = AiSettingsState(widget.admin);
  String? selected;
  bool showJournal = false;
  late final AiJournalState journal = AiJournalState(widget.admin);
  late final AiLocalState local = AiLocalState(widget.admin);
  late final ChatHistoryState history =
      widget.history ??
      ChatHistoryState(() => AiHistoryAdmin(widget.admin.client));

  @override
  void initState() {
    super.initState();
    state.addListener(_refresh);
    journal.addListener(_refresh);
    history.addListener(_refresh);
    local.addListener(_refresh);
    state.load();
    local.refresh();
    history.load();
  }

  void _refresh() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    state.removeListener(_refresh);
    journal.removeListener(_refresh);
    history.removeListener(_refresh);
    local.removeListener(_refresh);
    state.dispose();
    local.dispose();
    journal.dispose();
    if (widget.history == null) history.dispose();
    super.dispose();
  }

  void _showJournal(bool shown) {
    if (shown == showJournal) return;
    setState(() => showJournal = shown);
    if (shown) journal.load();
  }

  Widget _switch() => Padding(
    padding: const EdgeInsets.only(top: 16),
    child: Align(
      alignment: Alignment.centerLeft,
      child: RuleSegment(
        key: const ValueKey('ai-view-toggle'),
        value: !showJournal,
        onChanged: (connections) => _showJournal(!connections),
        yes: t('Connexions'),
        no: t('Journal'),
      ),
    ),
  );

  Widget _journal() => Padding(
    padding: const EdgeInsets.only(top: 16),
    child: AiJournalView(
      state: journal,
      providerNames: {for (final v in state.providers) v.id: v.name},
      connectionLabels: {for (final c in state.connections) c.id: c.label},
    ),
  );

  /// Onglets puis contenu du fournisseur choisi ; au départ, celui de la connexion par défaut.
  List<Widget> _providers() {
    final first = state.initialProviderId();
    final shown = selected ??= first == null ? null : aiTabOf(first);
    if (shown == null) return const [];
    final tabs = [
      for (final p in aiTabProviders(state.providers))
        if (!p.localRuntime || local.supported) p,
    ];
    final current = tabs.firstWhere(
      (p) => p.id == shown,
      orElse: () => tabs.first,
    );
    return [
      AiProviderTabs(
        providers: tabs,
        connections: state.connections,
        selected: current.id,
        onSelect: (id) => setState(() => selected = id),
      ),
      if (current.localRuntime)
        AiLocalPanel(
          key: ValueKey('ai-panel-${current.id}'),
          state: local,
          settings: state,
          providerId: current.id,
        )
      else
        AiProviderPanel(
          key: ValueKey('ai-panel-${current.id}'),
          state: state,
          providers: aiFamilyOf(state.providers, current),
        ),
    ];
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('IA'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 2),
          Text(
            t(
              'Choisis le service qui répond dans l’assistant. Ta clé d’API reste sur cet appareil et n’est jamais réaffichée ; le texte envoyé ne part qu’avec ton accord.',
            ),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          if (state.error != null)
            Padding(
              padding: const EdgeInsets.only(top: 10),
              child: Text(
                state.error!,
                key: const ValueKey('ai-error'),
                style: Fonts.sans(size: 13.5, color: p.accent),
              ),
            ),
          AiHistorySection(state: history, onCleared: widget.onHistoryCleared),
          _switch(),
          if (showJournal) _journal() else ..._providers(),
        ],
      ),
    );
  }
}
