import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_attached_file.dart';
import 'package:mira/state/store.dart';
import 'package:mira/ui/overlays.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';
import 'fake_ai_engine.dart';

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('demo_banner_visible_without_connection', (tester) async {
    final store = await pumpPanel(tester, null);
    expect(find.byKey(const ValueKey('demo-banner')), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('demo_banner_visible_with_engine_but_no_connection', (
    tester,
  ) async {
    final store = await pumpPanel(tester, FakeAiEngine());
    expect(find.byKey(const ValueKey('demo-banner')), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('banner_hidden_when_a_connection_answers', (tester) async {
    final store = await pumpPanel(tester, withDefaultConnection());
    expect(find.byKey(const ValueKey('demo-banner')), findsNothing);
    await close(tester, store);
  });

  testWidgets('stop_button_visible_while_busy', (tester) async {
    final engine = withDefaultConnection()..manual = true;
    final store = await pumpPanel(tester, engine);
    expect(find.byKey(const ValueKey('chat-send')), findsOneWidget);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 50));
    expect(find.byKey(const ValueKey('chat-stop')), findsOneWidget);
    expect(find.byKey(const ValueKey('chat-send')), findsNothing);
    await tester.tap(find.byKey(const ValueKey('chat-stop')));
    await tester.pump(const Duration(milliseconds: 50));
    expect(find.byKey(const ValueKey('chat-send')), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('insert_in_reply_opens_composer_with_text', (tester) async {
    final engine = withDefaultConnection()
      ..scripts.add([
        {'type': 'text', 'text': 'Bonjour **Bob**, merci.'},
        {'type': 'done', 'reason': 'end_turn'},
      ]);
    final store = await pumpPanel(tester, engine);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump(const Duration(milliseconds: 300));
    expect(store.chat.last.text, 'Bonjour **Bob**, merci.');
    await tester.tap(find.byKey(const ValueKey('insert-in-reply')));
    await tester.pump(const Duration(milliseconds: 300));
    expect(store.view, PaneView.compose);
    expect(store.c!.body.text, contains('Bonjour Bob, merci.'));
    expect(store.c!.body.text, isNot(contains('**')));
    await close(tester, store);
  });

  testWidgets('context_banner_is_gone', (tester) async {
    final store = await pumpPanel(tester, withDefaultConnection());
    expect(find.byKey(const ValueKey('ctx-text')), findsNothing);
    expect(find.byKey(const ValueKey('ctx-icon')), findsNothing);
    expect(find.byKey(const ValueKey('ctx-drop')), findsNothing);
    await close(tester, store);
  });

  testWidgets('opening_a_mail_does_not_add_it_to_the_context', (tester) async {
    final engine = withDefaultConnection()
      ..scripts.add([
        {'type': 'done', 'reason': 'end_turn'},
      ]);
    final store = await pumpPanel(tester, engine);
    expect(store.open, isNotNull);
    expect(store.chatAttachedIds, isEmpty);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 300));
    final request = engine.chatStarts.single['request'] as Map;
    expect(request['context'], isEmpty);
    await close(tester, store);
  });

  testWidgets('attach_menu_lists_recent_mails_and_attaches_several', (
    tester,
  ) async {
    final store = await pumpPanel(tester, withDefaultConnection());
    await tester.tap(find.byKey(const ValueKey('chat-attach')));
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byKey(const ValueKey('chat-attach-menu')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('chat-attach-option-mail')));
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byKey(const ValueKey('chat-mail-picker')), findsOneWidget);
    final ids = store.recentInboxMails().take(2).map((m) => m.id).toList();
    for (final id in ids) {
      await tester.tap(find.byKey(ValueKey('chat-mail-row-$id')));
      await tester.pump(const Duration(milliseconds: 100));
    }
    expect(store.chatAttachedIds, ids);
    for (final id in ids) {
      expect(find.byKey(ValueKey('chat-attach-chip-$id')), findsOneWidget);
    }
    Pops.close();
    await tester.pump();
    await close(tester, store);
  });

  testWidgets('attach_popover_closes_on_outside_click', (tester) async {
    final store = await pumpPanel(tester, withDefaultConnection());
    await tester.tap(find.byKey(const ValueKey('chat-attach')));
    await tester.pump(const Duration(milliseconds: 200));
    await tester.tap(find.byKey(const ValueKey('chat-attach-option-mail')));
    await tester.pump(const Duration(milliseconds: 200));
    expect(find.byKey(const ValueKey('chat-mail-picker')), findsOneWidget);
    await tester.tapAt(const Offset(300, 100));
    await tester.pump(const Duration(milliseconds: 200));
    expect(find.byKey(const ValueKey('chat-mail-picker')), findsNothing);
    expect(find.byKey(const ValueKey('chat-attach-menu')), findsNothing);
    await close(tester, store);
  });

  testWidgets('attach_menu_offers_a_file_option', (tester) async {
    final store = await pumpPanel(tester, withDefaultConnection());
    await tester.tap(find.byKey(const ValueKey('chat-attach')));
    await tester.pump(const Duration(milliseconds: 200));
    expect(
      find.byKey(const ValueKey('chat-attach-option-mail')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('chat-attach-option-file')),
      findsOneWidget,
    );
    Pops.close();
    await tester.pump();
    await close(tester, store);
  });

  testWidgets('attached_file_shows_a_chip_is_sent_then_cleared', (
    tester,
  ) async {
    final engine = withDefaultConnection()
      ..scripts.add([
        {'type': 'done', 'reason': 'end_turn'},
      ]);
    final store = await pumpPanel(tester, engine);
    store.attachChatFile(
      const ChatAttachedFile(id: 'f1', name: 'devis.txt', text: 'Total : 42'),
    );
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byKey(const ValueKey('chat-attach-chip-f1')), findsOneWidget);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 300));
    final request = engine.chatStarts.single['request'] as Map;
    final part = (request['context'] as List).single as Map;
    expect(part['kind'], 'attachment');
    expect(part['label'], 'devis.txt');
    expect(store.chatFiles, isEmpty);
    await close(tester, store);
  });

  testWidgets('file_chip_cross_detaches_the_file', (tester) async {
    final store = await pumpPanel(tester, withDefaultConnection());
    store.attachChatFile(
      const ChatAttachedFile(id: 'f1', name: 'devis.txt', text: 'Total : 42'),
    );
    await tester.pump(const Duration(milliseconds: 100));
    await tester.tap(find.byKey(const ValueKey('chat-attach-remove-f1')));
    await tester.pump(const Duration(milliseconds: 100));
    expect(store.chatFiles, isEmpty);
    expect(find.byKey(const ValueKey('chat-attach-chip-f1')), findsNothing);
    await close(tester, store);
  });

  testWidgets('attached_mails_are_sent_then_cleared', (tester) async {
    final engine = withDefaultConnection()
      ..scripts.add([
        {'type': 'done', 'reason': 'end_turn'},
      ]);
    final store = await pumpPanel(tester, engine);
    final ids = store.recentInboxMails().take(2).map((m) => m.id).toList();
    ids.forEach(store.toggleChatAttachment);
    await tester.pump(const Duration(milliseconds: 100));
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 300));
    final request = engine.chatStarts.single['request'] as Map;
    final context = request['context'] as List;
    expect(context, hasLength(2));
    expect(context.first['kind'], 'mail_message');
    expect(store.chatAttachedIds, isEmpty);
    await close(tester, store);
  });

  testWidgets('chip_cross_detaches_the_mail', (tester) async {
    final store = await pumpPanel(tester, withDefaultConnection());
    final id = store.recentInboxMails().first.id;
    store.toggleChatAttachment(id);
    await tester.pump(const Duration(milliseconds: 100));
    await tester.tap(find.byKey(ValueKey('chat-attach-remove-$id')));
    await tester.pump(const Duration(milliseconds: 100));
    expect(store.chatAttachedIds, isEmpty);
    expect(find.byKey(ValueKey('chat-attach-chip-$id')), findsNothing);
    await close(tester, store);
  });
}
