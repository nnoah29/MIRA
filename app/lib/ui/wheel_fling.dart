import 'dart:async';
import 'dart:math';

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

/// Silence après lequel le geste du pavé tactile est tenu pour fini (doigt levé).
const _gestureEnd = Duration(milliseconds: 60);

/// Fin du geste prise en compte pour sa vitesse (ms).
const _velocityWindow = 80.0;

/// Écart minimal compté entre deux événements (ms) : ils arrivent souvent par paquets, presque à la même milliseconde.
const _minInterval = 8.0;

/// Défilement transmis événement par événement (molette au-dessus d'une vue native),
/// prolongé par un élan quand le geste d'un pavé tactile s'arrête, comme le fait Flutter ailleurs.
class WheelFling {
  final _samples = <({double at, double delta})>[];
  Timer? _end;

  /// Fait défiler `position` de `delta` ; `at` (ms) est l'heure de l'événement dans la page.
  /// Un geste précis (pavé tactile) est suivi pour son élan, un cran de molette non.
  void scroll(
    ScrollPosition position,
    double delta, {
    required bool precise,
    required double at,
  }) {
    position.pointerScroll(delta);
    _end?.cancel();
    if (!precise) {
      _samples.clear();
      return;
    }
    _samples
      ..add((at: at, delta: delta))
      ..removeWhere((s) => s.at < at - _velocityWindow);
    _end = Timer(_gestureEnd, () => _launch(position));
  }

  /// Vitesse de la fin du geste (px/s) : distance parcourue depuis le premier événement retenu, sur la durée écoulée.
  double _velocity() {
    if (_samples.length < 2) return 0;
    final travelled = _samples.skip(1).fold(0.0, (sum, s) => sum + s.delta);
    final span = max(
      _samples.last.at - _samples.first.at,
      _minInterval * (_samples.length - 1),
    );
    final velocity = travelled / span * 1000;
    return velocity.clamp(-kMaxFlingVelocity, kMaxFlingVelocity);
  }

  /// Le doigt est levé : la vitesse du geste lance l'élan, avec la physique de défilement de l'app.
  void _launch(ScrollPosition position) {
    final velocity = _velocity();
    _samples.clear();
    if (velocity.abs() < kMinFlingVelocity || !position.hasPixels) return;
    if (position is ScrollPositionWithSingleContext) {
      position.goBallistic(velocity);
    }
  }

  void dispose() => _end?.cancel();
}
