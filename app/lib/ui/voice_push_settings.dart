//! Réglages de la dictée phrase par phrase : Espace maintenu, délai de maintien, durée de garde du modèle.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import '../voice/voice_services.dart';
import 'widgets.dart';

const _delaysMs = [300, 400, 600];
const _keepMinutes = [0, 1, 3, 10];

/// Section « Dicter phrase par phrase » de l'onglet Voix.
class VoicePushSettings extends StatefulWidget {
  const VoicePushSettings({super.key, required this.services});

  final VoiceServices services;

  @override
  State<VoicePushSettings> createState() => _VoicePushSettingsState();
}

class _VoicePushSettingsState extends State<VoicePushSettings> {
  VoiceServices get _s => widget.services;

  /// Active ou coupe Espace maintenu.
  Future<void> _setHold(bool enabled) async {
    await _s.preferences.setHoldSpaceEnabled(enabled);
    setState(() {});
  }

  /// Choisit le délai de maintien d'Espace.
  Future<void> _setDelay(int ms) async {
    await _s.preferences.setHoldDelay(Duration(milliseconds: ms));
    setState(() {});
  }

  /// Choisit la durée de garde du modèle et la transmet tout de suite au reconnaisseur.
  Future<void> _setKeep(int minutes) async {
    final keep = Duration(minutes: minutes);
    await _s.preferences.setKeepWarm(keep);
    _s.warm?.keepWarm = keep;
    setState(() {});
  }

  /// Un intitulé suivi d'une explication.
  Widget _heading(String title, String hint, Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(title, style: Fonts.sans(size: 13.5, color: p.muted)),
      Text(hint, style: Fonts.sans(size: 13, color: p.dim)),
      const SizedBox(height: 6),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final prefs = _s.preferences;
    return Column(
      key: const ValueKey('voice-push-settings'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          t('Dicter phrase par phrase'),
          style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
        ),
        const SizedBox(height: 4),
        _heading(
          t('Espace maintenu'),
          t(
            'Maintiens Espace dans un champ pour parler, relâche pour valider. Un appui court reste une espace.',
          ),
          p,
        ),
        Wrap(
          spacing: 8,
          children: [
            Btn(
              key: const ValueKey('voice-hold-on'),
              label: t('Activé'),
              on: prefs.holdSpaceEnabled,
              onTap: () => _setHold(true),
            ),
            Btn(
              key: const ValueKey('voice-hold-off'),
              label: t('Désactivé'),
              on: !prefs.holdSpaceEnabled,
              onTap: () => _setHold(false),
            ),
          ],
        ),
        const SizedBox(height: 14),
        _heading(
          t('Délai de maintien'),
          t('Durée d’appui sur Espace avant que la dictée démarre.'),
          p,
        ),
        Wrap(
          spacing: 8,
          children: [
            for (final ms in _delaysMs)
              Btn(
                key: ValueKey('voice-delay-$ms'),
                label: '$ms ms',
                on: prefs.holdDelay.inMilliseconds == ms,
                onTap: () => _setDelay(ms),
              ),
          ],
        ),
        const SizedBox(height: 14),
        _heading(
          t('Garder le modèle en mémoire'),
          t(
            'Après une dictée, le modèle reste chargé pour que la suivante démarre tout de suite ; il occupe environ 130 Mo pendant ce temps.',
          ),
          p,
        ),
        Wrap(
          spacing: 8,
          children: [
            for (final m in _keepMinutes)
              Btn(
                key: ValueKey('voice-warm-$m'),
                label: m == 0 ? t('Jamais') : '$m min',
                on: prefs.keepWarm.inMinutes == m,
                onTap: () => _setKeep(m),
              ),
          ],
        ),
        const SizedBox(height: 14),
      ],
    );
  }
}
