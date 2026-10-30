import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';

Map<String, dynamic> finished(List<String> sources) => {
  'type': 'tool_finished',
  'id': 'c1',
  'state': 'ok',
  'results': sources.length,
  'sources': sources,
  'denied_kind': null,
};

Future<void> settle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 200));
  await tester.pump(const Duration(milliseconds: 200));
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('cited_messages_show_as_cards_without_a_sources_header', (
    tester,
  ) async {
    final engine = withDefaultConnection();
    final store = await pumpPanel(tester, engine);
    final cited = store.mails.first;
    final other = store.mails[1];
    engine.scripts.add([
      finished([cited.id, other.id]),
      {'type': 'text', 'text': 'Voici [mail:${cited.id}] à traiter.'},
      {'type': 'done', 'reason': 'end_turn'},
    ]);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await settle(tester);
    expect(find.text(cited.subject), findsWidgets);
    expect(find.textContaining('[mail:'), findsNothing);
    expect(find.textContaining('Voici'), findsOneWidget);
    expect(find.text('SOURCES'), findsNothing);
    expect(find.byKey(ValueKey('chat-source-${other.id}')), findsNothing);
    await close(tester, store);
  });

  testWidgets('uncited_results_show_a_capped_sources_list', (tester) async {
    final engine = withDefaultConnection();
    final store = await pumpPanel(tester, engine);
    final ids = [for (final m in store.mails.take(8)) m.id];
    engine.scripts.add([
      finished(ids),
      {'type': 'text', 'text': 'Voici ce que j’ai vu.'},
      {'type': 'done', 'reason': 'end_turn'},
    ]);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await settle(tester);
    expect(find.text('SOURCES'), findsOneWidget);
    final shown = [
      for (final id in ids)
        if (find.byKey(ValueKey('chat-source-$id')).evaluate().isNotEmpty) id,
    ];
    expect(shown, hasLength(5));
    await close(tester, store);
  });
}
