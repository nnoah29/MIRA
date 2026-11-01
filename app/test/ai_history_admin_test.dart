import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_history_admin.dart';
import 'package:mira/engine/engine_client.dart';

import 'fake_ai_engine.dart';

void main() {
  late FakeAiEngine engine;
  late AiHistoryAdmin admin;
  setUp(() {
    engine = FakeAiEngine();
    admin = AiHistoryAdmin(engine);
  });

  test('save_sends_the_expected_request', () async {
    final meta = await admin.save(
      id: 'c-1',
      title: 'Ma question',
      payloadVersion: 1,
      payload: '{"messages":[]}',
    );
    final sent = engine.calls.single;
    expect(sent['domain'], 'ai');
    final r = (sent['request'] as Map).cast<String, dynamic>();
    expect(r['op'], 'save_ai_conversation');
    expect(r['id'], 'c-1');
    expect(r['title'], 'Ma question');
    expect(r['payload_version'], 1);
    expect(r['payload'], '{"messages":[]}');
    expect(meta.id, 'c-1');
    expect(meta.title, 'Ma question');
    expect(meta.updatedAt, meta.createdAt);
  });

  test('list_parses_meta_and_forwards_before', () async {
    for (final id in ['a', 'b', 'c']) {
      await admin.save(id: id, title: id, payloadVersion: 1, payload: '{}');
    }
    final first = await admin.list(limit: 2);
    expect(first.map((m) => m.id), ['c', 'b']);
    final next = await admin.list(limit: 2, before: 'b');
    expect(next.map((m) => m.id), ['a']);
    final sent = engine.requestsOf('list_ai_conversations').last;
    expect(sent['limit'], 2);
    expect(sent['before'], 'b');
  });

  test('get_returns_payload_text', () async {
    await admin.save(
      id: 'c-1',
      title: 'T',
      payloadVersion: 2,
      payload: '{"k":1}',
    );
    final doc = await admin.get('c-1');
    expect(doc.payload, '{"k":1}');
    expect(doc.meta.id, 'c-1');
    expect(doc.meta.payloadVersion, 2);
  });

  test('enabled_and_set_enabled_round_trip', () async {
    expect(await admin.enabled(), isTrue);
    expect(await admin.setEnabled(false), isFalse);
    expect(
      engine.requestsOf('set_ai_history_enabled').single['enabled'],
      false,
    );
    expect(await admin.enabled(), isFalse);
  });

  test('engine_error_becomes_EngineException', () async {
    engine
      ..failOpName = 'get_ai_conversation'
      ..failOp = const EngineException('not_found.ai_conversation', 'absente');
    await expectLater(
      admin.get('c-1'),
      throwsA(
        isA<EngineException>().having(
          (e) => e.code,
          'code',
          'not_found.ai_conversation',
        ),
      ),
    );
  });
}
