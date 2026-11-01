//! Réglage de l'historique des conversations de l'assistant : interrupteur et effacement.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../state/chat_history_state.dart';
import '../theme/app_theme.dart';
import 'chat_history_dialogs.dart';
import 'rule_widgets.dart';
import 'widgets.dart';

/// Section « Historique des conversations » des réglages de l'IA ; chaque changement passe par [state].
class AiHistorySection extends StatelessWidget {
  final ChatHistoryState state;

  /// Appelé après un effacement total confirmé : le fil ouvert doit repartir à neuf.
  final VoidCallback? onCleared;
  const AiHistorySection({super.key, required this.state, this.onCleared});

  Future<void> _clear(BuildContext context) async {
    if (!await showClearHistoryDialog(context)) return;
    await state.clearAll();
    onCleared?.call();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.only(top: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            t('Conserver l’historique des conversations'),
            style: Fonts.sans(size: 14, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 2),
          Text(
            t(
              'Enregistre sur cet appareil le texte affiché des conversations. Les contenus envoyés au modèle en coulisses ne sont pas conservés.',
            ),
            key: const ValueKey('ai-history-note'),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          const SizedBox(height: 10),
          Wrap(
            spacing: 16,
            runSpacing: 10,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              RuleSegment(
                key: const ValueKey('ai-history-toggle'),
                value: state.enabled,
                onChanged: state.setEnabled,
                yes: t('Activé'),
                no: t('Désactivé'),
              ),
              Btn(
                key: const ValueKey('ai-history-clear'),
                label: t('Effacer tout l’historique'),
                onTap: () => _clear(context),
              ),
            ],
          ),
          if (state.error != null)
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: Text(
                state.error!,
                key: const ValueKey('ai-history-error'),
                style: Fonts.sans(size: 13.5, color: p.accent),
              ),
            ),
        ],
      ),
    );
  }
}
