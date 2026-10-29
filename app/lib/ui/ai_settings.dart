//! Réglages « IA » : connexions, clés, modèles, essai, connexion par défaut.

import 'package:flutter/material.dart';

import '../engine/ai_admin.dart';
import '../i18n/i18n.dart';
import '../state/ai_journal_state.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_journal_view.dart';
import 'ai_provider_panel.dart';
import 'ai_provider_tabs.dart';
import 'rule_widgets.dart';

/// Section des réglages qui gère les connexions aux services d'IA.
class AiSettings extends StatefulWidget {
  final AiAdmin admin;
  const AiSettings({super.key, required this.admin});

  @override
  State<AiSettings> createState() => _AiSettingsState();
}

class _AiSettingsState extends State<AiSettings> {
  late final AiSettingsState state = AiSettingsState(widget.admin);
  String? selected;
  bool showJournal = false;
  late final AiJournalState journal = AiJournalState(widget.admin);

  @override
  void initState() {
    super.initState();
    state.addListener(_refresh);
    journal.addListener(_refresh);
    state.load();
  }

  void _refresh() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    state.removeListener(_refresh);
    journal.removeListener(_refresh);
    state.dispose();
    journal.dispose();
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
    ),
  );

  /// Onglets puis contenu du fournisseur choisi ; au départ, celui de la connexion par défaut.
  List<Widget> _providers() {
    final shown = selected ??= state.initialProviderId();
    if (shown == null) return const [];
    final current = state.providers.firstWhere(
      (p) => p.id == shown,
      orElse: () => state.providers.first,
    );
    return [
      AiProviderTabs(
        providers: state.providers,
        connections: state.connections,
        selected: current.id,
        onSelect: (id) => setState(() => selected = id),
      ),
      AiProviderPanel(
        key: ValueKey('ai-panel-${current.id}'),
        state: state,
        provider: current,
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
          _switch(),
          if (showJournal) _journal() else ..._providers(),
        ],
      ),
    );
  }
}
