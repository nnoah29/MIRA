import 'package:flutter/material.dart';

import '../../app_info.dart';
import '../../i18n/i18n.dart';
import '../../theme/app_theme.dart';
import '../about_settings.dart';
import '../widgets.dart';
import 'onboarding_controller.dart';
import 'step_layout.dart';

/// Page 2 : ce qui reste sur l'appareil, aucun serveur ni télémétrie, images distantes bloquées.
class PrivacyStep extends StatelessWidget {
  final OnboardingController controller;
  const PrivacyStep({super.key, required this.controller});

  Widget _point(Pal p, String title, String body) => Padding(
    padding: const EdgeInsets.only(bottom: 20),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: Fonts.sans(size: 18, w: FontWeight.w500, color: p.text),
        ),
        const SizedBox(height: 4),
        Text(body, style: Fonts.sans(size: 15, color: p.dim, height: 1.45)),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return StepLayout(
      title: t('TA VIE PRIVÉE'),
      children: [
        _point(
          p,
          t('Sur ton appareil'),
          t(
            'Tes messages sont copiés sur ton appareil et ne passent que par ton fournisseur de courrier.',
          ),
        ),
        _point(
          p,
          t('Aucun serveur MIRA'),
          t('Aucune télémétrie, aucun compte à créer chez nous.'),
        ),
        _point(
          p,
          t('Images distantes bloquées'),
          t(
            'Les images d’un message sont bloquées par défaut : tu choisis quand les afficher.',
          ),
        ),
        Btn(
          key: const ValueKey('onb-privacy-link'),
          icon: 'link',
          label: t('Politique de confidentialité'),
          onTap: () => openLink(Uri.parse(privacyUrl)),
        ),
      ],
    );
  }
}
