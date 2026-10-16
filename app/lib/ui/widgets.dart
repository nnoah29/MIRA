import 'package:flutter/material.dart';

import '../data/format.dart';
import '../data/models.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';

/// Zone interactive avec survol (équivalent d'un `button:hover`).
class Hov extends StatefulWidget {
  final Widget Function(BuildContext context, bool hover) builder;
  final VoidCallback? onTap;
  final String? tip;
  final bool opaque;

  /// Léger effet « bouton enfoncé » pendant l'appui.
  final bool press;
  const Hov({
    super.key,
    required this.builder,
    this.onTap,
    this.tip,
    this.opaque = true,
    this.press = false,
  });

  @override
  State<Hov> createState() => _HovState();
}

class _HovState extends State<Hov> {
  bool h = false;
  bool down = false;
  @override
  Widget build(BuildContext context) {
    Widget content = widget.builder(context, h);
    if (widget.press) {
      content = AnimatedScale(
        scale: down ? .94 : 1,
        duration: const Duration(milliseconds: 90),
        curve: Curves.easeOut,
        child: content,
      );
    }
    Widget w = MouseRegion(
      cursor: widget.onTap != null
          ? SystemMouseCursors.click
          : MouseCursor.defer,
      onEnter: (_) => setState(() => h = true),
      onExit: (_) => setState(() => h = false),
      child: GestureDetector(
        behavior: widget.opaque
            ? HitTestBehavior.opaque
            : HitTestBehavior.deferToChild,
        onTap: widget.onTap,
        onTapDown: widget.press ? (_) => setState(() => down = true) : null,
        onTapUp: widget.press ? (_) => setState(() => down = false) : null,
        onTapCancel: widget.press ? () => setState(() => down = false) : null,
        child: content,
      ),
    );
    if (widget.tip != null) {
      w = Tooltip(
        message: widget.tip!,
        waitDuration: const Duration(milliseconds: 700),
        child: w,
      );
    }
    return w;
  }
}

/// Texte « .mono » : capitales, interlettrage large.
class Mono extends StatelessWidget {
  final String text;
  final double size;
  final FontWeight w;
  final Color? color;
  final double ls;
  final double? height;
  final int? maxLines;
  final bool upper;
  final TextAlign? align;
  const Mono(
    this.text, {
    super.key,
    this.size = 11,
    this.w = FontWeight.w400,
    this.color,
    this.ls = .16,
    this.height = 1.5,
    this.maxLines,
    this.upper = true,
    this.align,
  });

  @override
  Widget build(BuildContext context) => Text(
    upper ? text.toUpperCase() : text,
    maxLines: maxLines,
    overflow: maxLines != null ? TextOverflow.ellipsis : null,
    textAlign: align,
    style: Fonts.mono(
      size: size,
      w: w,
      color: color ?? context.pal.dim,
      ls: ls,
      height: height,
    ),
  );
}

/// Bouton `.btn` (contour, étiquette mono).
class Btn extends StatelessWidget {
  final String? icon;
  final String label;
  final VoidCallback? onTap;
  final bool go, on, iconOnly;
  final EdgeInsets? padding;
  final double? fontSize;
  final String? tip;
  final double? width;
  const Btn({
    super.key,
    this.icon,
    required this.label,
    this.onTap,
    this.go = false,
    this.on = false,
    this.iconOnly = false,
    this.padding,
    this.fontSize,
    this.tip,
    this.width,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      tip: tip,
      press: true,
      builder: (_, h) {
        Color fg = go ? p.onAccent : (on ? p.accent : (h ? p.text : p.muted));
        Color bg = go
            ? p.accent
            : (on ? p.accentSoft : (h ? p.surface : Colors.transparent));
        Color bd = go ? p.accent : (on ? p.accent : (h ? p.dim : p.line));
        Widget child = Row(
          mainAxisSize: MainAxisSize.min,
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            if (icon != null) Ico(icon!, color: fg),
            if (icon != null && !iconOnly) const SizedBox(width: 10),
            if (!iconOnly)
              Mono(
                label,
                size: fontSize ?? 11,
                color: fg,
                w: go ? FontWeight.w700 : FontWeight.w400,
                height: null,
              ),
          ],
        );
        child = AnimatedContainer(
          duration: const Duration(milliseconds: 130),
          padding:
              padding ??
              (iconOnly
                  ? const EdgeInsets.symmetric(vertical: 10)
                  : const EdgeInsets.symmetric(horizontal: 16, vertical: 10)),
          width: iconOnly ? (width ?? 38) : null,
          decoration: BoxDecoration(
            color: bg,
            border: Border.all(color: bd),
          ),
          child: child,
        );
        if (go && h) {
          child = ColorFiltered(
            colorFilter: const ColorFilter.matrix(_bright),
            child: child,
          );
        }
        return child;
      },
    );
  }
}

const _bright = <double>[
  1.1,
  0,
  0,
  0,
  0,
  0,
  1.1,
  0,
  0,
  0,
  0,
  0,
  1.1,
  0,
  0,
  0,
  0,
  0,
  1,
  0,
];

/// Bouton icône `.ibtn` (34×34).
class IBtn extends StatelessWidget {
  final String icon;
  final VoidCallback? onTap;
  final String? tip;
  final bool on, filled;
  final double size, box;
  final double stroke;
  final Color? color;
  const IBtn(
    this.icon, {
    super.key,
    this.onTap,
    this.tip,
    this.on = false,
    this.filled = false,
    this.size = 16,
    this.box = 34,
    this.stroke = 1.6,
    this.color,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      tip: tip,
      press: true,
      builder: (_, h) => AnimatedContainer(
        duration: const Duration(milliseconds: 130),
        width: box,
        height: box,
        alignment: Alignment.center,
        color: h ? p.surface : Colors.transparent,
        child: Ico(
          icon,
          size: size,
          stroke: stroke,
          filled: filled,
          color: color ?? (on ? p.accent : (h ? p.text : p.muted)),
        ),
      ),
    );
  }
}

class Avatar extends StatelessWidget {
  final String name;
  final double size, font;
  const Avatar(this.name, {super.key, this.size = 38, this.font = 11});

  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    alignment: Alignment.center,
    color: Color(colorOf(name)),
    child: Text(
      initials(name),
      style: Fonts.mono(
        size: font,
        w: FontWeight.w700,
        color: const Color(0xFF1b1b1b),
        ls: 0,
        height: 1,
      ),
    ),
  );
}

class Kbd extends StatelessWidget {
  final String k;
  const Kbd(this.k, {super.key});
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
      decoration: BoxDecoration(
        color: p.surface,
        border: Border.all(color: p.line),
      ),
      child: Text(
        k,
        style: Fonts.mono(
          size: 11,
          w: FontWeight.w500,
          color: p.muted,
          ls: 0,
          height: 1.5,
        ),
      ),
    );
  }
}

/// Bordure en tirets (`border: 1px dashed`).
class DashedBox extends StatelessWidget {
  final Widget child;
  final Color color;
  final Color? fill;
  final EdgeInsets padding;
  const DashedBox({
    super.key,
    required this.child,
    required this.color,
    this.fill,
    this.padding = EdgeInsets.zero,
  });

  @override
  Widget build(BuildContext context) => CustomPaint(
    foregroundPainter: _DashPainter(color),
    child: Container(color: fill, padding: padding, child: child),
  );
}

class _DashPainter extends CustomPainter {
  final Color color;
  _DashPainter(this.color);
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;
    final path = Path()
      ..addRect(Rect.fromLTWH(.5, .5, size.width - 1, size.height - 1));
    for (final m in path.computeMetrics()) {
      var d = 0.0;
      while (d < m.length) {
        canvas.drawPath(m.extractPath(d, d + 3), paint);
        d += 6;
      }
    }
  }

  @override
  bool shouldRepaint(_DashPainter o) => o.color != color;
}

/// Étiquette carrée de couleur (libellés).
class Square extends StatelessWidget {
  final double size;
  final Color color;
  const Square(this.size, this.color, {super.key});
  @override
  Widget build(BuildContext context) =>
      Container(width: size, height: size, color: color);
}

// ---- rendu des blocs de message ----

TextSpan spanOf(Span s, TextStyle base, Pal p) {
  var st = base;
  if (s.bold) {
    st = Fonts.sansWeight(st, FontWeight.w500).copyWith(color: p.text);
  }
  if (s.italic) st = st.copyWith(fontStyle: FontStyle.italic);
  if (s.underline) st = st.copyWith(decoration: TextDecoration.underline);
  if (s.at || s.link) st = st.copyWith(color: p.accent);
  if (s.link) {
    st = st.copyWith(
      decoration: TextDecoration.underline,
      decorationColor: p.accent,
    );
  }
  if (s.code) {
    st = st.copyWith(
      fontFamily: 'monospace',
      fontSize: (base.fontSize ?? 16) * .85,
    );
  }
  return TextSpan(text: s.text, style: st);
}

Widget paraWidget(Para b, TextStyle base, Pal p) {
  var spans = b.spans;
  if (spans.isNotEmpty && spans.last.text == '\n') {
    spans = spans.sublist(0, spans.length - 1);
  }
  if (spans.isEmpty) return Text(' ', style: base);
  return Text.rich(
    TextSpan(
      style: base,
      children: spans.map((s) => spanOf(s, base, p)).toList(),
    ),
  );
}

Widget blocksView(
  List<Block> bs,
  TextStyle base,
  Pal p, {
  double gap = 14,
  double? maxW,
}) {
  return Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    mainAxisSize: MainAxisSize.min,
    children: [
      for (final b in bs)
        switch (b) {
          Para() => Padding(
            padding: EdgeInsets.only(bottom: b.loose ? 0 : gap),
            child: maxW == null
                ? paraWidget(b, base, p)
                : ConstrainedBox(
                    constraints: BoxConstraints(maxWidth: maxW),
                    child: paraWidget(b, base, p),
                  ),
          ),
          Quote() => Container(
            width: double.infinity,
            margin: const EdgeInsets.symmetric(vertical: 14),
            padding: const EdgeInsets.only(left: 14),
            decoration: BoxDecoration(
              border: Border(left: BorderSide(color: p.accent, width: 2)),
            ),
            child: blocksView(
              b.children,
              base.copyWith(color: p.dim),
              p,
              gap: gap,
              maxW: maxW,
            ),
          ),
        },
    ],
  );
}

/// Apparition douce (fondu + léger glissement), jouée une fois à la création du widget.
/// Respecte le réglage système « réduire les animations ».
class FadeSlideIn extends StatefulWidget {
  final Widget child;
  final Duration delay;
  final Duration duration;
  final Offset from; // décalage de départ, en pixels
  const FadeSlideIn({
    super.key,
    required this.child,
    this.delay = Duration.zero,
    this.duration = const Duration(milliseconds: 260),
    this.from = const Offset(0, 8),
  });

  @override
  State<FadeSlideIn> createState() => _FadeSlideInState();
}

class _FadeSlideInState extends State<FadeSlideIn>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(
    vsync: this,
    duration: widget.delay + widget.duration,
  );
  late final Animation<double> _t = CurvedAnimation(
    parent: _c,
    curve: Interval(
      widget.delay.inMilliseconds /
          (widget.delay + widget.duration).inMilliseconds,
      1,
      curve: Curves.easeOutCubic,
    ),
  );
  bool _started = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (!_started) {
      _started = true;
      if (MediaQuery.maybeDisableAnimationsOf(context) ?? false) {
        _c.value = 1;
      } else {
        _c.forward();
      }
    }
  }

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: _t,
    builder: (_, child) => Opacity(
      opacity: _t.value,
      child: Transform.translate(
        offset: Offset(
          widget.from.dx * (1 - _t.value),
          widget.from.dy * (1 - _t.value),
        ),
        child: child,
      ),
    ),
    child: widget.child,
  );
}

/// Petit « rebond » quand [trigger] change (étoile, coche…).
class Bounce extends StatefulWidget {
  final Object? trigger;
  final Widget child;
  const Bounce({super.key, required this.trigger, required this.child});
  @override
  State<Bounce> createState() => _BounceState();
}

class _BounceState extends State<Bounce> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 420),
    value: 1,
  );

  @override
  void didUpdateWidget(Bounce old) {
    super.didUpdateWidget(old);
    if (old.trigger != widget.trigger) _c.forward(from: 0);
  }

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: _c,
    builder: (_, child) => Transform.scale(
      scale: .55 + .45 * Curves.elasticOut.transform(_c.value),
      child: child,
    ),
    child: widget.child,
  );
}

/// Chiffre qui change avec un petit effet de zoom + fondu (compteurs).
class AnimatedCount extends StatelessWidget {
  final int value;
  final TextStyle style;
  const AnimatedCount(this.value, {super.key, required this.style});

  @override
  Widget build(BuildContext context) => AnimatedSwitcher(
    duration: const Duration(milliseconds: 220),
    switchInCurve: Curves.easeOutBack,
    transitionBuilder: (child, a) => FadeTransition(
      opacity: a,
      child: ScaleTransition(
        scale: Tween(begin: .6, end: 1.0).animate(a),
        child: child,
      ),
    ),
    child: Text('$value', key: ValueKey(value), style: style),
  );
}

/// Zone défilante verticale avec barre de défilement (contrôleur dédié : pas de dépendance
/// au `PrimaryScrollController`, qui n'est pas attaché sur ordinateur).
class ScrollArea extends StatefulWidget {
  final EdgeInsetsGeometry? padding;
  final Widget child;
  const ScrollArea({super.key, this.padding, required this.child});
  @override
  State<ScrollArea> createState() => _ScrollAreaState();
}

class _ScrollAreaState extends State<ScrollArea> {
  final ctl = ScrollController();

  @override
  void dispose() {
    ctl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Scrollbar(
    controller: ctl,
    child: SingleChildScrollView(
      controller: ctl,
      padding: widget.padding,
      child: widget.child,
    ),
  );
}
