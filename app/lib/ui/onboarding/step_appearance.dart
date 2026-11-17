import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../i18n/i18n.dart';
import '../../state/store.dart';
import '../../theme/app_theme.dart';
import '../widgets.dart';
import 'onboarding_controller.dart';
import 'step_layout.dart';

/// Page 3 : thème clair ou sombre, appliqué tout de suite.
class AppearanceStep extends StatelessWidget {
  final OnboardingController controller;
  const AppearanceStep({super.key, required this.controller});

  Widget _bar(Color c, double w) => Container(
    width: w,
    height: 6,
    margin: const EdgeInsets.only(bottom: 6),
    color: c,
  );

  Widget _preview(Pal q) => Container(
    height: 110,
    decoration: BoxDecoration(
      color: q.bg,
      border: Border.all(color: q.line),
    ),
    child: Row(
      children: [
        Container(
          width: 42,
          color: q.bgSide,
          padding: const EdgeInsets.all(8),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [_bar(q.accent, 20), _bar(q.dim, 24), _bar(q.dim, 18)],
          ),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.all(10),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _bar(q.text, 70),
                _bar(q.muted, 110),
                _bar(q.dim, 90),
                _bar(q.muted, 100),
              ],
            ),
          ),
        ),
      ],
    ),
  );

  Widget _card(MailStore s, Pal p, String value, String label, Pal q) {
    final on = s.theme == value;
    return Hov(
      key: ValueKey('onb-theme-$value'),
      onTap: () => s.setSetting('theme', value),
      builder: (_, h) => Container(
        width: 250,
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: on ? p.accentSoft : (h ? p.surface : null),
          border: Border.all(color: on ? p.accent : p.line),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _preview(q),
            const SizedBox(height: 12),
            Row(
              children: [
                Mono(label, color: on ? p.accent : p.muted, height: null),
                if (on) ...[
                  const SizedBox(width: 8),
                  Icon(
                    Icons.check,
                    key: ValueKey('onb-theme-$value-on'),
                    size: 14,
                    color: p.accent,
                  ),
                ],
              ],
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    return StepLayout(
      title: t('APPARENCE'),
      lead: t('Choisis ton thème. Tu pourras le changer dans les réglages.'),
      children: [
        Wrap(
          spacing: 16,
          runSpacing: 16,
          children: [
            _card(s, p, 'dark', t('Sombre'), Pal.dark),
            _card(s, p, 'light', t('Clair'), Pal.light),
          ],
        ),
      ],
    );
  }
}
