//! Avertissement avant de connecter l'abonnement d'un service : ce qui part, ce que ça coûte, comment s'en défaire.

import 'package:flutter/material.dart';

import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'ai_connection_card.dart';
import 'widgets.dart';

/// Montre l'avertissement ; rend `true` quand l'utilisateur accepte de continuer.
Future<bool> confirmSessionLogin(
  BuildContext context,
  AiProviderInfo provider,
) async {
  final name = aiProviderTitle(provider);
  final accepted = await showDialog<bool>(
    context: context,
    barrierDismissible: false,
    builder: (dialog) {
      final p = dialog.pal;
      return Dialog(
        key: const ValueKey('ai-session-warning'),
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 560),
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: _body(dialog, p, name),
          ),
        ),
      );
    },
  );
  return accepted == true;
}

Widget _point(Pal p, String text) => Padding(
  padding: const EdgeInsets.only(bottom: 8),
  child: Text('•  $text', style: Fonts.sans(size: 13.5, color: p.dim)),
);

Widget _body(BuildContext dialog, Pal p, String name) => Column(
  mainAxisSize: MainAxisSize.min,
  crossAxisAlignment: CrossAxisAlignment.stretch,
  children: [
    Text(
      t('Utiliser ton abonnement {0} dans MIRA ?', [name]),
      style: Fonts.sans(size: 16, w: FontWeight.w500, color: p.text),
    ),
    const SizedBox(height: 12),
    _point(
      p,
      t(
        'Le contenu que tu envoies à l’assistant est traité par le service, selon les conditions de ton compte. Les personnes qui t’écrivent n’ont pas consenti à cet envoi.',
      ),
    ),
    _point(
      p,
      t(
        'L’usage compte dans ton abonnement, avec un plafond que tu règles côté service.',
      ),
    ),
    _point(
      p,
      t('Tu peux déconnecter MIRA à tout moment dans les réglages du service.'),
    ),
    _point(
      p,
      t('Cette connexion est en préversion : elle peut cesser de fonctionner.'),
    ),
    const SizedBox(height: 14),
    Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Btn(
          key: const ValueKey('ai-session-warning-cancel'),
          label: t('Annuler'),
          onTap: () => Navigator.of(dialog).pop(false),
        ),
        const SizedBox(width: 8),
        Btn(
          key: const ValueKey('ai-session-warning-accept'),
          label: t('Continuer'),
          go: true,
          onTap: () => Navigator.of(dialog).pop(true),
        ),
      ],
    ),
  ],
);
