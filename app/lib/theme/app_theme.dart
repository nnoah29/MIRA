import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

/// Variables CSS d'origine (`:root` / `:root[data-theme="light"]`).
@immutable
class Pal extends ThemeExtension<Pal> {
  final Color bg, bgSide, surface, surface2, line, lineSoft, text, muted, dim;
  final Color accent, accentSoft, onAccent, toastBg, toastFg, toastAccent, body;
  const Pal({
    required this.bg,
    required this.bgSide,
    required this.surface,
    required this.surface2,
    required this.line,
    required this.lineSoft,
    required this.text,
    required this.muted,
    required this.dim,
    required this.accent,
    required this.accentSoft,
    required this.onAccent,
    required this.toastBg,
    required this.toastFg,
    required this.toastAccent,
    required this.body,
  });

  static const dark = Pal(
    bg: Color(0xFF1b1b1b),
    bgSide: Color(0xFF1e1e1e),
    surface: Color(0xFF242424),
    surface2: Color(0xFF2a2a2a),
    line: Color(0xFF333333),
    lineSoft: Color(0xFF2b2b2b),
    text: Color(0xFFecebe6),
    muted: Color(0xFFadaca5),
    dim: Color(0xFF8b8a84),
    accent: Color(0xFFe8542b),
    accentSoft: Color(0x1fe8542b),
    onAccent: Color(0xFF1b1b1b),
    toastBg: Color(0xFFecebe6),
    toastFg: Color(0xFF1b1b1b),
    toastAccent: Color(0xFFbb3510),
    body: Color(0xFFd6d5cd),
  );

  static const light = Pal(
    bg: Color(0xFFf3f1ea),
    bgSide: Color(0xFFeae7de),
    surface: Color(0xFFfbfaf6),
    surface2: Color(0xFFe3e0d6),
    line: Color(0xFFcdc9bd),
    lineSoft: Color(0xFFdcd8cc),
    text: Color(0xFF1d1c1a),
    muted: Color(0xFF4b4943),
    dim: Color(0xFF6b685f),
    accent: Color(0xFFd6431a),
    accentSoft: Color(0x1ad6431a),
    onAccent: Color(0xFFffffff),
    toastBg: Color(0xFF1d1c1a),
    toastFg: Color(0xFFf3f1ea),
    toastAccent: Color(0xFFff8a63),
    body: Color(0xFF34322e),
  );

  /// Même palette avec une autre couleur principale (dérivée pour le thème actif).
  Pal withAccent(Color a, {required bool dark}) {
    final isStock = identical(this, Pal.dark) || identical(this, Pal.light);
    final isOrangeDark = a.toARGB32() == Pal.dark.accent.toARGB32();
    final isOrangeLight = a.toARGB32() == Pal.light.accent.toARGB32();
    if (isStock && (isOrangeDark || isOrangeLight)) {
      return dark ? Pal.dark : Pal.light;
    }
    final hsl = HSLColor.fromColor(a);
    // Thème clair : teinte un peu plus soutenue pour garder le contraste sur fond clair.
    final acc = dark
        ? a
        : hsl.withLightness((hsl.lightness * .9).clamp(0, 1)).toColor();
    final h = HSLColor.fromColor(acc);
    final lum = acc.computeLuminance();
    final on = dark
        ? (lum < .1 ? const Color(0xFFffffff) : const Color(0xFF1b1b1b))
        : (lum > .55 ? const Color(0xFF1b1b1b) : const Color(0xFFffffff));
    final toast = dark
        ? h.withLightness((h.lightness * .74).clamp(0, 1)).toColor()
        : h.withLightness((h.lightness + .2).clamp(0, .85)).toColor();
    return Pal(
      bg: bg,
      bgSide: bgSide,
      surface: surface,
      surface2: surface2,
      line: line,
      lineSoft: lineSoft,
      text: text,
      muted: muted,
      dim: dim,
      accent: acc,
      accentSoft: acc.withValues(alpha: dark ? .12 : .10),
      onAccent: on,
      toastBg: toastBg,
      toastFg: toastFg,
      toastAccent: toast,
      body: body,
    );
  }

  @override
  Pal copyWith() => this;

  @override
  Pal lerp(ThemeExtension<Pal>? other, double t) =>
      t < .5 ? this : (other as Pal);
}

extension PalCtx on BuildContext {
  Pal get pal => Theme.of(this).extension<Pal>()!;
}

/// Polices : Hanken Grotesk (texte), Anton (titres), JetBrains Mono (étiquettes).
class Fonts {
  /// Charge toutes les polices embarquées avant le premier affichage (évite tout flash de police).
  static Future<void> preload() async {
    for (final w in [
      FontWeight.w400,
      FontWeight.w500,
      FontWeight.w600,
      FontWeight.w700,
    ]) {
      GoogleFonts.hankenGrotesk(fontWeight: w);
    }
    for (final w in [FontWeight.w400, FontWeight.w500, FontWeight.w700]) {
      GoogleFonts.jetBrainsMono(fontWeight: w);
    }
    GoogleFonts.anton();
    await GoogleFonts.pendingFonts();
  }

  /// Change la graisse d'un style Hanken Grotesk en choisissant la bonne police embarquée.
  static TextStyle sansWeight(TextStyle base, FontWeight w) =>
      GoogleFonts.hankenGrotesk(textStyle: base, fontWeight: w);

  /// Flutter arrondit chaque ligne à l'entier ; le CSS garde des hauteurs fractionnaires.
  /// On cale donc la hauteur de ligne sur l'entier le plus proche (0,5 → inférieur).
  static double? snap(double size, double? h) =>
      h == null ? null : ((size * h - 1e-4).round()) / size;

  static TextStyle sans({
    double size = 15,
    FontWeight w = FontWeight.w400,
    Color? color,
    double? height = 1.5,
    double? ls,
  }) => GoogleFonts.hankenGrotesk(
    fontSize: size,
    fontWeight: w,
    color: color,
    height: snap(size, height),
    letterSpacing: ls,
  ).copyWith(leadingDistribution: TextLeadingDistribution.even);

  static TextStyle display({
    double size = 34,
    Color? color,
    double height = 1,
    double ls = .01,
  }) => GoogleFonts.anton(
    fontSize: size,
    fontWeight: FontWeight.w400,
    color: color,
    height: snap(size, height),
    letterSpacing: size * ls,
  ).copyWith(leadingDistribution: TextLeadingDistribution.even);

  static TextStyle mono({
    double size = 11,
    FontWeight w = FontWeight.w400,
    Color? color,
    double ls = .16,
    double? height = 1.5,
  }) => GoogleFonts.jetBrainsMono(
    fontSize: size,
    fontWeight: w,
    color: color,
    letterSpacing: size * ls,
    height: snap(size, height),
  ).copyWith(leadingDistribution: TextLeadingDistribution.even);
}

ThemeData buildTheme(bool dark, {Color? accent, Pal? pal}) {
  final base = pal ?? (dark ? Pal.dark : Pal.light);
  final p = accent == null ? base : base.withAccent(accent, dark: dark);
  return ThemeData(
    brightness: dark ? Brightness.dark : Brightness.light,
    scaffoldBackgroundColor: p.bg,
    canvasColor: p.bg,
    splashFactory: NoSplash.splashFactory,
    highlightColor: Colors.transparent,
    hoverColor: Colors.transparent,
    useMaterial3: true,
    colorScheme: ColorScheme(
      brightness: dark ? Brightness.dark : Brightness.light,
      primary: p.accent,
      onPrimary: p.onAccent,
      secondary: p.accent,
      onSecondary: p.onAccent,
      error: p.accent,
      onError: p.onAccent,
      surface: p.bg,
      onSurface: p.text,
    ),
    textSelectionTheme: TextSelectionThemeData(
      cursorColor: p.accent,
      selectionColor: p.accent,
      selectionHandleColor: p.accent,
    ),
    scrollbarTheme: ScrollbarThemeData(
      thumbColor: WidgetStatePropertyAll(p.line),
      thickness: const WidgetStatePropertyAll(8),
      radius: Radius.zero,
      thumbVisibility: const WidgetStatePropertyAll(false),
    ),
    extensions: [p],
  );
}
