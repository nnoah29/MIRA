import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'onboarding_flow_test.dart' show pumpFlow, tapKey;

Future<void> goTo(WidgetTester tester, int page) async {
  for (var i = 0; i < page; i++) {
    await tapKey(tester, 'onb-next');
  }
}

void main() {
  testWidgets('appearance_cards_apply_theme_immediately', (tester) async {
    final store = await pumpFlow(tester);
    await goTo(tester, 2);
    await tapKey(tester, 'onb-theme-light');
    expect(store.theme, 'light');
    expect(store.prefs.getString('theme'), 'light');
    await tapKey(tester, 'onb-theme-dark');
    expect(store.theme, 'dark');
  });

  testWidgets('appearance_marks_current_theme', (tester) async {
    await pumpFlow(tester);
    await goTo(tester, 2);
    expect(find.byKey(const ValueKey('onb-theme-dark-on')), findsOneWidget);
    expect(find.byKey(const ValueKey('onb-theme-light-on')), findsNothing);
    await tapKey(tester, 'onb-theme-light');
    expect(find.byKey(const ValueKey('onb-theme-light-on')), findsOneWidget);
    expect(find.byKey(const ValueKey('onb-theme-dark-on')), findsNothing);
  });
}
