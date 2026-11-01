import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/ai_settings.dart';
import 'package:mira/ui/overlays.dart';
import 'package:mira/ui/rule_widgets.dart';

import 'ai_settings_harness.dart';
import 'chat_history_support.dart';
import 'fake_ai_engine.dart';

Finder inToggle(String label) =>
    find.descendant(of: byKey('ai-history-toggle'), matching: find.text(label));

FakeAiEngine withConversations(int count) {
  final engine = FakeAiEngine();
  for (var i = 0; i < count; i++) {
    seedConversation(engine, 'c$i', const []);
  }
  return engine;
}

void main() {
  testWidgets('toggle_is_on_by_default_and_note_is_shown', (tester) async {
    await pumpSettings(tester);
    expect(
      tester.widget<RuleSegment>(byKey('ai-history-toggle')).value,
      isTrue,
    );
    expect(
      find.text(
        'Enregistre sur cet appareil le texte affiché des conversations. Les contenus envoyés au modèle en coulisses ne sont pas conservés.',
      ),
      findsOneWidget,
    );
  });

  testWidgets('toggling_off_sends_false_and_keeps_conversations', (
    tester,
  ) async {
    final engine = await pumpSettings(tester, using: withConversations(2));
    await tester.tap(inToggle('DÉSACTIVÉ'));
    await tester.pumpAndSettle();
    expect(engine.requestsOf('set_ai_history_enabled').last['enabled'], false);
    expect(engine.history.enabled, isFalse);
    expect(engine.history.conversations, hasLength(2));
    expect(
      tester.widget<RuleSegment>(byKey('ai-history-toggle')).value,
      isFalse,
    );
    await tester.tap(inToggle('ACTIVÉ'));
    await tester.pumpAndSettle();
    expect(engine.history.enabled, isTrue);
  });

  testWidgets('toggle_reads_the_engine_value_when_off', (tester) async {
    final off = FakeAiEngine()..history.enabled = false;
    await pumpSettings(tester, using: off);
    expect(
      tester.widget<RuleSegment>(byKey('ai-history-toggle')).value,
      isFalse,
    );
  });

  testWidgets('clear_button_asks_confirmation_then_clears', (tester) async {
    final engine = await pumpSettings(tester, using: withConversations(2));
    await tapKey(tester, 'ai-history-clear');
    expect(byKey('confirm-clear-history'), findsOneWidget);
    await tester.tap(find.text('ANNULER'));
    await tester.pumpAndSettle();
    expect(engine.history.conversations, hasLength(2));
    await tapKey(tester, 'ai-history-clear');
    await tapKey(tester, 'confirm-clear-history');
    expect(engine.history.conversations, isEmpty);
  });

  testWidgets('clear_calls_onHistoryCleared_only_after_confirmation', (
    tester,
  ) async {
    final engine = withConversations(1)..providers.clear();
    var cleared = 0;
    await tester.binding.setSurfaceSize(const Size(900, 2400));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    addTearDown(Pops.close);
    await tester.pumpWidget(
      MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(
          body: SingleChildScrollView(
            child: AiSettings(
              admin: AiAdmin(engine),
              onHistoryCleared: () => cleared++,
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tapKey(tester, 'ai-history-clear');
    await tester.tap(find.text('ANNULER'));
    await tester.pumpAndSettle();
    expect(cleared, 0);
    await tapKey(tester, 'ai-history-clear');
    await tapKey(tester, 'confirm-clear-history');
    expect(cleared, 1);
  });
}
