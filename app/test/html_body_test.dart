import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/body_skeleton.dart';
import 'package:mira/ui/html_body.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

final _pixel = base64Decode(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
);

Msg _msg(
  String html, {
  List<String> blocked = const [],
  Map<String, Uint8List>? inline,
}) => Msg(from: const Person('Bob', 'bob@x.org'), date: 0, body: const [])
  ..html = html
  ..blocked = blocked
  ..inline = inline ?? {};

Future<void> _show(WidgetTester tester, Msg m) async {
  SharedPreferences.setMockInitialValues({});
  final store = MailStore(await SharedPreferences.getInstance());
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(true),
        home: Scaffold(
          body: SingleChildScrollView(
            child: MailHtml(msg: m, base: const TextStyle(fontSize: 16)),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 300)),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 300));
}

void main() {
  testWidgets('tableau, image data: et image cid: sont rendus', (tester) async {
    final data = 'data:image/png;base64,${base64Encode(_pixel)}';
    await _show(
      tester,
      _msg(
        '<table><tr><td>Cellule A</td><td>Cellule B</td></tr></table>'
        '<img src="$data"><img src="cid:logo">',
        inline: {'logo': Uint8List.fromList(_pixel)},
      ),
    );
    expect(
      find.textContaining('Cellule A', findRichText: true),
      findsOneWidget,
    );
    expect(find.byType(Image), findsNWidgets(2));
  });

  testWidgets('image distante absente + bandeau tant qu\'elle est bloquée', (
    tester,
  ) async {
    await _show(
      tester,
      _msg(
        '<p>Salut</p><img src="cid:inconnu">',
        blocked: ['https://x.test/a.png'],
      ),
    );
    expect(find.text('Afficher'), findsOneWidget);
    expect(find.text('Toujours pour cet expéditeur'), findsOneWidget);
    expect(find.byType(Image), findsNothing);
  });

  testWidgets('HTML coloré : posé sur fond clair', (tester) async {
    await _show(tester, _msg('<p style="color:#fff">Texte blanc</p>'));
    final box = tester
        .widgetList<Container>(find.byType(Container))
        .where(
          (c) =>
              (c.decoration as BoxDecoration?)?.color ==
              const Color(0xFFFFFFFF),
        );
    expect(box, isNotEmpty);
  });

  test('openLink refuse javascript: et file:', () async {
    expect(await openLink('javascript:alert(1)'), isFalse);
    expect(await openLink('file:///etc/passwd'), isFalse);
  });

  testWidgets('squelette pendant la construction, puis le HTML apparaît', (
    tester,
  ) async {
    SharedPreferences.setMockInitialValues({});
    final store = MailStore(await SharedPreferences.getInstance());
    await tester.pumpWidget(
      ChangeNotifierProvider.value(
        value: store,
        child: MaterialApp(
          theme: buildTheme(true),
          home: Scaffold(
            body: MailHtml(
              msg: _msg('<p>Bonjour tout le monde</p>'),
              base: const TextStyle(),
            ),
          ),
        ),
      ),
    );
    expect(find.byType(BodySkeleton), findsOneWidget);
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 300)),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    expect(find.byType(BodySkeleton), findsNothing);
    expect(
      find.textContaining('Bonjour tout', findRichText: true),
      findsOneWidget,
    );
  });

  testWidgets(
    'la carte blanche entre aussi dans le fondu (pas d\u2019apparition brusque)',
    (tester) async {
      SharedPreferences.setMockInitialValues({});
      final store = MailStore(await SharedPreferences.getInstance());
      await tester.pumpWidget(
        ChangeNotifierProvider.value(
          value: store,
          child: MaterialApp(
            theme: buildTheme(true),
            home: Scaffold(
              body: MailHtml(
                msg: _msg('<p style="color:#000">Clair</p>'),
                base: const TextStyle(),
              ),
            ),
          ),
        ),
      );
      final fade = find.ancestor(
        of: find.byWidgetPredicate(
          (w) =>
              w is Container &&
              (w.decoration as BoxDecoration?)?.color ==
                  const Color(0xFFFFFFFF),
        ),
        matching: find.byType(AnimatedOpacity),
      );
      expect(tester.widget<AnimatedOpacity>(fade.first).opacity, 0);
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 300)),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      expect(tester.widget<AnimatedOpacity>(fade.first).opacity, 1);
    },
  );
}
