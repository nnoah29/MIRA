//! Interface de libération mémoire : rendre au système ce que l'application a libéré sans qu'il soit retourné.

export 'memory_trim_stub.dart'
    if (dart.library.io) 'memory_trim_io.dart'
    show systemMemoryTrimmer;

/// Rend au système la mémoire libre que l'allocateur a gardée.
abstract interface class MemoryTrimmer {
  void trim();
}
