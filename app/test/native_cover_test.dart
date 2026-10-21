import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/native_cover.dart';
import 'package:mira/ui/overlays.dart';

/// Ce qu'une vue « native » a vécu : ses états « couverte », et les superpositions qui l'ont chevauchée.
class _Seen {
  final covered = <bool>[];
  var overlapped = 0;
}

/// Écran de 800 × 600 : un bouton en haut à gauche, une carte à `area` dont la vue « native » occupe le bas à partir de `nativeTop`,
/// affichée ou non (`onScreen`).
Future<(BuildContext, _Seen)> _screen(
  WidgetTester tester,
  Rect area, {
  bool onScreen = true,
  double nativeTop = 0,
}) async {
  late BuildContext anchor;
  final seen = _Seen();
  await tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(true),
      home: Stack(
        children: [
          Positioned(
            left: 0,
            top: 0,
            width: 40,
            height: 40,
            child: Builder(
              builder: (ctx) {
                anchor = ctx;
                return const SizedBox.expand();
              },
            ),
          ),
          Positioned.fromRect(
            rect: area,
            child: TickerMode(
              enabled: onScreen,
              child: NativeCoverBuilder(
                onOverlapped: () => seen.overlapped++,
                builder: (_, covered) {
                  seen.covered.add(covered);
                  return Padding(
                    padding: EdgeInsets.only(top: nativeTop),
                    child: const NativeViewArea(child: SizedBox.expand()),
                  );
                },
              ),
            ),
          ),
        ],
      ),
    ),
  );
  return (anchor, seen);
}

Future<void> _settle(WidgetTester tester) async {
  await tester.pump();
  await tester.pump();
}

/// Ouvre puis ferme un popover de 100 × 200 sous le bouton.
Future<void> _popover(WidgetTester tester, BuildContext anchor) async {
  Pops.show(anchor, 'test', (_, _) => const SizedBox(width: 100, height: 200));
  await _settle(tester);
  Pops.close();
  await _settle(tester);
}

void main() {
  testWidgets('un popover qui chevauche la vue la signale une seule fois', (
    tester,
  ) async {
    final (anchor, seen) = await _screen(
      tester,
      const Rect.fromLTWH(0, 50, 800, 550),
    );
    await _popover(tester, anchor);
    expect(seen.overlapped, 1);
    expect(seen.covered.last, isFalse);
  });

  testWidgets('un popover à l\'écart de la vue ne la signale pas', (
    tester,
  ) async {
    final (anchor, seen) = await _screen(
      tester,
      const Rect.fromLTWH(400, 300, 400, 300),
    );
    await _popover(tester, anchor);
    expect(seen.overlapped, 0);
  });

  testWidgets(
    'un popover qui ne touche que l\'en-tête de la carte la laisse dépliée',
    (tester) async {
      final (anchor, seen) = await _screen(
        tester,
        const Rect.fromLTWH(0, 50, 800, 550),
        nativeTop: 300,
      );
      await _popover(tester, anchor);
      expect(seen.overlapped, 0);
    },
  );

  testWidgets('une vue gardée hors écran ignore les popovers', (tester) async {
    final (anchor, seen) = await _screen(
      tester,
      const Rect.fromLTWH(0, 50, 800, 550),
      onScreen: false,
    );
    await _popover(tester, anchor);
    expect(seen.overlapped, 0);
  });

  testWidgets('une boîte de dialogue masque la vue le temps d\'être ouverte', (
    tester,
  ) async {
    final (anchor, seen) = await _screen(
      tester,
      const Rect.fromLTWH(400, 300, 400, 300),
    );
    showDialog<void>(context: anchor, builder: (_) => const SizedBox());
    await _settle(tester);
    expect(seen.covered.last, isTrue);
    Navigator.of(anchor).pop();
    await tester.pumpAndSettle();
    expect(seen.covered.last, isFalse);
    expect(seen.overlapped, 0);
  });

  testWidgets(
    'une modale hors navigateur masque la vue le temps d\'être ouverte',
    (tester) async {
      final (anchor, seen) = await _screen(
        tester,
        const Rect.fromLTWH(400, 300, 400, 300),
      );
      final entry = OverlayEntry(
        builder: (_) =>
            const CoversNativeViews(modal: true, child: SizedBox.expand()),
      );
      Overlay.of(anchor).insert(entry);
      await _settle(tester);
      expect(seen.covered.last, isTrue);
      entry.remove();
      await _settle(tester);
      expect(seen.covered.last, isFalse);
      expect(seen.overlapped, 0);
    },
  );
}
