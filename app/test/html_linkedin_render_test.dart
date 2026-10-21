import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/mail_layout.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/html_body.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Rend la notification de groupe LinkedIn (structure réelle, données remplacées) dans 1000 px.
Future<void> _showLinkedIn(WidgetTester tester) async {
  SharedPreferences.setMockInitialValues({});
  final store = MailStore(await SharedPreferences.getInstance());
  final raw = File('test/fixtures/linkedin_group_mail.html').readAsStringSync();
  final msg = Msg(from: const Person('L', 'l@x.org'), date: 0, body: const [])
    ..html = prepareMailHtml(raw)
    ..blocked = const []
    ..inline = {};
  await tester.binding.setSurfaceSize(const Size(1000, 1300));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(
          body: SingleChildScrollView(
            child: MailHtml(msg: msg, base: const TextStyle(fontSize: 16)),
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

TextAlign _alignOf(WidgetTester tester, String needle) {
  final text = find.textContaining(needle, findRichText: true).first;
  return tester.widget<RichText>(text).textAlign;
}

void main() {
  testWidgets('une table align=center ne centre pas son texte', (tester) async {
    await _showLinkedIn(tester);
    for (final needle in [
      'Voir les recommandations',
      'Valeria Buitrago',
      'October 26',
    ]) {
      expect(_alignOf(tester, needle), isNot(TextAlign.center), reason: needle);
    }
  });

  testWidgets('une cellule text-align:center garde son propre texte centré', (
    tester,
  ) async {
    await _showLinkedIn(tester);
    expect(_alignOf(tester, 'Ne manquez aucune'), TextAlign.center);
  });

  testWidgets('en-tête : logo à gauche, avatar à droite', (tester) async {
    await _showLinkedIn(tester);
    final images = find.byType(Image);
    final logo = tester.getRect(images.at(0));
    final avatar = tester.getRect(images.at(1));
    expect(avatar.left - logo.right, greaterThan(150));
  });

  testWidgets('les images à border-radius 100 % sont rondes', (tester) async {
    await _showLinkedIn(tester);
    final images = find.byType(Image);
    for (final index in [1, 2]) {
      final clip = find.ancestor(
        of: images.at(index),
        matching: find.byType(ClipOval),
      );
      expect(clip, findsOneWidget, reason: 'image $index');
    }
    final plain = find.ancestor(
      of: images.at(0),
      matching: find.byType(ClipOval),
    );
    expect(plain, findsNothing);
  });

  testWidgets('une ligne avatar | texte garde le texte contre l’avatar', (
    tester,
  ) async {
    await _showLinkedIn(tester);
    final avatar = tester.getRect(find.byType(Image).at(2));
    final name = find.textContaining('Valeria Buitrago', findRichText: true);
    expect(tester.getRect(name.first).left - avatar.right, lessThan(40));
    final group = tester.getRect(find.byType(Image).at(4));
    final text = find.textContaining('Welcome to the', findRichText: true);
    expect(tester.getRect(text.first).left - group.right, lessThan(40));
  });

  testWidgets(
    'la carte à border-radius 8 px a des coins arrondis et sa bordure',
    (tester) async {
      await _showLinkedIn(tester);
      final rounded = find.byWidgetPredicate((widget) {
        if (widget is! DecoratedBox) return false;
        final decoration = widget.decoration;
        return decoration is BoxDecoration &&
            decoration.border != null &&
            decoration.borderRadius == BorderRadius.circular(8);
      });
      expect(rounded, findsWidgets);
    },
  );
}
