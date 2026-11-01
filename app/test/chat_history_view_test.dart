import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/store.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';
import 'chat_history_support.dart';
import 'fake_ai_engine.dart';

final openButton = find.byKey(const ValueKey('chat-history-open'));

Finder key(String k) => find.byKey(ValueKey(k));

List<ChatMsg> sample() => [
  ChatMsg(role: 'me', text: 'Question posée'),
  ChatMsg(role: 'ai', text: 'Réponse donnée', fromEngine: true),
];

Future<void> settle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 300));
  await tester.pump(const Duration(milliseconds: 50));
}

Future<void> openHistory(WidgetTester tester) async {
  await tester.tap(openButton);
  await settle(tester);
}

Future<MailStore> pumpWith(
  WidgetTester tester,
  void Function(FakeAiEngine engine) seed,
) async {
  final engine = withDefaultConnection();
  seed(engine);
  final store = await pumpPanel(tester, engine);
  await settle(tester);
  return store;
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('header_button_hidden_in_demo', (tester) async {
    final store = await pumpPanel(tester, null);
    expect(openButton, findsNothing);
    await close(tester, store);
    final withEngine = await pumpPanel(tester, withDefaultConnection());
    expect(openButton, findsOneWidget);
    await close(tester, withEngine);
  });

  testWidgets('rows_show_title_and_highlight_current', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
      seedConversation(e, 'b', sample());
    });
    await openHistory(tester);
    expect(find.text('Titre a'), findsOneWidget);
    expect(find.text('Titre b'), findsOneWidget);
    expect(key('chat-history-current-a'), findsNothing);
    await tester.tap(key('chat-history-row-a'));
    await settle(tester);
    await openHistory(tester);
    expect(key('chat-history-current-a'), findsOneWidget);
    expect(key('chat-history-current-b'), findsNothing);
    await close(tester, store);
  });

  testWidgets('tap_row_opens_and_returns_to_thread', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
    });
    await openHistory(tester);
    await tester.tap(key('chat-history-row-a'));
    await settle(tester);
    expect(store.chatCtl.conversationId, 'a');
    expect(key('chat-history-row-a'), findsNothing);
    expect(find.text('Question posée'), findsOneWidget);
    expect(find.text('Réponse donnée'), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('rows_disabled_while_busy', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
      e.manual = true;
    });
    final sending = store.chatCtl.send('Salut', noContext);
    await tester.pump(const Duration(milliseconds: 50));
    expect(store.chatCtl.busy, isTrue);
    await openHistory(tester);
    await tester.tap(key('chat-history-row-a'));
    await settle(tester);
    expect(key('chat-history-row-a'), findsOneWidget);
    expect(store.chatCtl.conversationId, isNull);
    await store.chatCtl.stop();
    await sending;
    await close(tester, store);
  });

  testWidgets('rename_dialog_limits_to_80_chars_and_updates_list', (
    tester,
  ) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
    });
    await openHistory(tester);
    await tester.tap(key('chat-history-rename-a'));
    await settle(tester);
    final field = key('chat-history-rename-field');
    await tester.enterText(field, 'x' * 100);
    expect(tester.widget<TextField>(field).controller!.text.length, 80);
    await tester.enterText(field, 'Nouveau titre');
    await tester.tap(key('confirm-rename-conversation'));
    await settle(tester);
    await settle(tester);
    expect(find.text('Nouveau titre'), findsOneWidget);
    expect(
      store.history.items.firstWhere((m) => m.id == 'a').title,
      'Nouveau titre',
    );
    await close(tester, store);
  });

  testWidgets('delete_asks_confirmation_and_removes_row', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
      seedConversation(e, 'b', sample());
    });
    await openHistory(tester);
    await tester.tap(key('chat-history-delete-a'));
    await settle(tester);
    expect(key('confirm-delete-conversation'), findsOneWidget);
    await tester.tap(find.text('ANNULER'));
    await settle(tester);
    expect(key('chat-history-row-a'), findsOneWidget);
    await tester.tap(key('chat-history-delete-a'));
    await settle(tester);
    await tester.tap(key('confirm-delete-conversation'));
    await settle(tester);
    expect(key('chat-history-row-a'), findsNothing);
    expect(key('chat-history-row-b'), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('deleting_the_open_conversation_starts_a_fresh_thread', (
    tester,
  ) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
    });
    await store.chatCtl.open('a');
    expect(store.chat, hasLength(3));
    await openHistory(tester);
    await tester.tap(key('chat-history-delete-a'));
    await settle(tester);
    await tester.tap(key('confirm-delete-conversation'));
    await settle(tester);
    expect(store.chatCtl.conversationId, isNull);
    expect(store.chat, hasLength(1));
    await close(tester, store);
  });

  testWidgets('clear_all_asks_confirmation_and_resets_open_thread', (
    tester,
  ) async {
    final engine = withDefaultConnection();
    seedConversation(engine, 'a', sample());
    final store = await pumpPanel(tester, engine);
    await settle(tester);
    await store.chatCtl.open('a');
    await openHistory(tester);
    await tester.tap(key('chat-history-clear'));
    await settle(tester);
    await tester.tap(key('confirm-clear-history'));
    await settle(tester);
    expect(engine.history.conversations, isEmpty);
    expect(store.chatCtl.conversationId, isNull);
    expect(store.chat, hasLength(1));
    engine.scripts.add(reply('Salut'));
    await store.chatCtl.send('Après', noContext);
    expect(engine.history.conversations, hasLength(1));
    expect(engine.history.conversations.single['id'], isNot('a'));
    await close(tester, store);
  });

  testWidgets('empty_state_text', (tester) async {
    final store = await pumpWith(tester, (_) {});
    await openHistory(tester);
    expect(key('chat-history-empty'), findsOneWidget);
    expect(find.text('Aucune conversation enregistrée'), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('disabled_state_text_and_note_in_thread', (tester) async {
    final store = await pumpWith(tester, (e) {
      e.history.enabled = false;
    });
    expect(key('chat-history-disabled-note'), findsOneWidget);
    await openHistory(tester);
    expect(find.text('L’historique est désactivé'), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('unreadable_row_can_be_deleted', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'bad', const [], version: 9);
    });
    await openHistory(tester);
    expect(key('chat-history-unreadable-bad'), findsOneWidget);
    await tester.tap(key('chat-history-row-bad'));
    await settle(tester);
    expect(find.text('Conversation illisible'), findsOneWidget);
    expect(key('chat-history-row-bad'), findsOneWidget);
    await tester.tap(key('chat-history-delete-bad'));
    await settle(tester);
    await tester.tap(key('confirm-delete-conversation'));
    await settle(tester);
    expect(key('chat-history-row-bad'), findsNothing);
    await close(tester, store);
  });

  testWidgets('narrow_width_keeps_actions_visible', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
    });
    final reports = <String>[];
    final previous = FlutterError.onError;
    FlutterError.onError = (details) => reports.add(details.toString());
    tester.view.physicalSize = const Size(420, 900);
    await settle(tester);
    await openHistory(tester);
    for (final id in ['chat-history-rename-a', 'chat-history-delete-a']) {
      expect(key(id), findsOneWidget);
      expect(tester.getTopRight(key(id)).dx, lessThanOrEqualTo(420));
    }
    FlutterError.onError = previous;
    final own = reports.where(
      (r) => r.contains('chat_history') || r.contains('assistant_panel.dart'),
    );
    expect(own, isEmpty, reason: own.join('\n'));
    await close(tester, store);
  });
}
