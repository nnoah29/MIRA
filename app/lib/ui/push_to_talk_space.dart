//! Espace maintenu pour dicter phrase par phrase : appui court = un espace, appui long = dictée tant qu'on maintient.

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../voice/dictation_types.dart';
import '../voice/voice_services.dart';

enum _Phase { idle, pending, dictating }

/// Enveloppe un champ de texte : Espace maintenu au-delà du délai réglé dicte dans ce champ, un appui court reste un espace.
class PushToTalkSpace extends StatefulWidget {
  const PushToTalkSpace({
    super.key,
    required this.target,
    required this.child,
    this.onEdited,
    this.onProblem,
  });

  final TextEditingController target;
  final Widget child;

  /// Appelé quand l'espace d'un appui court est inséré (par exemple pour marquer un brouillon modifié).
  final VoidCallback? onEdited;
  final void Function(DictationProblem problem)? onProblem;

  @override
  State<PushToTalkSpace> createState() => _PushToTalkSpaceState();
}

class _PushToTalkSpaceState extends State<PushToTalkSpace> {
  _Phase _phase = _Phase.idle;
  Timer? _timer;
  String _textBefore = '';
  VoiceServices? _services;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _services = voiceServicesOf(context);
  }

  @override
  void dispose() {
    _timer?.cancel();
    if (_phase == _Phase.dictating) {
      _services?.dictation.release(widget.target);
    }
    super.dispose();
  }

  /// Dit si Espace peut démarrer une dictée : réglage actif, modèle installé, aucune dictée continue, ni modificateur, ni composition.
  bool _canHold(VoiceServices services) {
    final d = services.dictation;
    final free =
        d.state == DictationState.idle ||
        (d.state == DictationState.stopping &&
            d.mode == DictationMode.pushToTalk);
    final composing = widget.target.value.composing;
    final kb = HardwareKeyboard.instance;
    return services.preferences.holdSpaceEnabled &&
        free &&
        d.models.isInstalled(d.language) &&
        !(composing.isValid && !composing.isCollapsed) &&
        !(kb.isControlPressed ||
            kb.isAltPressed ||
            kb.isMetaPressed ||
            kb.isShiftPressed);
  }

  /// Aiguille une touche : Espace pilote la dictée, toute autre touche pendant l'attente fait passer l'espace d'abord.
  KeyEventResult _onKey(FocusNode node, KeyEvent event) {
    if (event.logicalKey != LogicalKeyboardKey.space) {
      if (_phase == _Phase.pending && event is KeyDownEvent) _flushSpace();
      return KeyEventResult.ignored;
    }
    if (event is KeyDownEvent) return _down();
    if (event is KeyUpEvent) return _up();
    return _phase == _Phase.idle
        ? KeyEventResult.ignored
        : KeyEventResult.handled;
  }

  /// Espace enfoncé : on retient la touche jusqu'au délai de maintien ou au relâchement.
  KeyEventResult _down() {
    if (_phase != _Phase.idle) return KeyEventResult.handled;
    final services = _services;
    if (services == null || !_canHold(services)) return KeyEventResult.ignored;
    _phase = _Phase.pending;
    _timer = Timer(services.preferences.holdDelay, _startDictation);
    return KeyEventResult.handled;
  }

  /// Espace relâché : un appui court insère l'espace, un appui long termine la dictée.
  KeyEventResult _up() {
    switch (_phase) {
      case _Phase.idle:
        return KeyEventResult.ignored;
      case _Phase.pending:
        _flushSpace();
      case _Phase.dictating:
        unawaited(_finishDictation(spaceIfNothing: true));
    }
    return KeyEventResult.handled;
  }

  /// Le délai est écoulé : la dictée phrase par phrase démarre dans le champ.
  void _startDictation() {
    final services = _services;
    if (services == null || !mounted) return _reset();
    _phase = _Phase.dictating;
    _textBefore = widget.target.text;
    unawaited(_run(services));
  }

  /// Lance la dictée et signale un échec (micro refusé, par exemple) une fois terminée.
  Future<void> _run(VoiceServices services) async {
    final d = services.dictation;
    await d.start(widget.target, mode: DictationMode.pushToTalk);
    final problem = d.problem;
    if (mounted && problem != null && d.state == DictationState.idle) {
      widget.onProblem?.call(problem);
    }
  }

  /// Arrête la dictée jusqu'au bout ; si rien n'a été reconnu, l'espace que la personne voulait est inséré.
  Future<void> _finishDictation({required bool spaceIfNothing}) async {
    final before = _textBefore;
    _reset();
    await _services?.dictation.stop();
    if (mounted && spaceIfNothing && widget.target.text == before) {
      _insertSpace();
    }
  }

  /// Un appui court (ou interrompu par une autre touche) : l'espace retenu est inséré maintenant.
  void _flushSpace() {
    _reset();
    _insertSpace();
  }

  /// Revient au repos sans rien insérer.
  void _reset() {
    _timer?.cancel();
    _timer = null;
    _phase = _Phase.idle;
  }

  /// Insère une espace à la place du curseur, en remplaçant la sélection éventuelle.
  void _insertSpace() {
    final value = widget.target.value;
    final text = value.text;
    final sel = value.selection;
    final start = sel.isValid ? sel.start : text.length;
    final end = sel.isValid ? sel.end : text.length;
    widget.target.value = TextEditingValue(
      text: '${text.substring(0, start)} ${text.substring(end)}',
      selection: TextSelection.collapsed(offset: start + 1),
    );
    widget.onEdited?.call();
  }

  /// Le champ perd le focus : l'attente est abandonnée, la dictée s'arrête sans espace.
  void _focusChanged(bool focused) {
    if (focused) return;
    if (_phase == _Phase.pending) _reset();
    if (_phase == _Phase.dictating) {
      unawaited(_finishDictation(spaceIfNothing: false));
    }
  }

  @override
  Widget build(BuildContext context) => Focus(
    canRequestFocus: false,
    skipTraversal: true,
    onKeyEvent: _onKey,
    onFocusChange: _focusChanged,
    child: widget.child,
  );
}
