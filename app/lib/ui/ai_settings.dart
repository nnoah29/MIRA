//! Réglages « IA » : connexions, clés, modèles, essai, connexion par défaut.

import 'package:flutter/material.dart';

import '../engine/ai_admin.dart';
import '../i18n/i18n.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_add_connection.dart';
import 'ai_connection_card.dart';

/// Section des réglages qui gère les connexions aux services d'IA.
class AiSettings extends StatefulWidget {
  final AiAdmin admin;
  const AiSettings({super.key, required this.admin});

  @override
  State<AiSettings> createState() => _AiSettingsState();
}

class _AiSettingsState extends State<AiSettings> {
  late final AiSettingsState state = AiSettingsState(widget.admin);

  @override
  void initState() {
    super.initState();
    state.addListener(_refresh);
    state.load();
  }

  void _refresh() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    state.removeListener(_refresh);
    state.dispose();
    super.dispose();
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
          for (final c in state.connections)
            AiConnectionCard(
              key: ValueKey('ai-card-key-${c.id}'),
              state: state,
              connection: c,
            ),
          const SizedBox(height: 22),
          AiAddConnection(state: state),
        ],
      ),
    );
  }
}
