import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/store.dart';
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
}
