import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../app_info.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Ouvre une adresse dans le navigateur ; remplacée dans les tests.
Future<bool> Function(Uri url) openLink = launchUrl;

/// Onglet « À propos » : l'application, sa politique de confidentialité et son auteur.
class AboutSettings extends StatelessWidget {
  const AboutSettings({super.key});

  Widget _section(
    BuildContext context,
    String title,
    String body,
    List<_Link> links,
  ) {
    final p = context.pal;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 18),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: p.lineSoft)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 6),
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 620),
            child: Text(body, style: Fonts.sans(size: 13.5, color: p.dim)),
          ),
          const SizedBox(height: 14),
          Wrap(
            spacing: 10,
            runSpacing: 10,
            children: [
              for (final l in links)
                Btn(
                  key: ValueKey('about-${l.url}'),
                  icon: 'link',
                  label: l.label,
                  onTap: () => openLink(Uri.parse(l.url)),
                ),
            ],
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _section(
          context,
          t('MIRA {0}', [appVersion]),
          t(
            'Un client mail libre pour Linux et Android : plusieurs comptes, lecture hors ligne, thème sombre ou clair. Licence MIT.',
          ),
          [
            _Link(t('Site de MIRA'), siteUrl),
            _Link(t('Code source'), sourceUrl),
          ],
        ),
        _section(
          context,
          t('Confidentialité'),
          t(
            'MIRA n’a pas de serveur et n’envoie aucune télémétrie : tes messages restent sur ton appareil et ne passent que par ton fournisseur de courrier.',
          ),
          [
            _Link(t('Politique de confidentialité'), privacyUrl),
            _Link(t('Conditions d’utilisation'), termsUrl),
          ],
        ),
        _section(
          context,
          t('Auteur'),
          t('MIRA est conçu et développé par Noah TOFFA (@nnoah29).'),
          [_Link(t('Site de l’auteur'), authorUrl)],
        ),
      ],
    );
  }
}

class _Link {
  final String label, url;
  const _Link(this.label, this.url);
}
