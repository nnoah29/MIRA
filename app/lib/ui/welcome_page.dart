import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../app_info.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'about_settings.dart';
import 'mira_logo.dart';
import 'widgets.dart';

/// Accueil du premier lancement : présentation, langue, confidentialité, puis ajout du premier compte.
class WelcomePage extends StatelessWidget {
  const WelcomePage({super.key});

  Widget _point(Pal p, String title, String body) => Padding(
    padding: const EdgeInsets.only(bottom: 14),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
        ),
        Text(body, style: Fonts.sans(size: 13.5, color: p.dim)),
      ],
    ),
  );

  Widget _lang(BuildContext context, MailStore s, Pal p) => Container(
    decoration: BoxDecoration(border: Border.all(color: p.line)),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final (i, l) in AppLang.values.indexed)
          Hov(
            key: ValueKey('welcome-lang-${l.code}'),
            onTap: () => s.setLang(l),
            builder: (_, h) => Container(
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
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
    return Scaffold(
      backgroundColor: p.bg,
      body: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 520),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                MiraLogo(color: p.accent, size: 56),
                const SizedBox(height: 24),
                Text(
                  t('BIENVENUE DANS MIRA'),
                  style: Fonts.display(size: 30, ls: .02, color: p.text),
                ),
                const SizedBox(height: 20),
                _point(
                  p,
                  t('Tous tes comptes au même endroit'),
                  t('Gmail, Outlook, Yahoo ou n’importe quel serveur IMAP.'),
                ),
                _point(
                  p,
                  t('Hors ligne'),
                  t(
                    'Tes messages sont copiés sur ton appareil : lis et réponds sans réseau.',
                  ),
                ),
                _point(
                  p,
                  t('Privé par défaut'),
                  t(
                    'Aucun serveur MIRA, aucune télémétrie, images distantes bloquées.',
                  ),
                ),
                const SizedBox(height: 10),
                _lang(context, s, p),
                const SizedBox(height: 28),
                Wrap(
                  spacing: 12,
                  runSpacing: 12,
                  children: [
                    Btn(
                      key: const ValueKey('welcome-add'),
                      icon: 'plus',
                      label: t('Ajouter un compte'),
                      go: true,
                      onTap: () => s.finishWelcome(addAccount: true),
                    ),
                    Btn(
                      key: const ValueKey('welcome-later'),
                      label: t('Plus tard'),
                      onTap: () => s.finishWelcome(addAccount: false),
                    ),
                  ],
                ),
                const SizedBox(height: 24),
                Btn(
                  key: const ValueKey('welcome-privacy'),
                  icon: 'link',
                  label: t('Politique de confidentialité'),
                  onTap: () => openLink(Uri.parse(privacyUrl)),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
