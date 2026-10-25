import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/store.dart';
import 'package:mira/ui/body_renderer.dart';

import 'app_test.dart' show pumpApp, teardown;

void main() {
  testWidgets('Apparence propose le mode d’affichage et le retient', (
    tester,
  ) async {
    final s = await pumpApp(tester, size: const Size(1500, 1200));
    expect(s.htmlMode, HtmlMode.auto);
    s.showView(PaneView.settings);
    await tester.pump(const Duration(milliseconds: 300));
    expect(
      find.text('Affichage du HTML'),
      webViewEnabled ? findsOneWidget : findsNothing,
    );
    s.setSetting('htmlMode', 'widgets');
    expect(s.htmlMode, HtmlMode.widgets);
    expect(s.prefs.getString('htmlMode'), 'widgets');
    s.setSetting('htmlMode', 'n’importe quoi');
    expect(s.htmlMode, HtmlMode.auto);
    await teardown(tester, s);
  });
}
