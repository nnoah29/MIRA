//! Choix du modèle d'une connexion : un bouton qui montre le modèle actuel et ouvre le popover des modèles.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../state/ai_settings_state.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'ai_model_popup.dart';
import 'overlays.dart';
import 'widgets.dart';

/// Bouton du modèle d'une connexion ; le choix passe par [AiModelPopup] puis par l'état.
class AiModelPicker extends StatelessWidget {
  final AiSettingsState state;
  final AiConnectionInfo connection;
  const AiModelPicker({
    super.key,
    required this.state,
    required this.connection,
  });

  void _open(BuildContext anchor) {
    final id = connection.id;
    Pops.show(
      anchor,
      'ai-model-$id',
      (ctx, close) =>
          AiModelPopup(state: state, connectionId: id, close: close),
      maxWidth: 320,
    );
  }

  Widget _button(BuildContext anchor, Pal p) {
    final model = connection.defaultModel;
    final chosen = model != null && model.isNotEmpty;
    return Hov(
      key: ValueKey('ai-model-button-${connection.id}'),
      onTap: () => _open(anchor),
      builder: (_, h) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
        decoration: BoxDecoration(
          color: h ? p.surface2 : null,
          border: Border.all(color: p.line),
        ),
        child: Row(
          children: [
            Expanded(
              child: Text(
                chosen ? model : t('Choisir un modèle'),
                overflow: TextOverflow.ellipsis,
                style: Fonts.sans(size: 14, color: chosen ? p.text : p.dim),
              ),
            ),
            Ico('chev', size: 14, color: p.muted),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Mono(t('Modèle'), size: 10.5, ls: .18, height: null),
        const SizedBox(height: 6),
        Builder(builder: (anchor) => _button(anchor, p)),
      ],
    );
  }
}
