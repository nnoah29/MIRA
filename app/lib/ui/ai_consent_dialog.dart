//! Boîte de consentement : nomme le service qui recevra la conversation et le mail ouvert.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Demande l'accord de l'utilisateur pour envoyer ses textes à [providerName] ; faux s'il refuse ou ferme.
Future<bool> showAiConsentDialog(
  BuildContext context,
  String providerName,
) async {
  final p = context.pal;
  final ok = await showDialog<bool>(
    context: context,
    builder: (d) => Dialog(
      backgroundColor: p.bgSide,
      shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 460),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                t('Envoyer tes messages à {0} ?', [providerName]),
                style: Fonts.sans(size: 16, w: FontWeight.w600, color: p.text),
              ),
              const SizedBox(height: 10),
              Text(
                t(
                  'Pour répondre, l’assistant envoie à {0} ta conversation et le texte du mail ouvert. Ce service applique sa propre politique de confidentialité. Rien n’est envoyé sans ton accord ; supprime la connexion dans les réglages pour le retirer.',
                  [providerName],
                ),
                style: Fonts.sans(size: 13.5, color: p.muted, height: 1.5),
              ),
              const SizedBox(height: 20),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  Btn(
                    key: const ValueKey('ai-consent-refuse'),
                    label: t('Refuser'),
                    onTap: () => Navigator.pop(d, false),
                  ),
                  const SizedBox(width: 10),
                  Btn(
                    key: const ValueKey('ai-consent-accept'),
                    label: t('Autoriser'),
                    go: true,
                    onTap: () => Navigator.pop(d, true),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    ),
  );
  return ok ?? false;
}
