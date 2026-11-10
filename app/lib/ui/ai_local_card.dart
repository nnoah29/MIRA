//! Carte d'un palier du modèle local : ce qu'il sait faire, son poids, et ses actions.

import 'package:flutter/material.dart';

import '../engine/ai_local_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_local_state.dart';
import '../theme/app_theme.dart';
import 'ai_local_confirm.dart';
import 'ai_local_text.dart';
import 'widgets.dart';

/// Carte d'un modèle : télécharger, suivre, annuler, utiliser, supprimer.
class AiLocalCard extends StatelessWidget {
  final AiLocalState state;
  final LocalModelStatus model;
  final bool inUse;
  final VoidCallback onUse;
  const AiLocalCard({
    super.key,
    required this.state,
    required this.model,
    required this.inUse,
    required this.onUse,
  });

  bool get _installing => state.installingId == model.id;

  Future<void> _install(BuildContext context) async {
    final ok = await confirmLocalInstall(
      context,
      model,
      serverInstalled: state.status?.serverInstalled ?? false,
    );
    if (ok) await state.install(model.id);
  }

  Widget _facts(Pal p) => Text(
    t('{0} · {1} · {2} · mémoire conseillée {3} Go', [
      model.name,
      localSize(model.size),
      model.license,
      (model.ramRecommendedMib / 1000).toStringAsFixed(1).replaceAll('.', ','),
    ]),
    style: Fonts.sans(size: 13, color: p.muted),
  );

  Widget _warning(Pal p) => Padding(
    padding: const EdgeInsets.only(top: 6),
    child: Text(
      t(
        'Mémoire disponible juste pour ce modèle : il peut être lent ou s’arrêter.',
      ),
      key: ValueKey('ai-local-memory-${model.id}'),
      style: Fonts.sans(size: 13, color: p.accent),
    ),
  );

  Widget _progress(Pal p) => Padding(
    padding: const EdgeInsets.only(top: 10),
    child: Row(
      children: [
        Expanded(
          child: LinearProgressIndicator(
            value: state.progress,
            color: p.accent,
            backgroundColor: p.line,
          ),
        ),
        const SizedBox(width: 12),
        Btn(
          key: const ValueKey('ai-local-cancel'),
          label: t('Annuler'),
          onTap: state.cancel,
        ),
      ],
    ),
  );

  Widget _actions(BuildContext context) {
    final busy = state.installingId != null;
    return Padding(
      padding: const EdgeInsets.only(top: 10),
      child: Wrap(
        spacing: 8,
        runSpacing: 8,
        children: [
          if (!model.installed)
            Btn(
              key: ValueKey('ai-local-install-${model.id}'),
              label: t('Télécharger'),
              go: true,
              onTap: busy ? null : () => _install(context),
            ),
          if (model.installed && !inUse)
            Btn(
              key: ValueKey('ai-local-use-${model.id}'),
              label: t('Utiliser comme assistant'),
              go: true,
              onTap: onUse,
            ),
          if (model.installed)
            Btn(
              key: ValueKey('ai-local-remove-${model.id}'),
              label: t('Supprimer'),
              onTap: () => state.remove(model.id),
            ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      key: ValueKey('ai-local-card-${model.id}'),
      margin: const EdgeInsets.only(top: 14),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(border: Border.all(color: p.line)),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            localTierName(model.tier),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 2),
          Text(
            localTierAbility(model),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          const SizedBox(height: 4),
          _facts(p),
          if (model.memoryWarning && !model.installed) _warning(p),
          if (inUse)
            Padding(
              padding: const EdgeInsets.only(top: 6),
              child: Text(
                t('Utilisé par l’assistant'),
                style: Fonts.sans(size: 13, color: p.accent),
              ),
            ),
          if (_installing) _progress(p) else _actions(context),
        ],
      ),
    );
  }
}
