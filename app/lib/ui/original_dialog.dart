import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Montre la source brute du message (en-têtes et corps tels que reçus), à copier ou enregistrer.
Future<void> showOriginal(BuildContext context, Mail m) async {
  final s = context.read<MailStore>();
  final text = await s.originalOf(m);
  if (!context.mounted) return;
  if (text == null) {
    s.toast(t('Original indisponible'), err: true);
    return;
  }
  await showDialog<void>(
    context: context,
    builder: (ctx) {
      final p = ctx.pal;
      return Dialog(
        key: const ValueKey('original-dialog'),
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 900, maxHeight: 700),
          child: Padding(
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        t('Message original'),
                        style: Fonts.sans(
                          size: 15,
                          w: FontWeight.w500,
                          color: p.text,
                        ),
                      ),
                    ),
                    Btn(
                      label: t('Copier'),
                      onTap: () {
                        Clipboard.setData(ClipboardData(text: text));
                        s.toast(t('Copié'));
                      },
                    ),
                    const SizedBox(width: 8),
                    Btn(
                      label: t('Enregistrer (.eml)'),
                      onTap: () => s.saveFile(
                        'message.eml',
                        Uint8List.fromList(utf8.encode(text)),
                      ),
                    ),
                    const SizedBox(width: 8),
                    IBtn(
                      'x',
                      tip: t('Fermer'),
                      onTap: () => Navigator.pop(ctx),
                    ),
                  ],
                ),
                const SizedBox(height: 14),
                Expanded(
                  child: Container(
                    decoration: BoxDecoration(
                      color: p.surface,
                      border: Border.all(color: p.line),
                    ),
                    child: SingleChildScrollView(
                      padding: const EdgeInsets.all(12),
                      child: SelectableText(
                        text,
                        style: Fonts.mono(size: 12, color: p.text),
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      );
    },
  );
}
