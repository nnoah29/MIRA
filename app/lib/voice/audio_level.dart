//! Niveau sonore d'un paquet d'échantillons, de 0 (silence) à 1 (voix forte), pour l'animation du micro.

import 'dart:math' as math;
import 'dart:typed_data';

const _floorDb = -50.0;
const _rangeDb = 40.0;

/// Niveau perçu d'un paquet : valeur efficace en décibels ramenée sur l'échelle de 0 à 1.
double audioLevel(Float32List samples) {
  if (samples.isEmpty) return 0;
  var sum = 0.0;
  for (final s in samples) {
    sum += s * s;
  }
  final rms = math.sqrt(sum / samples.length);
  if (rms < 1e-5) return 0;
  final db = 20 * math.log(rms) / math.ln10;
  return ((db - _floorDb) / _rangeDb).clamp(0.0, 1.0);
}
