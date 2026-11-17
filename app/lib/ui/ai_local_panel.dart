//! Onglet « Local » : un modèle d'IA qui tourne sur la machine, facultatif, en trois paliers.

import 'package:flutter/material.dart';

import '../engine/ai_local_models.dart';
import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_local_state.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import 'ai_local_card.dart';
import 'widgets.dart';

/// Les trois paliers du modèle local ; seule l'action de l'utilisateur télécharge ou lance quoi que ce soit.
class AiLocalPanel extends StatelessWidget {
  final AiLocalState state;
  final AiSettingsState settings;
  final String providerId;
  const AiLocalPanel({
    super.key,
    required this.state,
    required this.settings,
    required this.providerId,
  });

  /// Connexion du modèle local, créée par le moteur au premier modèle installé.
  AiConnectionInfo? get _connection =>
      settings.connectionsOf(providerId).firstOrNull;

  /// Fait de ce modèle celui de l'assistant : modèle par défaut de la connexion, puis connexion par défaut.
  Future<void> _use(LocalModelStatus model) async {
    final connection = _connection;
    if (connection == null) return;
    await settings.setModel(connection, model.id);
    await settings.setDefault(connection.id);
  }

  bool _inUse(LocalModelStatus model) {
    final c = _connection;
    return c != null && c.isDefault && c.defaultModel == model.id;
  }

  Widget _intro(Pal p) => Padding(
    padding: const EdgeInsets.only(top: 16),
    child: Text(
      t(
        'Un modèle d’IA qui tourne sur ta machine : ni compte, ni clé, rien ne sort de l’ordinateur. C’est facultatif : tant que tu n’en télécharges pas un, il n’occupe ni disque ni mémoire.',
      ),
      style: Fonts.sans(size: 13.5, color: p.dim),
    ),
  );

  Widget _stop(Pal p, LocalRuntimeStatus status) => Padding(
    padding: const EdgeInsets.only(top: 14),
    child: Align(
      alignment: Alignment.centerLeft,
      child: Btn(
        key: const ValueKey('ai-local-stop'),
        label: t('Libérer la mémoire'),
        onTap: state.stop,
      ),
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final status = state.status ?? LocalRuntimeStatus.unsupported;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _intro(p),
        if (state.error != null)
          Padding(
            padding: const EdgeInsets.only(top: 10),
            child: Text(
              state.error!,
              key: const ValueKey('ai-local-error'),
              style: Fonts.sans(size: 13.5, color: p.accent),
            ),
          ),
        if (status.runningModel != null) _stop(p, status),
        for (final model in status.models)
          AiLocalCard(
            state: state,
            model: model,
            inUse: _inUse(model),
            onUse: () => _use(model),
          ),
      ],
    );
  }
}
