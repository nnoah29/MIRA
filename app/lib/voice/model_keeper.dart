//! Garde du modèle en mémoire : il reste chargé un moment après la dernière dictée, puis il est rendu.

import 'dart:async';

/// Décide quand rendre le modèle de reconnaissance : après [keepFor] sans dictée, ou tout de suite si la durée est nulle.
class ModelKeeper {
  ModelKeeper({required this.keepFor, required this.release});

  Duration keepFor;
  final void Function() release;
  Timer? _timer;

  /// Une dictée commence : l'échéance de libération est annulée.
  void used() {
    _timer?.cancel();
    _timer = null;
  }

  /// Une dictée vient de finir : le modèle sera rendu dans [keepFor], ou maintenant si la durée est nulle.
  void idle() {
    used();
    if (keepFor <= Duration.zero) return release();
    _timer = Timer(keepFor, release);
  }

  /// Abandonne l'échéance sans rien rendre.
  void dispose() => used();
}

/// Réglage de la durée de garde du modèle, pour un reconnaisseur qui le garde en mémoire.
abstract interface class KeepWarmSetting {
  set keepWarm(Duration keep);
}
