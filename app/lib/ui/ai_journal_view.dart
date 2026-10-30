//! Page « Journal » de l'IA : ce qui a été envoyé à quel service, en comptes et en codes, jamais en texte.

import 'package:flutter/material.dart';

import '../data/format.dart';
import '../engine/ai_models.dart';
import '../engine/engine_client.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';
import '../state/ai_journal_state.dart';
import '../theme/app_theme.dart';
import 'ai_kind_names.dart';
import 'widgets.dart';

/// Issue d'une génération dans la langue active : les issues propres au journal, sinon le texte du code d'erreur.
String aiOutcomeText(String outcome) => switch (outcome) {
  'end_turn' => t('Terminé'),
  'max_tokens' => t('Coupé : limite de longueur'),
  'cancelled' => t('Annulé'),
  'running' => t('En cours'),
  'interrupted' => t('Interrompu'),
  _ => engineErrorText(EngineException(outcome, '')),
};

const _calm = {'end_turn', 'max_tokens', 'cancelled', 'running', 'interrupted'};

/// Liste des envois, plus récents d'abord, avec « Afficher plus » et « Vider le journal ».
class AiJournalView extends StatelessWidget {
  final AiJournalState state;
  final Map<String, String> providerNames;
  const AiJournalView({
    super.key,
    required this.state,
    required this.providerNames,
  });

  Future<void> _confirmClear(BuildContext context) async {
    final ok = await showDialog<bool>(
      context: context,
      builder: (d) => _ClearDialog(onChoose: (v) => Navigator.pop(d, v)),
    );
    if (ok == true) await state.clear();
  }

  String _details(AiJournalRow r) {
    final kinds = r.kinds.map(aiKindName).join(', ');
    final tokens = r.inputTokens == null && r.outputTokens == null
        ? null
        : t('{0} → {1} jetons', [r.inputTokens ?? 0, r.outputTokens ?? 0]);
    final tools = r.tools.map((u) => '${u.name} ×${u.count}').join(', ');
    return [
      if (kinds.isNotEmpty) kinds,
      if (tools.isNotEmpty) t('Outils : {0}', [tools]),
      tn(r.chars, 'caractère', 'caractères'),
      if (r.masked > 0) tn(r.masked, 'masqué', 'masqués'),
      ?tokens,
    ].join(' · ');
  }

  Widget _row(Pal p, int i, AiJournalRow r) {
    final name = providerNames[r.provider] ?? r.provider;
    final failed = !_calm.contains(r.outcome);
    return Container(
      key: ValueKey('ai-journal-row-$i'),
      padding: const EdgeInsets.symmetric(vertical: 12),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: p.line)),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '$name · ${r.model}${r.local ? ' · ${t('Local')}' : ''}',
                  style: Fonts.sans(size: 15, color: p.text),
                ),
                const SizedBox(height: 4),
                Text(
                  '${fullDate(r.at)} · ${_details(r)}',
                  style: Fonts.sans(size: 13, color: p.dim),
                ),
              ],
            ),
          ),
          const SizedBox(width: 12),
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 220),
            child: Text(
              aiOutcomeText(r.outcome),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              textAlign: TextAlign.end,
              style: Fonts.sans(size: 13.5, color: failed ? p.accent : p.muted),
            ),
          ),
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
            key: const ValueKey('ai-journal-more'),
            label: t('Afficher plus'),
            onTap: state.loading ? null : state.more,
          ),
        const Spacer(),
        Btn(
          key: const ValueKey('ai-journal-clear'),
          label: t('Vider le journal'),
          onTap: state.rows.isEmpty ? null : () => _confirmClear(context),
        ),
      ],
    ),
  );

  Widget _failure(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        state.error!,
        key: const ValueKey('ai-journal-error'),
        style: Fonts.sans(size: 13.5, color: p.accent),
      ),
      if (state.rows.isEmpty) ...[
        const SizedBox(height: 16),
        Btn(
          key: const ValueKey('ai-journal-retry'),
          label: t('Réessayer'),
          onTap: state.loading ? null : state.load,
        ),
      ],
    ],
  );

  Widget _empty(Pal p) => Text(
    t('Aucun envoi pour l’instant.'),
    key: const ValueKey('ai-journal-empty'),
    style: Fonts.sans(size: 13.5, color: p.dim),
  );

  Widget _loading(Pal p) => Text(
    t('Chargement…'),
    key: const ValueKey('ai-journal-loading'),
    style: Fonts.sans(size: 13.5, color: p.dim),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final empty = state.loaded && state.rows.isEmpty && state.error == null;
    return Column(
      key: const ValueKey('ai-journal-view'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (!state.loaded) _loading(p),
        if (state.error != null) _failure(p),
        if (empty) _empty(p),
        for (var i = 0; i < state.rows.length; i++) _row(p, i, state.rows[i]),
        if (state.rows.isNotEmpty) _footer(context),
      ],
    );
  }
}

class _ClearDialog extends StatelessWidget {
  final ValueChanged<bool> onChoose;
  const _ClearDialog({required this.onChoose});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Dialog(
      backgroundColor: p.bgSide,
      shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 420),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                t('Vider le journal ?'),
                style: Fonts.sans(size: 16, w: FontWeight.w600, color: p.text),
              ),
              const SizedBox(height: 8),
              Text(
                t(
                  'Toutes les lignes du journal des envois sont effacées de cet appareil.',
                ),
                style: Fonts.sans(size: 13.5, color: p.dim),
              ),
              const SizedBox(height: 20),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  Btn(label: t('Annuler'), onTap: () => onChoose(false)),
                  const SizedBox(width: 10),
                  Btn(
                    key: const ValueKey('confirm-clear-journal'),
                    label: t('Vider'),
                    go: true,
                    onTap: () => onChoose(true),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
