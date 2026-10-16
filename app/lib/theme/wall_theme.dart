import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'app_theme.dart';

/// Couleurs de départ tirées du fond d'écran (via ShellArie : `/tmp/qs_colors.json`).
class WallSeed {
  final Color bg, fg;

  /// Teintes candidates pour l'accent (palette du fond d'écran, `color1..color14`).
  final List<Color> candidates;
  const WallSeed(this.bg, this.fg, this.candidates);

  @override
  bool operator ==(Object other) =>
      other is WallSeed &&
      other.bg == bg &&
      other.fg == fg &&
      _same(other.candidates, candidates);

  @override
  int get hashCode => Object.hash(bg, fg, candidates.length);

  static bool _same(List<Color> a, List<Color> b) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }
}

double contrast(Color a, Color b) {
  final la = a.computeLuminance(), lb = b.computeLuminance();
  return (math.max(la, lb) + .05) / (math.min(la, lb) + .05);
}

Color mix(Color a, Color b, double t) => Color.lerp(a, b, t)!;

/// Éclaircit / assombrit [c] (vers blanc ou noir selon le fond) jusqu'à atteindre le contraste [min].
Color ensureContrast(Color c, Color bg, double min) {
  if (contrast(c, bg) >= min) return c;
  final target = bg.computeLuminance() < .4 ? Colors.white : Colors.black;
  var out = c;
  for (var i = 1; i <= 20 && contrast(out, bg) < min; i++) {
    out = mix(c, target, i / 20);
  }
  return out;
}

/// Choisit l'accent : la teinte la plus « vivante » du fond d'écran, rendue lisible sur [bg].
/// Si le fond d'écran est presque gris, on garde la couleur du texte.
Color pickAccent(
  List<Color> candidates,
  Color bg,
  Color fg, {
  required bool dark,
}) {
  HSLColor? best;
  var bestScore = 0.0;
  for (final c in candidates) {
    final h = HSLColor.fromColor(c);
    // Saturation d'abord ; on évite les teintes quasi blanches ou noires.
    final score =
        h.saturation * (1 - (h.lightness - .5).abs() * 1.2).clamp(.1, 1);
    if (score > bestScore) {
      bestScore = score;
      best = h;
    }
  }
  if (best == null || best.saturation < .06) return fg;
  final h = best.withSaturation(best.saturation.clamp(.52, .86));
  final tuned = h.withLightness(
    dark ? h.lightness.clamp(.56, .68) : h.lightness.clamp(.36, .46),
  );
  return ensureContrast(tuned.toColor(), bg, 3.2);
}

/// Palette complète de MIRA à partir de trois couleurs, avec les mêmes ratios que ShellArie
/// (surfaces = fond + 3 / 6 / 9 % de texte, contours 10 / 16 %…).
({Pal pal, bool dark}) palFromSeed(WallSeed s) {
  final dark = s.bg.computeLuminance() < .4;
  final bg = s.bg;
  final text = ensureContrast(s.fg, bg, 10);
  final muted = ensureContrast(mix(text, bg, .25), bg, 6.5);
  final dim = ensureContrast(mix(text, bg, .45), bg, 4.5);
  final accent = pickAccent(s.candidates, bg, text, dark: dark);
  final onAccent =
      contrast(accent, const Color(0xFF111111)) >=
          contrast(accent, Colors.white)
      ? const Color(0xFF111111)
      : Colors.white;
  final hsl = HSLColor.fromColor(accent);
  final toastAccent = dark
      ? hsl.withLightness((hsl.lightness * .72).clamp(0, 1)).toColor()
      : hsl.withLightness((hsl.lightness + .22).clamp(0, .85)).toColor();
  final pal = Pal(
    bg: bg,
    bgSide: mix(bg, s.fg, .03),
    surface: mix(bg, s.fg, .06),
    surface2: mix(bg, s.fg, .09),
    line: mix(bg, s.fg, .16),
    lineSoft: mix(bg, s.fg, .10),
    text: text,
    muted: muted,
    dim: dim,
    accent: accent,
    accentSoft: accent.withValues(alpha: dark ? .12 : .10),
    onAccent: onAccent,
    toastBg: text,
    toastFg: bg,
    toastAccent: toastAccent,
    body: mix(text, bg, .1),
  );
  return (pal: pal, dark: dark);
}
