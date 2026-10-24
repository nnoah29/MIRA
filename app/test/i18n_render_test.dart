import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/i18n/i18n.dart';
import 'package:mira/state/store.dart';

import 'app_test.dart' show pumpApp, teardown;

/// Textes affichés à l'écran qui sont une clé française pourtant traduite : un `t()` a été oublié.
List<String> untranslatedOnScreen(WidgetTester tester, Set<String> sample) {
  final shown = <String>{};
  for (final w in tester.widgetList<Text>(find.byType(Text))) {
    final text = w.data ?? w.textSpan?.toPlainText();
    if (text != null) shown.add(text.trim());
  }
  return [
    for (final fr in appLang.strings.keys)
      if (shown.contains(fr.trim()) && appLang.strings[fr]!.trim() != fr.trim())
        fr,
  ]..removeWhere(sample.contains);
}

void main() {
  tearDown(() => appLang = AppLang.fr);

  testWidgets(
    'en anglais, aucun texte de l’interface principale n’est resté en français',
    (tester) async {
      final s = await pumpApp(tester, prefs: {'lang': 'en'});
      appLang = AppLang.en;
      await tester.pump(const Duration(milliseconds: 100));
      expect(untranslatedOnScreen(tester, {}), isEmpty);
      await tester.tap(find.byTooltip('Switch account'));
      await tester.pump(const Duration(milliseconds: 300));
      expect(find.text('Add an account'), findsOneWidget);
      expect(untranslatedOnScreen(tester, {}), isEmpty);
      await teardown(tester, s);
    },
  );

  testWidgets('en anglais, chaque onglet des réglages est traduit', (
    tester,
  ) async {
    final s = await pumpApp(
      tester,
      prefs: {'lang': 'en'},
      size: const Size(1500, 1200),
    );
    appLang = AppLang.en;
    s.showView(PaneView.settings);
    await tester.pump(const Duration(milliseconds: 300));
    var count = 0;
    while (find.byKey(ValueKey('settings-tab-$count')).evaluate().isNotEmpty) {
      count++;
    }
    expect(count, greaterThan(3));
    for (var i = 0; i < count; i++) {
      await tester.tap(find.byKey(ValueKey('settings-tab-$i')));
      await tester.pump(const Duration(milliseconds: 300));
      expect(untranslatedOnScreen(tester, {}), isEmpty, reason: 'onglet $i');
    }
    await teardown(tester, s);
  });
}
