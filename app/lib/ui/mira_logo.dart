import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

/// Logo MIRA : cadre ouvert en dégradé de la couleur principale et « M » centré.
class MiraLogo extends StatelessWidget {
  final double size;
  final Color color;
  const MiraLogo({super.key, required this.color, this.size = 32});

  @override
  Widget build(BuildContext context) => SizedBox.square(
    dimension: size,
    child: CustomPaint(painter: _MiraLogoPainter(color)),
  );
}

class _MiraLogoPainter extends CustomPainter {
  final Color color;
  _MiraLogoPainter(this.color);

  /// Éclaircit ou assombrit la couleur principale pour les extrémités du dégradé.
  Color _shade(double delta) {
    final h = HSLColor.fromColor(color);
    return h.withLightness((h.lightness + delta).clamp(0.12, 0.88)).toColor();
  }

  @override
  void paint(Canvas canvas, Size size) {
    final k = size.width / 200;
    canvas.scale(k);
    final frame = Path()
      ..moveTo(30, 118)
      ..lineTo(30, 30)
      ..lineTo(170, 30)
      ..lineTo(170, 170)
      ..lineTo(82, 170);
    final shader = LinearGradient(
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
      colors: [_shade(.15), color, _shade(-.14)],
      stops: const [0, .55, 1],
    ).createShader(const Rect.fromLTWH(26, 30, 148, 140));
    final stroke = (7 * k).clamp(1.4, 40) / k;
    canvas.drawPath(
      frame,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = stroke
        ..strokeJoin = StrokeJoin.miter
        ..shader = shader,
    );
    _paintLetter(canvas);
  }

  /// Dessine le « M » en Anton au centre du cadre.
  void _paintLetter(Canvas canvas) {
    final tp = TextPainter(
      text: TextSpan(
        text: 'M',
        style: GoogleFonts.anton(fontSize: 74, color: color, height: 1),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(canvas, Offset(100 - tp.width / 2, 100 - tp.height / 2 + 2));
  }

  @override
  bool shouldRepaint(_MiraLogoPainter old) => old.color != color;
}
