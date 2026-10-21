import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/wheel_fling.dart';

/// Lecteur haut de 400 px sur 5 000 px de contenu ; rend sa position de défilement.
Future<ScrollPosition> _reader(WidgetTester tester) async {
  final controller = ScrollController();
  addTearDown(controller.dispose);
  await tester.pumpWidget(
    MaterialApp(
      home: SizedBox(
        height: 400,
        child: SingleChildScrollView(
          controller: controller,
          child: const SizedBox(height: 5000),
        ),
      ),
    ),
  );
  return controller.position;
}

/// Envoie des événements `(heure dans la page, déplacement)`, puis laisse le geste finir et l'élan s'éteindre.
Future<void> _gesture(
  WidgetTester tester,
  ScrollPosition position,
  List<(double, double)> events, {
  required bool precise,
}) async {
  final wheel = WheelFling();
  for (final (at, delta) in events) {
    wheel.scroll(position, delta, precise: precise, at: at);
  }
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pumpAndSettle();
  wheel.dispose();
}

final _steadySwipe = [for (var i = 0; i < 10; i++) (i * 10.0, 20.0)];

void main() {
  testWidgets('un glissement au pavé tactile se prolonge par un élan', (
    tester,
  ) async {
    final position = await _reader(tester);
    await _gesture(tester, position, _steadySwipe, precise: true);
    expect(position.pixels, greaterThan(200 + 50));
  });

  testWidgets('un petit coup rapide (deux événements) lance aussi un élan', (
    tester,
  ) async {
    final position = await _reader(tester);
    await _gesture(tester, position, [(0.0, 72.0), (4.0, 76.0)], precise: true);
    expect(position.pixels, greaterThan(148 + 100));
  });

  testWidgets(
    'des événements reçus en paquet ne donnent pas un élan démesuré',
    (tester) async {
      final position = await _reader(tester);
      await _gesture(tester, position, [
        for (var i = 0; i < 5; i++) (0.0, 30.0),
      ], precise: true);
      expect(position.pixels, lessThan(4600));
      expect(position.pixels, greaterThan(150));
    },
  );

  testWidgets('des crans de molette ne lancent pas d’élan', (tester) async {
    final position = await _reader(tester);
    await _gesture(tester, position, _steadySwipe, precise: false);
    expect(position.pixels, 200);
  });
}
