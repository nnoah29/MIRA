import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';

import 'wall_theme.dart';

/// Le thème du fond d'écran n'existe que sous Linux (ShellArie / Quickshell).
bool get wallSupported => Platform.isLinux;

String get _jsonPath =>
    Platform.environment['QS_COLORS_FILE'] ?? '/tmp/qs_colors.json';
String get _cssPath =>
    '${Platform.environment['HOME'] ?? ''}/.cache/theme-color/colors.css';

Color? _hex(Object? v) {
  if (v is! String) return null;
  final m = RegExp(r'^#?([0-9a-fA-F]{6})').firstMatch(v.trim());
  return m == null
      ? null
      : Color(0xFF000000 | int.parse(m.group(1)!, radix: 16));
}

Future<WallSeed?> _read() async {
  try {
    final j = jsonDecode(await File(_jsonPath).readAsString());
    if (j is! Map) return null;
    final bg = _hex(j['base']), fg = _hex(j['text']);
    if (bg == null || fg == null) return null;

    // Teintes de la palette du fond d'écran (pour une vraie couleur d'accent).
    final cands = <Color>[];
    try {
      final css = await File(_cssPath).readAsString();
      for (final m in RegExp(
        r'--color(\d+):\s*#([0-9a-fA-F]{6})',
      ).allMatches(css)) {
        final n = int.parse(m.group(1)!);
        if (n >= 1 && n <= 14) {
          cands.add(Color(0xFF000000 | int.parse(m.group(2)!, radix: 16)));
        }
      }
    } catch (_) {
      /* palette indisponible : on retombe sur le JSON */
    }
    if (cands.isEmpty) {
      for (final k in [
        'secondary',
        'tertiary',
        'materialBlue',
        'materialGreen',
        'materialOrange',
        'materialRed',
        'materialYellow',
      ]) {
        final c = _hex(j[k]);
        if (c != null) cands.add(c);
      }
    }
    return WallSeed(bg, fg, cands);
  } catch (_) {
    return null;
  }
}

/// Valeur actuelle puis une nouvelle valeur à chaque réécriture du fichier de couleurs.
Stream<WallSeed?> watchWallSeed() {
  late StreamController<WallSeed?> ctl;
  StreamSubscription<FileSystemEvent>? sub;
  Timer? debounce;
  var last = const Object();

  Future<void> emit() async {
    final s = await _read();
    if (ctl.isClosed || s == last) return;
    last = s ?? const Object();
    ctl.add(s);
  }

  ctl = StreamController<WallSeed?>(
    onListen: () async {
      await emit();
      final file = File(_jsonPath);
      try {
        // Surveille le dossier (le fichier est recréé à chaque génération).
        sub = file.parent.watch().listen((e) {
          if (e.path.endsWith(file.uri.pathSegments.last)) {
            debounce?.cancel();
            debounce = Timer(const Duration(milliseconds: 150), emit);
          }
        }, onError: (_) {});
      } catch (_) {
        /* surveillance impossible */
      }
    },
    onCancel: () {
      debounce?.cancel();
      sub?.cancel();
    },
  );
  return ctl.stream;
}
