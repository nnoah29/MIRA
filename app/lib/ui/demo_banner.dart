//! Bandeau de l'assistant de démonstration : dit que les réponses sont simulées et où configurer une IA.

import 'package:flutter/material.dart';

import '../theme/app_theme.dart';
import '../theme/icons.dart';
import '../i18n/i18n.dart';
import 'widgets.dart';

/// Bandeau « assistant de démonstration » ; [onSettings] est absent quand aucune IA n'est configurable.
class DemoBanner extends StatelessWidget {
  final VoidCallback? onSettings;
  const DemoBanner({super.key, this.onSettings});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final text = onSettings == null
        ? t('Assistant de démonstration : les réponses sont simulées.')
        : t('Assistant de démonstration : configure une IA dans les réglages.');
    return Container(
      key: const ValueKey('demo-banner'),
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
      decoration: BoxDecoration(
        color: p.accentSoft,
        border: Border.all(color: p.accent),
      ),
      child: Row(
        children: [
          Ico('spark', size: 14, color: p.accent),
          const SizedBox(width: 10),
          Expanded(
            child: Text(text, style: Fonts.sans(size: 13.5, color: p.text)),
          ),
          if (onSettings != null) ...[
            const SizedBox(width: 10),
            Btn(label: t('Réglages'), onTap: onSettings),
          ],
        ],
      ),
    );
  }
}
