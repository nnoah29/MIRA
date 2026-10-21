import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/mail_layout.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/html_body.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Charge la police de l'app : sans elle, le texte de test est bien plus large que dans l'app réelle.
Future<void> _loadFont(WidgetTester tester) async {
  final file = File('assets/google_fonts/HankenGrotesk-Regular.ttf');
  final bytes = file.readAsBytesSync();
  final loader = FontLoader('MailTest')
    ..addFont(Future.value(ByteData.view(bytes.buffer)));
  await tester.runAsync(loader.load);
}

/// Rend l'en-tête d'une newsletter (structure réelle d'Alison, sans liens de suivi).
Future<void> _showAlison(WidgetTester tester) async {
  await _loadFont(tester);
  SharedPreferences.setMockInitialValues({});
  final store = MailStore(await SharedPreferences.getInstance());
  final raw = File('test/fixtures/alison_header.html').readAsStringSync();
  final msg = Msg(from: const Person('A', 'a@x.org'), date: 0, body: const [])
    ..html = prepareMailHtml(raw)
    ..blocked = const []
    ..inline = {};
  await tester.binding.setSurfaceSize(const Size(1000, 800));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(
          body: SingleChildScrollView(
            child: MailHtml(
              msg: msg,
              base: const TextStyle(fontSize: 16, fontFamily: 'MailTest'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 300)),
  );
  await tester.pump(const Duration(milliseconds: 600));
}

void main() {
  testWidgets('les icônes de partage en ligne restent sur une seule ligne', (
    tester,
  ) async {
    await _showAlison(tester);
    final images = find.byType(Image);
    final first = tester.getRect(images.at(0));
    final second = tester.getRect(images.at(1));
    final third = tester.getRect(images.at(2));
    expect((second.top - first.top).abs(), lessThan(2));
    expect((third.top - first.top).abs(), lessThan(2));
    expect(second.left, greaterThan(first.right - 1));
    expect(third.left, greaterThan(second.right - 1));
  });

  testWidgets('le logo à 60 % se calcule sur la cellule de 270 px', (
    tester,
  ) async {
    await _showAlison(tester);
    expect(tester.getSize(find.byType(Image).at(3)).width, 162);
  });

  testWidgets('les liens de navigation ne se cassent pas lettre par lettre', (
    tester,
  ) async {
    await _showAlison(tester);
    final text = find.textContaining('My Dashboard', findRichText: true);
    final box = tester.getRect(text.first);
    expect(box.width, greaterThan(60));
    expect(box.height, lessThan(40));
    final categories = find.textContaining(
      'Course Categories',
      findRichText: true,
    );
    expect(tester.getRect(categories.first).height, lessThan(40));
  });
}
