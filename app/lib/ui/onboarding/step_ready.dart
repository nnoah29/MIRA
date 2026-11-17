import 'package:flutter/material.dart';

import '../../i18n/i18n.dart';
import '../../theme/app_theme.dart';
import '../widgets.dart';
import 'onboarding_controller.dart';
import 'step_layout.dart';

/// Page 7 : repères pour bien commencer, puis entrée dans la boîte.
class ReadyStep extends StatelessWidget {
  final OnboardingController controller;
  const ReadyStep({super.key, required this.controller});

  Widget _tip(Pal p, String title, String body) => Padding(
    padding: const EdgeInsets.only(bottom: 18),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: Fonts.sans(size: 17, w: FontWeight.w500, color: p.text),
        ),
        const SizedBox(height: 3),
        Text(body, style: Fonts.sans(size: 14.5, color: p.dim, height: 1.45)),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return StepLayout(
      title: t('TOUT EST PRÊT'),
      children: [
        _tip(
          p,
          t('Réglages'),
          t(
            'Le bouton Paramètres, en haut à droite : comptes, thème, notifications, règles.',
          ),
        ),
        _tip(
          p,
          t('Assistant IA'),
          t(
            'Le bouton Assistant IA, en haut : il résume et rédige avec ton accord.',
          ),
        ),
        _tip(
          p,
          t('Recherche'),
          t('La barre du haut cherche dans tous tes messages.'),
        ),
        Row(
          children: [
            const Kbd('?'),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                t('Appuie sur ? pour voir tous les raccourcis clavier.'),
                style: Fonts.sans(size: 14.5, color: p.dim),
              ),
            ),
          ],
        ),
        const SizedBox(height: 28),
        Btn(
          key: const ValueKey('onb-finish'),
          label: t('Ouvrir ma boîte'),
          go: true,
          onTap: controller.finish,
        ),
      ],
    );
  }
}
