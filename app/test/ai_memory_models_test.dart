import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/ai_memory_admin.dart';
import 'package:mira/engine/engine_client.dart';

import 'fake_ai_engine.dart';

const record = {
  'id': 'm1',
  'text': 'Je signe « Noah »',
  'kind': 'preference',
  'source': 'ai',
  'created_at': 1700000000000,
  'model_id': 'embed',
};

void main() {
  late FakeAiEngine engine;
  late AiAdmin admin;
  setUp(() {
    engine = FakeAiEngine();
    admin = AiAdmin(engine);
  });

  test('record_reads_kind_source_and_date', () {
    final r = MemoryItem.fromJson(record);
    expect((r.id, r.kind, r.source), ('m1', 'preference', 'ai'));
    expect(r.createdAt, DateTime.fromMillisecondsSinceEpoch(1700000000000));
    expect(r.byAi, isTrue);
  });

  test('settings_round_trip_local_and_provider', () {
    const local = MemorySettings(enabled: true);
    expect(local.toJson(), {
      'enabled': true,
      'embedder': {'kind': 'local'},
    });
    const remote = MemorySettings(
      enabled: true,
      connection: 'ai-1',
      model: 'embed-1',
    );
    final json = remote.toJson();
    expect(json['embedder'], {
      'kind': 'provider',
      'connection': 'ai-1',
      'model': 'embed-1',
    });
    final back = MemorySettings.fromJson(json);
    expect(
      (back.connection, back.model, back.local),
      ('ai-1', 'embed-1', false),
    );
  });

  test('status_reads_counts_and_settings', () async {
    engine.memory.stale = 2;
    final status = await admin.memoryStatus();
    expect((status.count, status.stale, status.modelInstalled), (0, 2, true));
    expect(status.settings.enabled, isFalse);
    expect(status.settings.local, isTrue);
  });

  test('requests_carry_the_expected_fields', () async {
    await admin.setMemorySettings(const MemorySettings(enabled: true));
    final made = await admin.remember('chat noir', kind: 'fact');
    await admin.searchMemories('chat', limit: 3);
    await admin.updateMemory(made.id, 'chien gris');
    await admin.listMemories(limit: 10, after: 'x');
    expect(engine.requestsOf('remember_memory').single['kind'], 'fact');
    final search = engine.requestsOf('search_memories').single;
    expect((search['query'], search['limit']), ('chat', 3));
    expect(engine.requestsOf('update_memory').single['id'], made.id);
    expect(engine.requestsOf('list_memories').single['after'], 'x');
  });

  test('flow_remember_search_forget_and_forget_all', () async {
    await admin.setMemorySettings(const MemorySettings(enabled: true));
    final a = await admin.remember('chat noir', kind: 'fact');
    await admin.remember('chien gris', kind: 'fact');
    final hits = await admin.searchMemories('chat', limit: 5);
    expect((hits.single.item.id, hits.single.score), (a.id, 0.9));
    await admin.forgetMemory(a.id);
    expect((await admin.listMemories()).length, 1);
    expect(await admin.forgetAllMemories(), 1);
    final report = await admin.reindexMemories();
    expect((report.done, report.failed), (0, 0));
  });

  test('disabled_memory_error_keeps_its_code', () async {
    await expectLater(
      admin.remember('x', kind: 'fact'),
      throwsA(
        isA<EngineException>().having(
          (e) => e.code,
          'code',
          'incompatible_state.ai_memory_disabled',
        ),
      ),
    );
  });

  test('install_returns_the_operation_id', () async {
    engine.memory.modelInstalled = false;
    final id = await admin.installMemoryModel();
    expect(id, isNotEmpty);
    expect(engine.requestsOf('install_memory_model'), hasLength(1));
  });

  test('memory_proposed_line_becomes_an_event', () {
    final event = AiMemoryProposal.fromLine({
      'type': 'memory_proposed',
      'id': 'c9',
      'proposal': {
        'action': 'remember',
        'text': 'Je signe « Noah »',
        'kind': 'preference',
        'id': null,
      },
    });
    expect(
      (event.id, event.action, event.kind),
      ('c9', 'remember', 'preference'),
    );
    expect((event.text, event.memoryId), ('Je signe « Noah »', null));
    expect(AiMemoryProposal.fromJson(event.toJson()).text, event.text);
  });
}
