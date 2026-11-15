//! Section « Mémoire de l'assistant » des réglages de l'IA : activation, lieu du calcul, installation du modèle, souvenirs.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_memory_state.dart';
import '../theme/app_theme.dart';
import 'ai_memory_dialogs.dart';
import 'ai_memory_embedder.dart';
import 'ai_memory_list.dart';
import 'rule_widgets.dart';
import 'widgets.dart';

/// Réglages de la mémoire ; chaque changement passe par [state] et rien n'est téléchargé sans clic.
class AiMemorySection extends StatefulWidget {
  final AiMemoryState state;
  final List<AiConnectionInfo> connections;
  const AiMemorySection({
    super.key,
    required this.state,
    required this.connections,
  });

  @override
  State<AiMemorySection> createState() => _AiMemorySectionState();
}

class _AiMemorySectionState extends State<AiMemorySection> {
  bool showList = false;

  AiMemoryState get s => widget.state;

  Future<void> _forgetAll() async {
    if (await showForgetAllMemoriesDialog(context)) await s.forgetAll();
  }

  Widget _install(Pal p) {
    final status = s.status;
    final local = status?.settings.local ?? true;
    if (!local || status == null || status.modelInstalled) {
      return const SizedBox.shrink();
    }
    if (s.installing) {
      return Padding(
        padding: const EdgeInsets.only(top: 12),
        child: Row(
          children: [
            Expanded(child: LinearProgressIndicator(value: s.progress)),
            const SizedBox(width: 10),
            Btn(
              key: const ValueKey('ai-memory-cancel'),
              label: t('Annuler'),
              onTap: s.cancel,
            ),
          ],
        ),
      );
    }
    return Padding(
      padding: const EdgeInsets.only(top: 12),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Btn(
          key: const ValueKey('ai-memory-install'),
          label: t('Installer le modèle de mémoire'),
          go: true,
          onTap: s.install,
        ),
      ),
    );
  }

  Widget _actions() {
    final stale = s.status?.stale ?? 0;
    return Wrap(
      spacing: 10,
      runSpacing: 8,
      children: [
        Btn(
          key: const ValueKey('ai-memory-show-list'),
          label: showList
              ? t('Masquer les souvenirs')
              : t('Voir les souvenirs'),
          onTap: () => setState(() => showList = !showList),
        ),
        if (stale > 0)
          Btn(
            key: const ValueKey('ai-memory-reindex'),
            label: t('Réindexer {0}', [tn(stale, 'souvenir', 'souvenirs')]),
            onTap: s.reindex,
          ),
        Btn(
          key: const ValueKey('ai-memory-forget-all'),
          label: t('Tout oublier'),
          onTap: _forgetAll,
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final count = s.status?.count ?? 0;
    return ListenableBuilder(
      listenable: s,
      builder: (_, _) => Padding(
        key: const ValueKey('ai-memory-section'),
        padding: const EdgeInsets.only(top: 16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              t('Mémoire de l’assistant'),
              style: Fonts.sans(size: 14, w: FontWeight.w500, color: p.text),
            ),
            const SizedBox(height: 2),
            Text(
              t(
                'L’assistant peut retenir ce que tu lui demandes ou ce que tu acceptes qu’il propose, puis s’en servir pour te répondre. Désactivée par défaut ; rien n’est retenu sans ton clic.',
              ),
              style: Fonts.sans(size: 13.5, color: p.dim),
            ),
            const SizedBox(height: 10),
            RuleSegment(
              key: const ValueKey('ai-memory-toggle'),
              value: s.enabled,
              onChanged: s.setEnabled,
              yes: t('Activée'),
              no: t('Désactivée'),
            ),
            const SizedBox(height: 10),
            Text(
              tn(count, 'souvenir', 'souvenirs'),
              key: const ValueKey('ai-memory-count'),
              style: Fonts.sans(size: 13.5, color: p.dim),
            ),
            const SizedBox(height: 10),
            AiMemoryEmbedder(state: s, connections: widget.connections),
            _install(p),
            const SizedBox(height: 12),
            _actions(),
            if (s.error != null)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Text(
                  s.error!,
                  key: const ValueKey('ai-memory-error'),
                  style: Fonts.sans(size: 13.5, color: p.accent),
                ),
              ),
            if (showList)
              Padding(
                padding: const EdgeInsets.only(top: 12),
                child: AiMemoryList(state: s),
              ),
          ],
        ),
      ),
    );
  }
}
