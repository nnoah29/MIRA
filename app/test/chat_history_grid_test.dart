import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart' show close;
import 'chat_history_support.dart';
import 'chat_history_view_test.dart'
    show key, openHistory, pumpWith, sample, settle;

Finder get gridView => find.byType(GridView);

Future<void> showGrid(WidgetTester tester) async {
  await tester.tap(key('chat-history-layout-grid'));
  await settle(tester);
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('list_is_the_default_and_rows_are_roomy', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
    });
    await openHistory(tester);
    expect(gridView, findsNothing);
    expect(store.historyGrid, isFalse);
    expect(
      tester.getSize(key('chat-history-row-a')).height,
      greaterThanOrEqualTo(64),
    );
    await close(tester, store);
  });

  testWidgets('toggle_shows_cards_and_remembers_the_choice', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
      seedConversation(e, 'b', sample());
    });
    await openHistory(tester);
    await showGrid(tester);
    expect(gridView, findsOneWidget);
    expect(key('chat-history-row-a'), findsOneWidget);
    expect(key('chat-history-row-b'), findsOneWidget);
    expect(store.historyGrid, isTrue);
    expect(store.prefs.getString('historyView'), 'grid');
    await tester.tap(key('chat-history-layout-list'));
    await settle(tester);
    expect(gridView, findsNothing);
    expect(store.prefs.getString('historyView'), 'list');
    await close(tester, store);
  });

  testWidgets('card_opens_renames_and_deletes_like_a_row', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
      seedConversation(e, 'b', sample());
    });
    await openHistory(tester);
    await showGrid(tester);
    await tester.tap(key('chat-history-rename-b'));
    await settle(tester);
    await tester.enterText(key('chat-history-rename-field'), 'Carte renommée');
    await tester.tap(key('confirm-rename-conversation'));
    await settle(tester);
    await settle(tester);
    expect(find.text('Carte renommée'), findsOneWidget);
    await tester.tap(key('chat-history-delete-b'));
    await settle(tester);
    await tester.tap(key('confirm-delete-conversation'));
    await settle(tester);
    expect(key('chat-history-row-b'), findsNothing);
    await tester.tap(key('chat-history-row-a'));
    await settle(tester);
    expect(store.chatCtl.conversationId, 'a');
    expect(gridView, findsNothing);
    await close(tester, store);
  });

  testWidgets('current_card_is_highlighted', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
      seedConversation(e, 'b', sample());
    });
    await store.chatCtl.open('a');
    await openHistory(tester);
    await showGrid(tester);
    expect(key('chat-history-current-a'), findsOneWidget);
    expect(key('chat-history-current-b'), findsNothing);
    await close(tester, store);
  });

  testWidgets('cards_are_disabled_while_a_reply_is_generating', (tester) async {
    final store = await pumpWith(tester, (e) {
      seedConversation(e, 'a', sample());
      e.manual = true;
    });
    final sending = store.chatCtl.send('Salut', noContext);
    await tester.pump(const Duration(milliseconds: 50));
    await openHistory(tester);
    await showGrid(tester);
    await tester.tap(key('chat-history-row-a'));
    await settle(tester);
    expect(store.chatCtl.conversationId, isNull);
    expect(key('chat-history-row-a'), findsOneWidget);
    await store.chatCtl.stop();
    await sending;
    await close(tester, store);
  });

  testWidgets('grid_fits_a_narrow_pane', (tester) async {
    final store = await pumpWith(tester, (e) {
      for (final id in ['a', 'b', 'c']) {
        seedConversation(e, id, sample());
      }
      seedConversation(e, 'bad', const [], version: 9);
    });
    final reports = <String>[];
    final previous = FlutterError.onError;
    FlutterError.onError = (details) => reports.add(details.toString());
    tester.view.physicalSize = const Size(420, 900);
    await settle(tester);
    await openHistory(tester);
    await showGrid(tester);
    expect(key('chat-history-unreadable-bad'), findsOneWidget);
    for (final id in ['a', 'b', 'c', 'bad']) {
      for (final action in ['rename', 'delete']) {
        final button = key('chat-history-$action-$id');
        expect(button, findsOneWidget);
        expect(tester.getTopRight(button).dx, lessThanOrEqualTo(420));
      }
    }
    FlutterError.onError = previous;
    final own = reports.where(
      (r) => r.contains('chat_history') || r.contains('assistant_panel.dart'),
    );
    expect(own, isEmpty, reason: own.join('\n'));
    await close(tester, store);
  });
}
