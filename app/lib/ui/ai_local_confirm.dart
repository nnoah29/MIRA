//! Confirmation avant de télécharger un modèle local : taille, licence, où il va, comment s'en défaire.

import 'package:flutter/material.dart';

import '../engine/ai_local_models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'ai_local_text.dart';
import 'widgets.dart';

/// Montre la confirmation ; rend `true` quand l'utilisateur accepte de télécharger.
Future<bool> confirmLocalInstall(
  BuildContext context,
  LocalModelStatus model, {
  required bool serverInstalled,
}) async {
  final accepted = await showDialog<bool>(
    context: context,
    barrierDismissible: false,
    builder: (dialog) {
      final p = dialog.pal;
      return Dialog(
        key: const ValueKey('ai-local-confirm'),
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: _body(dialog, p, model, serverInstalled),
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

Widget _body(
  BuildContext dialog,
  Pal p,
  LocalModelStatus model,
  bool serverInstalled,
) => Column(
  mainAxisSize: MainAxisSize.min,
  crossAxisAlignment: CrossAxisAlignment.stretch,
  children: [
    Text(
      t('Télécharger {0} ?', [model.name]),
      style: Fonts.sans(size: 16, w: FontWeight.w500, color: p.text),
    ),
    const SizedBox(height: 12),
    _point(
      p,
      t('Taille : {0}. Licence : {1}.', [localSize(model.size), model.license]),
    ),
    if (!serverInstalled)
      _point(
        p,
        t(
          'Le moteur qui fait tourner le modèle (environ 25 Mo) est téléchargé avec lui.',
        ),
      ),
    _point(
      p,
      t(
        'Le modèle tourne sur ta machine : rien de ce que tu lui envoies ne quitte l’ordinateur.',
      ),
    ),
    _point(
      p,
      t('Tu peux le supprimer ici à tout moment : tout l’espace est rendu.'),
    ),
    const SizedBox(height: 14),
    Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Btn(
          key: const ValueKey('ai-local-confirm-cancel'),
          label: t('Annuler'),
          onTap: () => Navigator.of(dialog).pop(false),
        ),
        const SizedBox(width: 8),
        Btn(
          key: const ValueKey('ai-local-confirm-accept'),
          label: t('Télécharger'),
          go: true,
          onTap: () => Navigator.of(dialog).pop(true),
        ),
      ],
    ),
  ],
);
