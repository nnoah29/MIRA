import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/app_info.dart';
import 'package:mira/state/store.dart';
import 'package:mira/ui/about_settings.dart';

import 'app_test.dart' show pumpApp, teardown;

void main() {
  test('la version affichée est celle du pubspec', () {
    final pubspec = File('pubspec.yaml').readAsStringSync();
    final m = RegExp(
      r'^version: (\S+?)\+',
      multiLine: true,
    ).firstMatch(pubspec);
    expect(m!.group(1), appVersion);
  });

  testWidgets(
    'le dernier onglet des réglages est « À propos » et ouvre les liens',
    (tester) async {
      final opened = <String>[];
      openLink = (u) async {
        opened.add(u.toString());
        return true;
      };
      final s = await pumpApp(tester, size: const Size(1500, 1200));
      s.showView(PaneView.settings);
      await tester.pump(const Duration(milliseconds: 300));
      await tester.tap(find.byKey(const ValueKey('settings-tab-5')));
      await tester.pump(const Duration(milliseconds: 300));
      expect(find.text('MIRA $appVersion'), findsOneWidget);
      for (final url in [siteUrl, sourceUrl, privacyUrl, termsUrl, authorUrl]) {
        await tester.tap(find.byKey(ValueKey('about-$url')));
      }
      expect(opened, [siteUrl, sourceUrl, privacyUrl, termsUrl, authorUrl]);
      await teardown(tester, s);
    },
  );
}
