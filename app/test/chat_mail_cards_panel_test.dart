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

Map<String, dynamic> shown(List<Map<String, dynamic>> cards, {String? title}) =>
    {
      'type': 'mail_cards_shown',
      'id': 'c2',
      'cards': {'title': title, 'cards': cards},
    };

Future<void> settle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 200));
  await tester.pump(const Duration(milliseconds: 200));
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('cards_show_title_reason_and_open_the_mail_on_tap', (
    tester,
  ) async {
    final engine = withDefaultConnection();
    final store = await pumpPanel(tester, engine);
    final first = store.mails.first;
    final second = store.mails[1];
    engine.scripts.add([
      finished([first.id, second.id]),
      shown([
        {'id': first.id, 'reason': 'RAISON-UNE'},
        {'id': second.id, 'reason': null},
      ], title: 'TITRE-LISTE'),
      {'type': 'text', 'text': 'Voilà.'},
      {'type': 'done', 'reason': 'end_turn'},
    ]);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await settle(tester);
    expect(find.text('TITRE-LISTE'), findsOneWidget);
    expect(find.text('RAISON-UNE'), findsOneWidget);
    expect(find.byKey(ValueKey('chat-card-${first.id}')), findsOneWidget);
    expect(find.byKey(ValueKey('chat-card-${second.id}')), findsOneWidget);
    expect(find.text('SOURCES'), findsNothing);
    await tester.ensureVisible(find.byKey(ValueKey('chat-card-${second.id}')));
    await tester.tap(find.byKey(ValueKey('chat-card-${second.id}')));
    await tester.pump();
    expect(store.open, second.id);
    await close(tester, store);
  });

  testWidgets('an_id_no_tool_returned_is_never_shown_as_a_card', (
    tester,
  ) async {
    final engine = withDefaultConnection();
    final store = await pumpPanel(tester, engine);
    final first = store.mails.first;
    final other = store.mails[1];
    engine.scripts.add([
      finished([first.id]),
      shown([
        {'id': first.id},
        {'id': other.id},
        {'id': 'inconnu'},
      ]),
      {'type': 'text', 'text': 'Voilà.'},
      {'type': 'done', 'reason': 'end_turn'},
    ]);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await settle(tester);
    expect(find.byKey(ValueKey('chat-card-${first.id}')), findsOneWidget);
    expect(find.byKey(ValueKey('chat-card-${other.id}')), findsNothing);
    expect(find.byKey(const ValueKey('chat-card-inconnu')), findsNothing);
    await close(tester, store);
  });
}
