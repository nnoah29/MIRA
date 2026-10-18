import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'account_form_widgets.dart';
import 'widgets.dart';

/// Réglages des notifications de nouveaux messages : activation, contenu, son, heures calmes, comptes.
class NotificationSettings extends StatelessWidget {
  const NotificationSettings({super.key});

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final cfg = s.notifySettings;
    final on = cfg.enabled;

    Widget row(String title, String hint, Widget ctl, {bool last = false}) =>
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
          decoration: BoxDecoration(
            border: last ? null : Border(bottom: BorderSide(color: p.lineSoft)),
          ),
          child: Wrap(
            alignment: WrapAlignment.spaceBetween,
            crossAxisAlignment: WrapCrossAlignment.center,
            runSpacing: 12,
            spacing: 24,
            children: [
              ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 480),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      title,
                      style: Fonts.sans(
                        size: 15,
                        w: FontWeight.w500,
                        color: p.text,
                      ),
                    ),
                    Text(hint, style: Fonts.sans(size: 13.5, color: p.dim)),
                  ],
                ),
              ),
              ctl,
            ],
          ),
        );

    Widget flag(String key, bool value) => Segments<String>(
      keyPrefix: key,
      options: [('1', t('Activé')), ('0', t('Désactivé'))],
      value: value ? '1' : '0',
      onChanged: (v) => s.setNotify(key, v),
    );

    final rest = [
      row(
        t('Contenu'),
        t(
          'Expéditeur et objet, ou seulement « Nouveau message » pour garder tes mails discrets à l’écran.',
        ),
        Segments<String>(
          keyPrefix: 'notifyPreview',
          options: [('1', t('Expéditeur et objet')), ('0', t('Discret'))],
          value: cfg.preview ? '1' : '0',
          onChanged: (v) => s.setNotify('notifyPreview', v),
        ),
      ),
      row(
        t('Son'),
        t('Joue le son de nouveau message du système.'),
        flag('notifySound', cfg.sound),
      ),
      row(
        t('Quand MIRA est au premier plan'),
        t('Notifier aussi quand tu as déjà la fenêtre de MIRA sous les yeux.'),
        flag('notifyFocused', cfg.whenFocused),
      ),
      row(
        t('Ne pas déranger'),
        t(
          'Aucune notification pendant ces heures ; les messages arrivent quand même.',
        ),
        Segments<String>(
          keyPrefix: 'notifyQuiet',
          options: [
            ('', t('Jamais')),
            ('22-7', '22 h – 7 h'),
            ('23-8', '23 h – 8 h'),
          ],
          value: prefsQuiet(s),
          onChanged: (v) => s.setNotify('notifyQuiet', v),
        ),
      ),
      if (s.accounts.length > 1 || s.source != null)
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                t('Comptes'),
                style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
              ),
              Text(
                t(
                  'Choisis les comptes dont les nouveaux messages sont notifiés.',
                ),
                style: Fonts.sans(size: 13.5, color: p.dim),
              ),
              const SizedBox(height: 12),
              for (final a in s.accounts)
                Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          a.email,
                          style: Fonts.sans(size: 14, color: p.text),
                        ),
                      ),
                      Segments<String>(
                        keyPrefix: 'notify-${a.id}',
                        options: [('1', t('Activé')), ('0', t('Désactivé'))],
                        value: cfg.muted.contains(a.id) ? '0' : '1',
                        onChanged: (v) => s.muteAccount(a.id, v == '0'),
                      ),
                    ],
                  ),
                ),
            ],
          ),
        ),
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        row(
          t('Nouveaux messages'),
          t(
            'Une notification du bureau à l’arrivée d’un message en Réception. Un clic l’ouvre dans MIRA.',
          ),
          flag('notify', on),
        ),
        Opacity(
          opacity: on ? 1 : .4,
          child: IgnorePointer(
            ignoring: !on,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: rest,
            ),
          ),
        ),
        Divider(height: 1, color: p.lineSoft),
        row(
          t('Essayer'),
          t(
            'Affiche une notification pour vérifier qu’elles apparaissent bien.',
          ),
          Btn(
            key: const ValueKey('notify-test'),
            label: t('Envoyer une notification de test'),
            onTap: s.testNotification,
          ),
          last: true,
        ),
      ],
    );
  }

  /// Valeur affichée des heures calmes : une plage proposée, sinon « Jamais ».
  static String prefsQuiet(MailStore s) {
    final raw = s.prefs.getString('notifyQuiet') ?? '';
    return const {'22-7', '23-8'}.contains(raw) ? raw : '';
  }
}
