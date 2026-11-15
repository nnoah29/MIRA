import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/settings_panel.dart';

import 'ai_settings_harness.dart';

void main() {
  testWidgets('la mémoire n’est plus dans l’onglet IA', (tester) async {
    await pumpSettings(tester);
    expect(byKey('ai-memory-section'), findsNothing);
    expect(byKey('ai-history-toggle'), findsOneWidget);
  });

  test('l’onglet Mémoire suit IA avec un moteur, et n’existe pas sans', () {
    final tabs = settingsTabs(true);
    expect(tabs[tabs.indexOf('IA') + 1], 'Mémoire');
    expect(tabs.last, 'À propos');
    expect(settingsTabs(false), isNot(contains('Mémoire')));
  });
}
