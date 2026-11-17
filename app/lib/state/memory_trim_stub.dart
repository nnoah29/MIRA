//! Plateformes sans accès au système (web) : rien à rendre.

import 'memory_trimmer.dart';

/// Libérateur de la plateforme ; sans effet ici.
MemoryTrimmer systemMemoryTrimmer() => const _NoTrim();

class _NoTrim implements MemoryTrimmer {
  const _NoTrim();

  @override
  void trim() {}
}
