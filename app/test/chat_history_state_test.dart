import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_history_admin.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/chat_history_state.dart';

import 'fake_ai_engine.dart';

Map<String, dynamic> row(int i, {int version = 1}) => {
  'id': 'c$i',
  'title': 'Conversation $i',
  'created_at': 1000 + i,
  'updated_at': 1000 + i,
  'payload_version': version,
  'payload': '{"messages":[]}',
};

void seed(FakeAiEngine engine, int count) {
  for (var i = 0; i < count; i++) {
    engine.history.conversations.add(row(i));
  }
}

class HeldEngine extends FakeAiEngine {
  Completer<void>? hold;
  bool held = false;

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    final answer = await super.call(request);
    final gate = hold;
    if ((request['request'] as Map)['op'] == 'list_ai_conversations' &&
        gate != null) {
      hold = null;
      held = true;
      await gate.future;
    }
    return answer;
  }
}

ChatHistoryState stateOf(FakeAiEngine engine) =>
    ChatHistoryState(() => AiHistoryAdmin(engine));

void main() {
  test('load_reads_enabled_and_first_page', () async {
    final engine = FakeAiEngine()..history.enabled = false;
    seed(engine, 3);
    final state = stateOf(engine);
    await state.load();
    expect(state.loaded, isTrue);
    expect(state.enabled, isFalse);
    expect(state.items.map((m) => m.id), ['c2', 'c1', 'c0']);
    expect(state.hasMore, isFalse);
    expect(state.error, isNull);
  });

  test('has_more_when_a_51st_row_exists', () async {
    final engine = FakeAiEngine();
    seed(engine, chatHistoryPage + 1);
    final state = stateOf(engine);
    await state.load();
    expect(state.items, hasLength(chatHistoryPage));
    expect(state.hasMore, isTrue);
  });

  test('more_appends_after_the_last_id', () async {
    final engine = FakeAiEngine();
    seed(engine, chatHistoryPage + 3);
    final state = stateOf(engine);
    await state.load();
    final last = state.items.last.id;
    await state.more();
    expect(state.items, hasLength(chatHistoryPage + 3));
    expect(engine.requestsOf('list_ai_conversations').last['before'], last);
    expect(state.hasMore, isFalse);
  });

  test('stale_load_is_ignored', () async {
    final engine = HeldEngine();
    seed(engine, 1);
    final state = stateOf(engine);
    engine.hold = Completer<void>();
    final gate = engine.hold!;
    final first = state.load();
    while (!engine.held) {
      await Future<void>.delayed(Duration.zero);
    }
    engine.history.conversations.add(row(5));
    await state.load();
    gate.complete();
    await first;
    expect(state.items.map((m) => m.id), ['c5', 'c0']);
  });

  test('rename_updates_title_in_list', () async {
    final engine = FakeAiEngine();
    seed(engine, 2);
    final state = stateOf(engine);
    await state.load();
    await state.rename('c0', 'Nouveau titre');
    expect(state.items.firstWhere((m) => m.id == 'c0').title, 'Nouveau titre');
    expect(state.error, isNull);
  });

  test('delete_removes_row', () async {
    final engine = FakeAiEngine();
    seed(engine, 2);
    final state = stateOf(engine);
    await state.load();
    await state.delete('c1');
    expect(state.items.map((m) => m.id), ['c0']);
  });

  test('clear_all_empties_list', () async {
    final engine = FakeAiEngine();
    seed(engine, 2);
    final state = stateOf(engine);
    await state.load();
    await state.clearAll();
    expect(state.items, isEmpty);
    expect(state.error, isNull);
  });

  test('engine_error_is_translated_and_not_thrown', () async {
    final engine = FakeAiEngine();
    seed(engine, 1);
    final state = stateOf(engine);
    engine
      ..failOpName = 'list_ai_conversations'
      ..failOp = const EngineException('server.ai_unavailable', '');
    await state.load();
    expect(state.error, 'Service d’IA indisponible');
    expect(state.loading, isFalse);
    engine.failOp = null;
    await state.load();
    expect(state.error, isNull);
    expect(state.items, hasLength(1));
  });

  test('set_enabled_false_keeps_existing_items', () async {
    final engine = FakeAiEngine();
    seed(engine, 2);
    final state = stateOf(engine);
    await state.load();
    await state.setEnabled(false);
    expect(state.enabled, isFalse);
    expect(engine.history.enabled, isFalse);
    expect(state.items, hasLength(2));
  });

  test('set_enabled_failure_keeps_previous_value', () async {
    final engine = FakeAiEngine();
    final state = stateOf(engine);
    await state.load();
    engine
      ..failOpName = 'set_ai_history_enabled'
      ..failOp = const EngineException('server.ai_unavailable', '');
    await state.setEnabled(false);
    expect(state.enabled, isTrue);
    expect(state.error, 'Service d’IA indisponible');
  });

  test('unavailable_without_admin', () async {
    final state = ChatHistoryState(() => null);
    expect(state.available, isFalse);
    await state.load();
    await state.clearAll();
    expect(state.items, isEmpty);
    expect(state.error, isNull);
  });
}
