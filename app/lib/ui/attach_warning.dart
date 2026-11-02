//! Avertissement du composeur : le message parle d'une pièce jointe mais aucune n'est jointe.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Bandeau au-dessus de la barre d'envoi : joindre un fichier, ou envoyer quand même.
class AttachWarning extends StatelessWidget {
  final double padding;
  final VoidCallback onAttach, onSend;
  const AttachWarning({
    super.key,
    required this.padding,
    required this.onAttach,
    required this.onSend,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      padding: EdgeInsets.symmetric(horizontal: padding, vertical: 12),
      decoration: BoxDecoration(
        color: p.accentSoft,
        border: Border(top: BorderSide(color: p.accent)),
      ),
      child: Wrap(
        spacing: 14,
        runSpacing: 10,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          Ico('warn', size: 16, color: p.accent),
          Text(
            t(
              'Ton message parle d’une pièce jointe, mais aucun fichier n’est joint.',
            ),
            style: Fonts.sans(size: 13.5, color: p.text),
          ),
          Btn(
            key: const ValueKey('attach-warning-attach'),
            icon: 'clip',
            label: t('Joindre un fichier'),
            onTap: onAttach,
          ),
          Btn(
            key: const ValueKey('attach-warning-send'),
            icon: 'send',
            label: t('Envoyer quand même'),
            onTap: onSend,
          ),
        ],
      ),
    );
  }
}
