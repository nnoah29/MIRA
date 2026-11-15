//! Bouton micro : démarre ou arrête la dictée dans un champ, et signale ce qui l'empêche de fonctionner.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import '../voice/dictation_controller.dart';
import '../voice/dictation_types.dart';
import '../voice/voice_services.dart';
import 'widgets.dart';

/// Bouton de dictée lié à un champ de texte ; invisible quand les services de dictée ne sont pas fournis.
class MicButton extends StatefulWidget {
  const MicButton({
    super.key,
    required this.target,
    this.onProblem,
    this.box = 34,
  });

  final TextEditingController target;
  final void Function(DictationProblem problem)? onProblem;
  final double box;

  @override
  State<MicButton> createState() => _MicButtonState();
}

class _MicButtonState extends State<MicButton> {
  DictationController? _dictation;
  bool _started = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final next = voiceServicesOf(context)?.dictation;
    if (identical(next, _dictation)) return;
    _dictation?.removeListener(_changed);
    _dictation = next;
    next?.addListener(_changed);
  }

  @override
  void dispose() {
    _dictation?.removeListener(_changed);
    _dictation?.release(widget.target);
    super.dispose();
  }

  /// Signale l'échec d'une dictée que ce bouton a lancée, une fois la dictée terminée.
  void _changed() {
    final d = _dictation;
    if (d == null || !_started || d.state != DictationState.idle) return;
    _started = false;
    final problem = d.problem;
    if (problem != null) widget.onProblem?.call(problem);
  }

  /// Dit si la dictée en cours écrit dans le champ de ce bouton.
  bool get _mine {
    final d = _dictation;
    return d != null &&
        identical(d.target, widget.target) &&
        d.state != DictationState.idle;
  }

  /// Démarre la dictée sur ce champ, ou l'arrête si elle y est déjà.
  void _toggle() {
    final d = _dictation;
    if (d == null) return;
    if (_mine) {
      d.stop();
    } else {
      _started = true;
      d.start(widget.target);
    }
  }

  @override
  Widget build(BuildContext context) {
    final d = voiceServicesOf(context)?.dictation;
    if (d == null) return const SizedBox.shrink();
    return ListenableBuilder(
      listenable: d,
      builder: (context, _) {
        final on = _mine && d.state == DictationState.listening;
        return Stack(
          alignment: Alignment.topRight,
          children: [
            IBtn(
              'mic',
              key: const ValueKey('voice-mic-button'),
              tip: on ? t('Arrêter la dictée') : t('Dicter'),
              on: on,
              box: widget.box,
              onTap: _toggle,
            ),
            if (on)
              Padding(
                padding: const EdgeInsets.all(6),
                child: Container(
                  key: const ValueKey('voice-mic-listening'),
                  width: 7,
                  height: 7,
                  decoration: BoxDecoration(
                    color: context.pal.accent,
                    shape: BoxShape.circle,
                  ),
                ),
              ),
          ],
        );
      },
    );
  }
}
