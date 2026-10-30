//! Boîte d'autorisation d'un type de contenu : nomme le service et ce qu'il recevrait.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'ai_kind_names.dart';
import 'widgets.dart';

/// Demande si [providerName] peut recevoir les contenus [kinds] ; faux si l'utilisateur refuse ou ferme.
Future<bool> showAiContentDialog(
  BuildContext context,
  String providerName,
  List<String> kinds,
) async {
  final p = context.pal;
  final what = kinds.map(aiKindObject).join(', ');
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
                t('Autoriser {0} à recevoir : {1} ?', [providerName, what]),
                key: const ValueKey('ai-content-title'),
                style: Fonts.sans(size: 16, w: FontWeight.w600, color: p.text),
              ),
              const SizedBox(height: 10),
              Text(
                t(
                  'Ce contenu n’est pas encore autorisé pour ce service. Tu peux revenir sur ce choix dans les réglages de l’IA.',
                ),
                style: Fonts.sans(size: 13.5, color: p.muted, height: 1.5),
              ),
              const SizedBox(height: 20),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  Btn(
                    key: const ValueKey('ai-content-refuse'),
                    label: t('Refuser'),
                    onTap: () => Navigator.pop(d, false),
                  ),
                  const SizedBox(width: 10),
                  Btn(
                    key: const ValueKey('ai-content-accept'),
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
