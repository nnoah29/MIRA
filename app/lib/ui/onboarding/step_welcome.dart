import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../i18n/i18n.dart';
import '../../state/store.dart';
import '../../theme/app_theme.dart';
import '../mira_logo.dart';
import '../widgets.dart';
import 'onboarding_controller.dart';
import 'step_layout.dart';

/// Page 1 : promesse de MIRA et choix de la langue.
class WelcomeStep extends StatelessWidget {
  final OnboardingController controller;
  const WelcomeStep({super.key, required this.controller});

  Widget _lang(MailStore s, Pal p) => Container(
    decoration: BoxDecoration(border: Border.all(color: p.line)),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final (i, l) in AppLang.values.indexed)
          Hov(
            key: ValueKey('onb-lang-${l.code}'),
            onTap: () => s.setLang(l),
            builder: (_, h) => Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 9),
              decoration: BoxDecoration(
                color: s.lang == l ? p.accentSoft : null,
                border: i > 0 ? Border(left: BorderSide(color: p.line)) : null,
              ),
              child: Mono(
                l.label,
                ls: .12,
                color: s.lang == l ? p.accent : (h ? p.text : p.muted),
                height: null,
              ),
            ),
          ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        MiraLogo(color: p.accent, size: 64),
        const SizedBox(height: 28),
        StepLayout(
          title: t('TES MAILS. CHEZ TOI.'),
          lead: t(
            'Un client mail moderne et soigné, libre et gratuit, sur ton appareil.',
          ),
          children: [_lang(s, p)],
        ),
      ],
    );
  }
}
