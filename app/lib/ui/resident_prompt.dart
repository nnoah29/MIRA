import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../engine/remote/resident.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

const String _answered = 'residentAnswered';

/// Pose une seule fois la question « Garder MIRA actif en arrière-plan ? » ; sans unité installée, ne demande rien.
Future<void> askResidentOnce({
  required BuildContext context,
  required SharedPreferences prefs,
  required ResidentController controller,
}) async {
  if (prefs.getString(_answered) == '1') return;
  if (!await controller.isAvailable() || !context.mounted) return;
  final yes = await showDialog<bool>(
    context: context,
    barrierDismissible: false,
    builder: (dialog) {
      final p = dialog.pal;
      return Dialog(
        key: const ValueKey('resident-dialog'),
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(
                  t('Garder MIRA actif en arrière-plan ?'),
                  style: Fonts.sans(
                    size: 16,
                    w: FontWeight.w500,
                    color: p.text,
                  ),
                ),
                const SizedBox(height: 10),
                Text(
                  t(
                    'MIRA peut rester actif discrètement quand sa fenêtre est fermée, pour continuer la synchronisation et afficher vos notifications. Vous pourrez changer ce choix dans les réglages.',
                  ),
                  style: Fonts.sans(size: 13.5, color: p.dim),
                ),
                const SizedBox(height: 22),
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    Btn(
                      label: t('Non'),
                      onTap: () => Navigator.of(dialog).pop(false),
                    ),
                    const SizedBox(width: 8),
                    Btn(
                      label: t('Oui'),
                      go: true,
                      onTap: () => Navigator.of(dialog).pop(true),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      );
    },
  );
  await prefs.setString(_answered, '1');
  if (yes == true) await controller.enable();
}
