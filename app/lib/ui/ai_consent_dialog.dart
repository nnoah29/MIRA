//! Boîte de consentement : nomme le service qui recevra la conversation et le mail ouvert.

import 'package:flutter/material.dart';

import '../engine/ai_privacy_models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'ai_kind_names.dart';
import 'rule_widgets.dart';
import 'widgets.dart';

/// Demande l'accord de l'utilisateur pour envoyer ses textes à [providerName] (et à la machine [host] si l'adresse est libre) ; rend les types de contenu cochés, `null` s'il refuse ou ferme.
Future<List<String>?> showAiConsentDialog(
  BuildContext context,
  String providerName, {
  String? host,
}) => showDialog<List<String>>(
  context: context,
  builder: (d) => _ConsentDialog(providerName: providerName, host: host),
);

class _ConsentDialog extends StatefulWidget {
  final String providerName;
  final String? host;
  const _ConsentDialog({required this.providerName, this.host});

  @override
  State<_ConsentDialog> createState() => _ConsentDialogState();
}

class _ConsentDialogState extends State<_ConsentDialog> {
  final Set<String> ticked = {aiKindMail};

  void _toggle(String kind) => setState(() {
    ticked.contains(kind) ? ticked.remove(kind) : ticked.add(kind);
  });

  Widget _kinds() => Wrap(
    spacing: 8,
    runSpacing: 8,
    children: [
      for (final kind in aiKnownKinds)
        RuleChip(
          key: ValueKey('ai-consent-kind-$kind'),
          label: aiKindName(kind),
          selected: ticked.contains(kind),
          onTap: () => _toggle(kind),
        ),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final name = widget.providerName;
    return Dialog(
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
                t('Envoyer tes messages à {0} ?', [name]),
                style: Fonts.sans(size: 16, w: FontWeight.w600, color: p.text),
              ),
              const SizedBox(height: 10),
              Text(
                t(
                  'Pour répondre, l’assistant envoie à {0} ta conversation et les contenus que tu autorises ci-dessous. Ce service applique sa propre politique de confidentialité. Rien n’est envoyé sans ton accord ; supprime la connexion dans les réglages pour le retirer.',
                  [name],
                ),
                style: Fonts.sans(size: 13.5, color: p.muted, height: 1.5),
              ),
              if (widget.host != null)
                Padding(
                  padding: const EdgeInsets.only(top: 10),
                  child: Text(
                    t('Adresse du service : {0}', [widget.host]),
                    key: const ValueKey('ai-consent-host'),
                    style: Fonts.sans(
                      size: 13.5,
                      w: FontWeight.w500,
                      color: p.text,
                    ),
                  ),
                ),
              const SizedBox(height: 16),
              Mono(
                t('Contenu autorisé'),
                size: 10.5,
                ls: .12,
                color: p.muted,
                height: null,
              ),
              const SizedBox(height: 8),
              _kinds(),
              const SizedBox(height: 20),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  Btn(
                    key: const ValueKey('ai-consent-refuse'),
                    label: t('Refuser'),
                    onTap: () => Navigator.pop(context),
                  ),
                  const SizedBox(width: 10),
                  Btn(
                    key: const ValueKey('ai-consent-accept'),
                    label: t('Autoriser'),
                    go: true,
                    onTap: () => Navigator.pop(context, ticked.toList()),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
