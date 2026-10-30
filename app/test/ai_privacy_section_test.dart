import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/rule_widgets.dart';

import 'ai_settings_harness.dart';
import 'fake_ai_engine.dart';

class GatedEngine extends FakeAiEngine {
  final gate = Completer<void>();

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    final op = (request['request'] as Map)['op'];
    if (op == 'set_ai_allowed_content') await gate.future;
    return super.call(request);
  }
}

bool chipOn(WidgetTester tester, String key) =>
    tester.widget<RuleChip>(byKey(key)).selected;

bool switchOn(WidgetTester tester, String key) =>
    tester.widget<RuleToggle>(byKey(key)).value;

void main() {
  testWidgets('mail_message_checkbox_reflects_and_saves_the_allowed_kinds', (
    tester,
  ) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-0', isDefault: true, hasKey: true)],
    );
    expect(chipOn(tester, 'ai-privacy-kinds-ai-0'), isFalse);
    await tapKey(tester, 'ai-privacy-kinds-ai-0');
    expect(engine.requestsOf('set_ai_allowed_content').last['kinds'], [
      'mail_message',
    ]);
    expect(chipOn(tester, 'ai-privacy-kinds-ai-0'), isTrue);
    await tapKey(tester, 'ai-privacy-kinds-ai-0');
    expect(engine.requestsOf('set_ai_allowed_content').last['kinds'], isEmpty);
    expect(chipOn(tester, 'ai-privacy-kinds-ai-0'), isFalse);
  });

  testWidgets('masking_switch_reflects_and_saves_the_mode', (tester) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-0', isDefault: true, hasKey: true)],
    );
    expect(switchOn(tester, 'ai-privacy-mask-ai-0'), isTrue);
    expect(byKey('ai-privacy-cat-ai-0-iban'), findsOneWidget);
    await tapKey(tester, 'ai-privacy-mask-ai-0');
    final sent = engine.requestsOf('set_ai_masking').single['masking'] as Map;
    expect(sent['mode'], 'off');
    expect(sent['categories'], ['iban', 'card', 'secret', 'otp']);
    expect(switchOn(tester, 'ai-privacy-mask-ai-0'), isFalse);
    expect(byKey('ai-privacy-cat-ai-0-iban'), findsNothing);
  });

  testWidgets('category_checkboxes_save_the_list', (tester) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-0', isDefault: true, hasKey: true)],
    );
    expect(chipOn(tester, 'ai-privacy-cat-ai-0-email'), isFalse);
    await tapKey(tester, 'ai-privacy-cat-ai-0-email');
    var sent = engine.requestsOf('set_ai_masking').last['masking'] as Map;
    expect(sent['categories'], ['iban', 'card', 'secret', 'otp', 'email']);
    await tapKey(tester, 'ai-privacy-cat-ai-0-iban');
    sent = engine.requestsOf('set_ai_masking').last['masking'] as Map;
    expect(sent['mode'], 'mask');
    expect(sent['categories'], ['card', 'secret', 'otp', 'email']);
    expect(chipOn(tester, 'ai-privacy-cat-ai-0-iban'), isFalse);
  });

  testWidgets('best_effort_notice_is_shown', (tester) async {
    await pumpSettings(
      tester,
      connections: [connection('ai-0', isDefault: true, hasKey: true)],
    );
    expect(
      find.text('Détection au mieux : elle peut se tromper.'),
      findsOneWidget,
    );
  });

  testWidgets('section_is_disabled_while_busy', (tester) async {
    final engine = GatedEngine();
    await pumpSettings(
      tester,
      using: engine,
      connections: [connection('ai-0', isDefault: true, hasKey: true)],
    );
    await tester.tap(byKey('ai-privacy-kinds-ai-0'));
    await tester.pump();
    final absorb = tester.widget<AbsorbPointer>(byKey('ai-privacy-ai-0'));
    expect(absorb.absorbing, isTrue);
    await tester.tap(byKey('ai-privacy-mask-ai-0'), warnIfMissed: false);
    await tester.pump();
    expect(engine.requestsOf('set_ai_masking'), isEmpty);
    engine.gate.complete();
    await tester.pumpAndSettle();
    expect(
      tester.widget<AbsorbPointer>(byKey('ai-privacy-ai-0')).absorbing,
      isFalse,
    );
  });
}
