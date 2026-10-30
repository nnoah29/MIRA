import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';

const toolCalled = {
  'type': 'tool_called',
  'id': 'c1',
  'name': 'search_messages',
  'summary': 'devis',
};

Map<String, dynamic> finished(
  String state, {
  int results = 0,
  List<String> sources = const [],
  String? denied,
}) => {
  'type': 'tool_finished',
  'id': 'c1',
  'state': state,
  'results': results,
  'sources': sources,
  'denied_kind': denied,
};

List<Map<String, dynamic>> script(
  Map<String, dynamic> end, [
  String reply = 'Voici.',
]) => [
  {'type': 'started', 'operation': 'op-1'},
  toolCalled,
  end,
  {'type': 'text', 'text': reply},
  {'type': 'done', 'reason': 'end_turn'},
];

Future<void> settle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 200));
  await tester.pump(const Duration(milliseconds: 200));
}

Future<void> ask(WidgetTester tester) async {
  await tester.tap(find.text('RÉSUME CE MAIL'));
  await settle(tester);
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('running_chip_shows_label_and_query', (tester) async {
    final engine = withDefaultConnection()..manual = true;
    final store = await pumpPanel(tester, engine);
    await ask(tester);
    engine.live!
      ..add({'type': 'started', 'operation': 'op-1'})
      ..add(toolCalled);
    await settle(tester);
    expect(find.text('Recherche « devis »'), findsOneWidget);
    engine.live!
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await settle(tester);
    await close(tester, store);
  });

  testWidgets('ok_chip_shows_result_count', (tester) async {
    final engine = withDefaultConnection()
      ..scripts.add(script(finished('ok', results: 2)));
    final store = await pumpPanel(tester, engine);
    await ask(tester);
    expect(find.text('Recherche « devis » · 2 résultats'), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('denied_chip_offers_authorize_below_it', (tester) async {
    final engine = withDefaultConnection()
      ..scripts.add(script(finished('denied', denied: 'mail_list')));
    final store = await pumpPanel(tester, engine);
    await ask(tester);
    final chip = find.byKey(const ValueKey('tool-chip-c1'));
    final authorize = find.byKey(const ValueKey('tool-authorize-c1'));
    expect(find.text('Recherche « devis » · refusé'), findsOneWidget);
    expect(authorize, findsOneWidget);
    expect(
      tester.getTopLeft(authorize).dy,
      greaterThanOrEqualTo(tester.getBottomLeft(chip).dy),
    );
    await close(tester, store);
  });

  testWidgets('authorize_opens_dialog_prechecked_and_retries', (tester) async {
    final engine = withDefaultConnection()
      ..scripts.addAll([
        script(finished('denied', denied: 'mail_list'), 'Pas d’accès.'),
        script(finished('ok', results: 1), 'Voici.'),
      ]);
    final store = await pumpPanel(tester, engine);
    await ask(tester);
    await tester.tap(find.byKey(const ValueKey('tool-authorize-c1')));
    await settle(tester);
    final title = tester.widget<Text>(
      find.byKey(const ValueKey('ai-content-title')),
    );
    expect(
      title.data,
      'Autoriser OpenAI à recevoir : les listes de messages ?',
    );
    await tester.tap(find.byKey(const ValueKey('ai-content-accept')));
    await settle(tester);
    await settle(tester);
    expect(engine.requestsOf('set_ai_allowed_content').single['kinds'], [
      'mail_list',
    ]);
    expect(engine.chatStarts, hasLength(2));
    expect(store.chat.length, 3);
    expect(store.chat.last.text, 'Voici.');
    await close(tester, store);
  });

  testWidgets('declining_authorization_changes_nothing', (tester) async {
    final engine = withDefaultConnection()
      ..scripts.add(script(finished('denied', denied: 'mail_list')));
    final store = await pumpPanel(tester, engine);
    await ask(tester);
    await tester.tap(find.byKey(const ValueKey('tool-authorize-c1')));
    await settle(tester);
    await tester.tap(find.byKey(const ValueKey('ai-content-refuse')));
    await settle(tester);
    expect(engine.chatStarts, hasLength(1));
    expect(store.chat.length, 3);
    expect(store.chat.last.tools.single.deniedKind, 'mail_list');
    await close(tester, store);
  });

  testWidgets('sources_list_known_messages_and_skips_unknown_ones', (
    tester,
  ) async {
    final engine = withDefaultConnection();
    final store = await pumpPanel(tester, engine);
    final known = store.mails.first;
    engine.scripts.add(
      script(finished('ok', results: 2, sources: [known.id, 'zz-inconnu'])),
    );
    await ask(tester);
    expect(find.text('SOURCES'), findsOneWidget);
    expect(find.text(known.subject), findsWidgets);
    expect(find.byKey(const ValueKey('chat-source-zz-inconnu')), findsNothing);
    await tester.tap(find.byKey(ValueKey('chat-source-${known.id}')));
    await settle(tester);
    expect(store.open, known.id);
    await close(tester, store);
  });
}
