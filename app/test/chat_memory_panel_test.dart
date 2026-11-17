import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';

Future<void> settle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 200));
  await tester.pump(const Duration(milliseconds: 200));
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('a_proposal_becomes_a_card_and_confirming_writes_the_memory', (
    tester,
  ) async {
    final engine = withDefaultConnection();
    engine.memory.settings = {
      'enabled': true,
      'embedder': {'kind': 'local'},
    };
    final store = await pumpPanel(tester, engine);
    engine.scripts.add([
      {
        'type': 'memory_proposed',
        'id': 'c1',
        'proposal': {
          'action': 'remember',
          'text': 'TEXTE-RETENU',
          'kind': 'fact',
          'id': null,
        },
      },
      {'type': 'text', 'text': 'Je te propose de le retenir.'},
      {'type': 'done', 'reason': 'end_turn'},
    ]);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await settle(tester);
    expect(find.text('TEXTE-RETENU'), findsOneWidget);
    expect(engine.memory.records, isEmpty);
    final confirm = find.byKey(const ValueKey('chat-memory-confirm-c1'));
    await tester.ensureVisible(confirm);
    await tester.tap(confirm);
    await settle(tester);
    expect(engine.memory.records.single['text'], 'TEXTE-RETENU');
    expect(find.byKey(const ValueKey('chat-memory-done-c1')), findsOneWidget);
    await close(tester, store);
  });
}
