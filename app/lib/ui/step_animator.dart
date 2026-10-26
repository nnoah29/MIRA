import 'package:flutter/animation.dart';
import 'package:flutter/scheduler.dart';

/// Anime une valeur par pas d'image plafonnés : une image lente n'envoie plus l'animation à sa fin.
class StepAnimator {
  /// Durée maximale comptée par image ; `null` supprime le plafond (tests qui avancent l'horloge d'un coup).
  static Duration? frameCap = const Duration(milliseconds: 34);

  final void Function(double value) onValue;
  late final Ticker _ticker;
  double value;
  double _from = 0, _to = 0;
  Duration _length = Duration.zero, _spent = Duration.zero;
  Duration _last = Duration.zero;
  Curve _curve = Curves.easeOutCubic;

  StepAnimator(TickerProvider vsync, {required this.onValue, this.value = 0}) {
    _ticker = vsync.createTicker(_tick);
  }

  /// Glisse jusqu'à [to] en [length] ; saute directement si [instant] (animations réduites).
  void animateTo(
    double to,
    Duration length, {
    Curve curve = Curves.easeOutCubic,
    bool instant = false,
  }) {
    if (instant || value == to) return jumpTo(to);
    _from = value;
    _to = to;
    _length = length;
    _curve = curve;
    _spent = Duration.zero;
    _last = Duration.zero;
    if (!_ticker.isActive) _ticker.start();
  }

  /// Fixe la valeur sans animation et arrête tout mouvement en cours.
  void jumpTo(double v) {
    _ticker.stop();
    value = v;
    onValue(v);
  }

  /// Arrête le mouvement en cours en gardant la valeur atteinte.
  void stop() => _ticker.stop();

  void dispose() => _ticker.dispose();

  void _tick(Duration elapsed) {
    final dt = elapsed - _last;
    _last = elapsed;
    final cap = frameCap;
    _spent += cap != null && dt > cap ? cap : dt;
    final progress = _spent.inMicroseconds / _length.inMicroseconds;
    if (progress >= 1) return jumpTo(_to);
    value = _from + (_to - _from) * _curve.transform(progress);
    onValue(value);
  }
}
