import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/ai_journal_state.dart';

import 'fake_ai_engine.dart';

Map<String, dynamic> row(int i) => {
  'id': 'j$i',
  'at': 1700000000000 - i * 1000,
  'provider': 'openai',
  'model': 'm',
  'local': false,
  'kinds': ['mail_message'],
  'chars': 10,
  'masked': 0,
  'input_tokens': 1,
  'output_tokens': 1,
  'outcome': 'end_turn',
};

class HeldEngine extends FakeAiEngine {
  Completer<void>? hold;
  bool held = false;

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    final answer = await super.call(request);
    final gate = hold;
    if ((request['request'] as Map)['op'] == 'list_ai_journal' &&
        gate != null) {
      hold = null;
      held = true;
      await gate.future;
    }
    return answer;
  }
}

void main() {
  test('failed_clear_keeps_the_error', () async {
    final engine = FakeAiEngine()..journal.addAll([row(0), row(1)]);
    final state = AiJournalState(AiAdmin(engine));
    await state.load();
    engine
      ..failOpName = 'clear_ai_journal'
      ..failOp = const EngineException('server.ai_unavailable', '');
    await state.clear();
    expect(state.error, 'Service d’IA indisponible');
    expect(state.rows, hasLength(2));
    engine.failOp = null;
    await state.clear();
    expect(state.error, isNull);
    expect(state.rows, isEmpty);
  });

  test('more_finishing_after_clear_does_not_reappend_rows', () async {
    final engine = HeldEngine()
      ..journal.addAll([for (var i = 0; i < 60; i++) row(i)]);
    final state = AiJournalState(AiAdmin(engine));
    await state.load();
    expect(state.rows, hasLength(50));
    final gate = engine.hold = Completer<void>();
    final more = state.more();
    await pumpEventQueue();
    expect(engine.held, isTrue);
    await state.clear();
    expect(state.rows, isEmpty);
    gate.complete();
    await more;
    expect(state.rows, isEmpty);
    expect(state.loading, isFalse);
    expect(state.hasMore, isFalse);
  });

  test(
    'load_during_a_fetch_is_not_skipped_and_the_stale_fetch_is_ignored',
    () async {
      final engine = HeldEngine()..journal.add(row(0));
      final state = AiJournalState(AiAdmin(engine));
      final gate = engine.hold = Completer<void>();
      final first = state.load();
      await pumpEventQueue();
      engine.journal
        ..clear()
        ..add(row(1));
      await state.load();
      expect(state.rows.single.id, 'j1');
      gate.complete();
      await first;
      expect(state.rows.single.id, 'j1');
      expect(state.loading, isFalse);
    },
  );
}
