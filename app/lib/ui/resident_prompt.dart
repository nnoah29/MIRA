import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../engine/remote/resident.dart';
import '../i18n/i18n.dart';

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
    builder: (dialog) => AlertDialog(
      title: Text(t('Garder MIRA actif en arrière-plan ?')),
      content: Text(
        t(
          'MIRA peut rester actif discrètement quand sa fenêtre est fermée, pour continuer la synchronisation et afficher vos notifications. Vous pourrez changer ce choix dans les réglages.',
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(dialog).pop(false),
          child: Text(t('Non')),
        ),
        FilledButton(
          onPressed: () => Navigator.of(dialog).pop(true),
          child: Text(t('Oui')),
        ),
      ],
    ),
  );
  await prefs.setString(_answered, '1');
  if (yes == true) await controller.enable();
}
