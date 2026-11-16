//! Libération de la mémoire sous Linux : `malloc_trim(0)` de la bibliothèque C, sans effet ailleurs.

import 'dart:ffi';
import 'dart:io';

import 'memory_trimmer.dart';

/// Libérateur de la plateforme : `malloc_trim` sous Linux, rien ailleurs.
MemoryTrimmer systemMemoryTrimmer() =>
    Platform.isLinux ? _LinuxTrim._open() : const _NoTrim();

class _NoTrim implements MemoryTrimmer {
  const _NoTrim();

  @override
  void trim() {}
}

class _LinuxTrim implements MemoryTrimmer {
  _LinuxTrim(this._malloc);

  final int Function(int) _malloc;

  /// Cherche `malloc_trim` dans le processus ; sans lui, le libérateur est inerte.
  static MemoryTrimmer _open() {
    try {
      final f = DynamicLibrary.process()
          .lookupFunction<IntPtr Function(IntPtr), int Function(int)>(
            'malloc_trim',
          );
      return _LinuxTrim(f);
    } on Object {
      return const _NoTrim();
    }
  }

  @override
  void trim() => _malloc(0);
}
