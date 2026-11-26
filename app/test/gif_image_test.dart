import 'dart:convert';
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/gif_playback.dart';
import 'package:mira/ui/gif_image.dart';

final _gif = base64Decode(
  'R0lGODlhBAAEAIEAAP8AAAAAAAAAAAAAACH/C05FVFNDQVBFMi4wAwEAAAAh+QQACgAAACwAAAAABAAEAAAICQABCBxIsCCAgAAh+QQBCgABACwAAAAABAAEAIEAAP8AAAAAAAAAAAAICQABCBxIsCCAgAA7',
);

Widget _host(GifPlayback mode) => MaterialApp(
  home: Center(
    child: GifImage(
      image: MemoryImage(_gif),
      playback: mode,
      width: 40,
      height: 40,
    ),
  ),
);

Future<void> _wait(WidgetTester tester, [int ms = 130]) async {
  await tester.runAsync(() => Future<void>.delayed(Duration(milliseconds: ms)));
  await tester.pump(Duration(milliseconds: ms));
}

Future<List<ui.Image>> _frames(WidgetTester tester, int count) async {
  final seen = <ui.Image>[];
  for (var i = 0; i < count; i++) {
    await _wait(tester);
    seen.add(tester.widget<RawImage>(find.byType(RawImage)).image!);
  }
  return seen;
}

int _distinct(List<ui.Image> images) =>
    {for (final image in images) identityHashCode(image)}.length;

void main() {
  test('un GIF est reconnu à ses octets ou à l\'extension de son adresse', () {
    expect(isGifSource(MemoryImage(_gif)), isTrue);
    expect(isGifSource(MemoryImage(base64Decode('iVBORw0KGgo='))), isFalse);
    expect(
      isGifSource(const NetworkImage('https://x.org/a/logo.GIF?w=2')),
      isTrue,
    );
    expect(
      isGifSource(const NetworkImage('https://x.org/a/logo.png')),
      isFalse,
    );
    expect(
      isGifSource(const NetworkImage('https://x.org/gif/logo.png')),
      isFalse,
    );
  });

  testWidgets(
    'figée : la première image reste affichée, aucune autre n\'est demandée',
    (tester) async {
      await tester.pumpWidget(_host(GifPlayback.off));
      await _wait(tester, 200);
      expect(find.byType(RawImage), findsOneWidget);
      expect(_distinct(await _frames(tester, 8)), 1);
      await tester.pumpAndSettle();
    },
  );

  testWidgets('toujours : le GIF s\'anime', (tester) async {
    await tester.pumpWidget(_host(GifPlayback.always));
    await _wait(tester, 200);
    expect(_distinct(await _frames(tester, 12)), greaterThan(1));
  });

  testWidgets(
    'au survol : figé au repos, animé sous la souris, figé de nouveau en la quittant',
    (tester) async {
      await tester.pumpWidget(_host(GifPlayback.hover));
      await _wait(tester, 200);
      expect(_distinct(await _frames(tester, 6)), 1);

      final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: const Offset(-50, -50));
      addTearDown(mouse.removePointer);
      await mouse.moveTo(tester.getCenter(find.byType(GifImage)));
      await tester.pump();
      expect(_distinct(await _frames(tester, 12)), greaterThan(1));

      await mouse.moveTo(const Offset(-50, -50));
      await tester.pump();
      await _wait(tester, 300);
      expect(_distinct(await _frames(tester, 8)), 1);
    },
  );

  testWidgets('une image illisible affiche le contenu de repli', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: GifImage(
          image: MemoryImage(base64Decode('AAAA')),
          playback: GifPlayback.off,
          errorBuilder: (_, _, _) => const Text('repli'),
        ),
      ),
    );
    await _wait(tester, 200);
    expect(find.text('repli'), findsOneWidget);
  });
}
