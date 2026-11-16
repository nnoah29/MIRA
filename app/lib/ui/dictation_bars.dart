//! Barres animées du micro : elles suivent le niveau sonore réel pour montrer que la voix est bien captée.

import 'package:flutter/material.dart';

import '../theme/app_theme.dart';

const barCount = 5;
const _minHeight = 3.0;
const _maxHeight = 16.0;

/// Cinq barres qui se décalent d'un cran à chaque niveau reçu ; ternes tant que le modèle charge.
class DictationBars extends StatefulWidget {
  const DictationBars({super.key, required this.level, required this.ready});

  final ValueNotifier<double> level;
  final bool ready;

  @override
  State<DictationBars> createState() => _DictationBarsState();
}

class _DictationBarsState extends State<DictationBars> {
  final List<double> _history = List<double>.filled(
    barCount,
    0,
    growable: true,
  );

  @override
  void initState() {
    super.initState();
    widget.level.addListener(_push);
  }

  @override
  void didUpdateWidget(DictationBars old) {
    super.didUpdateWidget(old);
    if (identical(old.level, widget.level)) return;
    old.level.removeListener(_push);
    widget.level.addListener(_push);
  }

  @override
  void dispose() {
    widget.level.removeListener(_push);
    super.dispose();
  }

  /// Décale l'historique et ajoute le dernier niveau.
  void _push() {
    setState(() {
      _history
        ..removeAt(0)
        ..add(widget.level.value);
    });
  }

  @override
  Widget build(BuildContext context) {
    final color = context.pal.accent.withValues(alpha: widget.ready ? 1 : .45);
    return RepaintBoundary(
      child: SizedBox(
        key: const ValueKey('voice-mic-bars'),
        height: _maxHeight,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [for (final v in _history) _bar(v, color)],
        ),
      ),
    );
  }

  /// Une barre : sa hauteur suit le niveau, avec une courte transition.
  Widget _bar(double v, Color color) => Padding(
    padding: const EdgeInsets.symmetric(horizontal: 1),
    child: AnimatedContainer(
      key: const ValueKey('voice-mic-bar'),
      duration: const Duration(milliseconds: 120),
      width: 3,
      height: _minHeight + (_maxHeight - _minHeight) * v,
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(1.5),
      ),
    ),
  );
}
