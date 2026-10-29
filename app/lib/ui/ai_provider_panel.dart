//! Contenu d'un onglet de fournisseur : ses cartes de connexion, ou l'invitation à en ajouter une.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_add_connection.dart';
import 'ai_connection_card.dart';

/// Ce que montre l'onglet de [provider] : cartes existantes, invitation, ajout (fournisseur libre : toujours).
class AiProviderPanel extends StatelessWidget {
  final AiSettingsState state;
  final AiProviderInfo provider;
  const AiProviderPanel({
    super.key,
    required this.state,
    required this.provider,
  });

  /// Phrase d'invitation d'un fournisseur sans connexion.
  Widget _invite(Pal p) => Text(
    provider.editableUrl
        ? t(
            'Un service compatible avec l’API d’OpenAI (serveur local, passerelle, autre fournisseur) : indique son adresse et ta propre clé d’API.',
          )
        : t(
            'Pour utiliser {0}, il te faut ta propre clé d’API, créée sur le site du service.',
            [provider.name],
          ),
    key: ValueKey('ai-invite-${provider.id}'),
    style: Fonts.sans(size: 13.5, color: p.dim),
  );

  @override
  Widget build(BuildContext context) {
    final mine = state.connectionsOf(provider.id);
    final canAdd = mine.isEmpty || provider.editableUrl;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (mine.isEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 16, bottom: 4),
            child: _invite(context.pal),
          ),
        for (final c in mine)
          AiConnectionCard(
            key: ValueKey('ai-card-key-${c.id}'),
            state: state,
            connection: c,
          ),
        if (canAdd) AiAddConnection(state: state, provider: provider),
      ],
    );
  }
}
