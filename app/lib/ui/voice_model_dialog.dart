//! Confirmation avant de télécharger un modèle de dictée : taille, source, ce qui est envoyé, comment le retirer.

import 'dart:async';

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import '../voice/voice_model_catalog.dart';
import '../voice/voice_model_store.dart';
import '../voice/voice_services.dart';
import 'ai_local_text.dart';
import 'widgets.dart';

/// Montre la confirmation ; rend `true` quand l'utilisateur accepte de télécharger.
Future<bool> confirmVoiceModelInstall(
  BuildContext context,
  VoiceModelEntry entry,
) async {
  final accepted = await showDialog<bool>(
    context: context,
    barrierDismissible: false,
    builder: (dialog) {
      final p = dialog.pal;
      return Dialog(
        key: const ValueKey('voice-confirm'),
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: _body(dialog, p, entry),
          ),
        ),
      );
    },
  );
  return accepted == true;
}

/// Affiche un message à l'utilisateur ; `err` pour une erreur.
typedef VoiceNotify = void Function(String message, {bool err});

/// Propose le téléchargement du modèle de la langue active, le lance si l'utilisateur confirme et annonce son issue.
Future<void> offerVoiceModelInstall(
  BuildContext context,
  VoiceServices services, {
  required VoiceNotify notify,
}) async {
  final language = services.dictation.language;
  final store = services.models;
  final running = store.statusOf(language);
  if (running is VoiceModelDownloading) {
    notify(
      t('Téléchargement du modèle de dictée en cours : {0} %', [
        (running.progress * 100).round(),
      ]),
    );
    return;
  }
  final entry = store.catalog.firstWhere((e) => e.language == language);
  if (!await confirmVoiceModelInstall(context, entry)) return;
  notify(
    t(
      'Téléchargement du modèle de dictée lancé : tu peux continuer, suivi dans Réglages › Messagerie',
    ),
  );
  await store.install(language);
  final done = store.statusOf(language);
  if (done is VoiceModelInstalled) {
    notify(t('Modèle de dictée prêt : la dictée démarre'));
    unawaited(services.dictation.retry());
  } else if (done is VoiceModelFailed) {
    notify(t('Téléchargement du modèle de dictée impossible'), err: true);
  }
}

Widget _point(Pal p, String text) => Padding(
  padding: const EdgeInsets.only(bottom: 8),
  child: Text('•  $text', style: Fonts.sans(size: 13.5, color: p.dim)),
);

Widget _body(BuildContext dialog, Pal p, VoiceModelEntry entry) => Column(
  mainAxisSize: MainAxisSize.min,
  crossAxisAlignment: CrossAxisAlignment.stretch,
  children: [
    Text(
      t('Télécharger le modèle de dictée {0} ?', [entry.language.label]),
      style: Fonts.sans(size: 16, w: FontWeight.w500, color: p.text),
    ),
    const SizedBox(height: 12),
    _point(
      p,
      t('Taille : {0}. Source : {1}.', [
        localSize(entry.archiveBytes),
        entry.url.host,
      ]),
    ),
    _point(
      p,
      t(
        'Seul ce téléchargement passe par Internet : ta voix et le texte dicté ne quittent jamais l’ordinateur.',
      ),
    ),
    _point(
      p,
      t(
        'Tu peux le supprimer dans les réglages à tout moment : tout l’espace est rendu.',
      ),
    ),
    const SizedBox(height: 14),
    Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Btn(
          key: const ValueKey('voice-confirm-cancel'),
          label: t('Annuler'),
          onTap: () => Navigator.of(dialog).pop(false),
        ),
        const SizedBox(width: 8),
        Btn(
          key: const ValueKey('voice-confirm-accept'),
          label: t('Télécharger'),
          go: true,
          onTap: () => Navigator.of(dialog).pop(true),
        ),
      ],
    ),
  ],
);
