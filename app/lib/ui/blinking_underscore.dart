//! Tiret orange clignotant du titre de la liste : une image par bascule, aucune quand la fenêtre n'a pas le focus.

import 'dart:async';

import 'package:flutter/material.dart';

import '../theme/app_theme.dart';

const _halfPeriod = Duration(milliseconds: 530);

/// Tiret du bas « _ » orange qui clignote, comme un curseur de terminal.
class BlinkingUnderscore extends StatefulWidget {
  const BlinkingUnderscore({super.key});
  @override
  State<BlinkingUnderscore> createState() => _BlinkingUnderscoreState();
}

class _BlinkingUnderscoreState extends State<BlinkingUnderscore>
    with WidgetsBindingObserver {
  Timer? _timer;
  bool _lit = true;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _follow(WidgetsBinding.instance.lifecycleState);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) => _follow(state);

  /// Clignote tant que la fenêtre est au premier plan ; sinon le minuteur est coupé et le tiret reste allumé.
  void _follow(AppLifecycleState? state) {
    final active = state == null || state == AppLifecycleState.resumed;
    if (!active) {
      _timer?.cancel();
      _timer = null;
      if (!_lit) setState(() => _lit = true);
      return;
    }
    _timer ??= Timer.periodic(_halfPeriod, (_) => setState(() => _lit = !_lit));
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => RepaintBoundary(
    child: Opacity(
      opacity: _lit ? 1 : 0,
      child: Container(width: 18, height: 4, color: context.pal.accent),
    ),
  );
}
