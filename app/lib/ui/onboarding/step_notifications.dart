import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../engine/remote/resident.dart';
import '../../i18n/i18n.dart';
import '../../platform_info.dart';
import '../../state/store.dart';
import '../../theme/app_theme.dart';
import '../account_form_widgets.dart';
import '../resident_prompt.dart';
import 'onboarding_controller.dart';
import 'step_layout.dart';

/// Contrôleur du mode résident utilisé par la page ; remplaçable dans les tests.
ResidentController Function() residentForOnboarding =
    ResidentController.forPlatform;

/// Page 6 : notifications de nouveaux messages et mode résident, chacun avec ce qu'il fait.
class NotificationsStep extends StatefulWidget {
  final OnboardingController controller;
  const NotificationsStep({super.key, required this.controller});

  @override
  State<NotificationsStep> createState() => _NotificationsStepState();
}

class _NotificationsStepState extends State<NotificationsStep> {
  ResidentController? _resident;
  bool _available = false;
  bool _enabled = false;

  @override
  void initState() {
    super.initState();
    if (platformHasDaemon) _load();
  }

  Future<void> _load() async {
    final resident = residentForOnboarding();
    final available = await resident.isAvailable();
    final enabled = available && await resident.isEnabled();
    if (!mounted) return;
    setState(() {
      _resident = resident;
      _available = available;
      _enabled = enabled;
    });
  }

  Future<void> _setResident(bool on) async {
    final resident = _resident;
    final prefs = context.read<MailStore>().prefs;
    if (resident == null) return;
    try {
      on ? await resident.enable() : await resident.disable();
      if (mounted) setState(() => _enabled = on);
    } catch (_) {
      await _load();
    }
    await prefs.setString(residentAnsweredKey, '1');
  }

  Widget _card(Pal p, String title, String body, Widget control, {Key? key}) =>
      Container(
        key: key,
        margin: const EdgeInsets.only(bottom: 14),
        padding: const EdgeInsets.all(18),
        decoration: BoxDecoration(border: Border.all(color: p.line)),
        child: Wrap(
          alignment: WrapAlignment.spaceBetween,
          crossAxisAlignment: WrapCrossAlignment.center,
          runSpacing: 12,
          spacing: 24,
          children: [
            ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 440),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title,
                    style: Fonts.sans(
                      size: 16,
                      w: FontWeight.w500,
                      color: p.text,
                    ),
                  ),
                  const SizedBox(height: 3),
                  Text(
                    body,
                    style: Fonts.sans(size: 14, color: p.dim, height: 1.4),
                  ),
                ],
              ),
            ),
            control,
          ],
        ),
      );

  Widget _onOff(String prefix, bool value, ValueChanged<bool> onChanged) =>
      Segments<String>(
        keyPrefix: prefix,
        options: [('1', t('Activé')), ('0', t('Désactivé'))],
        value: value ? '1' : '0',
        onChanged: (v) => onChanged(v == '1'),
      );

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    return StepLayout(
      title: t('NOTIFICATIONS'),
      lead: t(
        'Choisis comment MIRA te prévient. Tout se change dans les réglages.',
      ),
      children: [
        _card(
          p,
          t('Nouveaux messages'),
          t('Une notification quand un message arrive.'),
          _onOff(
            'notify',
            s.notifySettings.enabled,
            (on) => s.setNotify('notify', on ? '1' : '0'),
          ),
        ),
        if (_available)
          _card(
            p,
            t('Garder MIRA actif en arrière-plan'),
            t(
              'Les notifications et la synchronisation continuent fenêtre fermée.',
            ),
            _onOff('resident', _enabled, _setResident),
            key: const ValueKey('onb-resident'),
          ),
      ],
    );
  }
}
