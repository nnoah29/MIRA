import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/assistant.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';

Future<void> settle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 200));
  await tester.pump(const Duration(milliseconds: 200));
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  test('cleanup_suggestion_is_gone', () {
    final labels = [for (final s in suggestions) s.first];
    expect(labels, isNot(contains('Nettoie ma boîte')));
    expect(labels, contains('Mes non lus'));
  });

  testWidgets('unread_suggestion_sends_engine_question_when_connected', (
    tester,
  ) async {
    final engine = withDefaultConnection()
      ..scripts.add([
        {'type': 'text', 'text': 'Voici.'},
        {'type': 'done', 'reason': 'end_turn'},
      ]);
    final store = await pumpPanel(tester, engine);
    await tester.tap(find.text('MES NON LUS'));
    await settle(tester);
    final request = (engine.chatStarts.single['request'] as Map);
    final last = (request['messages'] as List).last as Map;
    expect(last['text'], 'Montre-moi mes mails non lus');
    expect(store.chat.last.fromEngine, isTrue);
    await close(tester, store);
  });

  testWidgets('demo_fallback_unchanged_without_connection', (tester) async {
    final store = await pumpPanel(tester, null);
    await tester.tap(find.text('MES NON LUS'));
    await tester.pump(const Duration(seconds: 3));
    expect(store.chat.last.fromEngine, isFalse);
    expect(store.chat.last.text, contains('non lu'));
    await close(tester, store);
  });

  testWidgets('fallback_text_promises_no_cleanup', (tester) async {
    final store = await pumpPanel(tester, null);
    final text = answerTo(store, 'ok').text;
    expect(text, contains('Je peux'));
    expect(text, isNot(contains('nettoyer')));
    await close(tester, store);
  });
}
