import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/ai_consent_dialog.dart';
import 'package:mira/ui/rule_widgets.dart';

import 'ai_settings_harness.dart';

bool chipOn(WidgetTester tester, String key) =>
    tester.widget<RuleChip>(byKey(key)).selected;

void main() {
  testWidgets('consent_dialog_lists_three_kinds', (tester) async {
    List<String>? answer;
    await tester.pumpWidget(
      MaterialApp(
        theme: buildTheme(false),
        home: Builder(
          builder: (context) => TextButton(
            onPressed: () async =>
                answer = await showAiConsentDialog(context, 'Mistral AI'),
            child: const Text('go'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
    for (final kind in ['mail_message', 'mail_list', 'tool_result']) {
      expect(byKey('ai-consent-kind-$kind'), findsOneWidget, reason: kind);
    }
    expect(chipOn(tester, 'ai-consent-kind-mail_message'), isTrue);
    expect(chipOn(tester, 'ai-consent-kind-mail_list'), isFalse);
    await tester.tap(byKey('ai-consent-kind-mail_list'));
    await tester.pumpAndSettle();
    await tester.tap(byKey('ai-consent-accept'));
    await tester.pumpAndSettle();
    expect(answer, containsAll(['mail_message', 'mail_list']));
    expect(answer, isNot(contains('tool_result')));
  });

  testWidgets('privacy_section_toggles_each_kind_independently', (
    tester,
  ) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-0', isDefault: true, hasKey: true)],
    );
    await tapKey(tester, 'ai-privacy-kinds-mail_list-ai-0');
    expect(engine.requestsOf('set_ai_allowed_content').last['kinds'], [
      'mail_list',
    ]);
    await tapKey(tester, 'ai-privacy-kinds-tool_result-ai-0');
    expect(
      engine.requestsOf('set_ai_allowed_content').last['kinds'],
      unorderedEquals(['mail_list', 'tool_result']),
    );
    expect(chipOn(tester, 'ai-privacy-kinds-ai-0'), isFalse);
    expect(chipOn(tester, 'ai-privacy-kinds-mail_list-ai-0'), isTrue);
  });
}
