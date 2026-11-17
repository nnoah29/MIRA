//! Choix du lieu de calcul des embeddings : sur l'appareil, ou chez le service d'une connexion avec avertissement de confidentialité.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_memory_state.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Pose le choix « local » ou « fournisseur » ; le mode fournisseur n'est enregistré qu'avec une connexion et un modèle.
class AiMemoryEmbedder extends StatefulWidget {
  final AiMemoryState state;
  final List<AiConnectionInfo> connections;
  const AiMemoryEmbedder({
    super.key,
    required this.state,
    required this.connections,
  });

  @override
  State<AiMemoryEmbedder> createState() => _AiMemoryEmbedderState();
}

class _AiMemoryEmbedderState extends State<AiMemoryEmbedder> {
  bool? remoteChoice;
  String? connection;
  late final TextEditingController model = TextEditingController(
    text: widget.state.status?.settings.model ?? '',
  )..addListener(() => setState(() {}));

  bool get _remote =>
      remoteChoice ?? !(widget.state.status?.settings.local ?? true);

  String? get _chosen => connection ?? widget.state.status?.settings.connection;

  bool get _canSave => _chosen != null && model.text.trim().isNotEmpty;

  @override
  void dispose() {
    model.dispose();
    super.dispose();
  }

  Future<void> _local() async {
    setState(() => remoteChoice = false);
    await widget.state.setLocal();
  }

  Widget _modes() => Wrap(
    spacing: 10,
    runSpacing: 8,
    children: [
      Btn(
        key: const ValueKey('ai-memory-local-mode'),
        label: t('Sur cet appareil'),
        go: !_remote,
        onTap: _local,
      ),
      Btn(
        key: const ValueKey('ai-memory-provider-mode'),
        label: t('Via mon fournisseur'),
        go: _remote,
        onTap: () => setState(() => remoteChoice = true),
      ),
    ],
  );

  Widget _provider(Pal p) => Padding(
    padding: const EdgeInsets.only(top: 12),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          t(
            'Le texte de chaque souvenir est envoyé à ce service pour être indexé : autorise d’abord le type « souvenirs » sur la connexion.',
          ),
          key: const ValueKey('ai-memory-privacy-note'),
          style: Fonts.sans(size: 13.5, color: p.accent),
        ),
        const SizedBox(height: 10),
        Wrap(
          spacing: 10,
          runSpacing: 8,
          children: [
            for (final c in widget.connections)
              Btn(
                key: ValueKey('ai-memory-connection-${c.id}'),
                label: c.label,
                go: _chosen == c.id,
                onTap: () => setState(() => connection = c.id),
              ),
          ],
        ),
        const SizedBox(height: 10),
        Row(
          children: [
            Expanded(
              child: TextField(
                key: const ValueKey('ai-memory-model-field'),
                controller: model,
                style: Fonts.sans(size: 14, color: p.text),
                cursorColor: p.accent,
                decoration: InputDecoration(
                  hintText: t(
                    'Modèle d’embeddings (ex. text-embedding-3-small)',
                  ),
                ),
              ),
            ),
            const SizedBox(width: 10),
            Btn(
              key: const ValueKey('ai-memory-model-save'),
              label: t('Enregistrer'),
              go: true,
              onTap: _canSave
                  ? () => widget.state.setProvider(_chosen!, model.text.trim())
                  : null,
            ),
          ],
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [_modes(), if (_remote) _provider(p)],
    );
  }
}
