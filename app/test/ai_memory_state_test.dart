import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/ai_memory_state.dart';

import 'fake_ai_engine.dart';

Map<String, dynamic> row(String id, String text, int at) => {
  'id': id,
  'text': text,
  'kind': 'fact',
  'source': 'user',
  'created_at': at,
  'model_id': 'embed',
};

void main() {
  late FakeAiEngine engine;
  late AiMemoryState state;
  setUp(() {
    engine = FakeAiEngine();
    engine.memory.settings = {
      'enabled': true,
      'embedder': {'kind': 'local'},
    };
    engine.memory.records
      ..add(row('a', 'chat noir', 1))
      ..add(row('b', 'chien gris', 2));
    state = AiMemoryState(
      AiAdmin(engine),
      pollEvery: const Duration(milliseconds: 1),
    );
  });

  test('load_reads_the_status_and_the_most_recent_first', () async {
    await state.load();
    expect(state.loaded, isTrue);
    expect((state.status!.count, state.enabled), (2, true));
    expect(state.items.map((m) => m.id), ['b', 'a']);
  });

  test(
    'switching_the_toggle_writes_the_settings_and_keeps_the_embedder',
    () async {
      await state.load();
      await state.setEnabled(false);
      expect(state.enabled, isFalse);
      expect(engine.memory.settings['enabled'], false);
      await state.setProvider('ai-1', 'embed-1');
      expect(engine.memory.settings['embedder'], {
        'kind': 'provider',
        'connection': 'ai-1',
        'model': 'embed-1',
      });
      expect(state.status!.settings.enabled, isFalse);
      await state.setLocal();
      expect(engine.memory.settings['embedder'], {'kind': 'local'});
    },
  );

  test(
    'install_follows_the_operation_then_reads_the_model_as_present',
    () async {
      engine.memory.modelInstalled = false;
      engine.local.script.addAll([
        {'type': 'running', 'done': 50, 'total': 100},
        {'type': 'finished'},
      ]);
      await state.load();
      expect(state.status!.modelInstalled, isFalse);
      await state.install();
      expect(state.installing, isFalse);
      expect(state.status!.modelInstalled, isTrue);
      expect(state.error, isNull);
    },
  );

  test('a_failed_install_shows_its_error_and_stops', () async {
    engine.memory.failInstall = const EngineException(
      'incompatible_state.ai_local_busy',
      '',
    );
    await state.install();
    expect(state.installing, isFalse);
    expect(state.error, contains('installation'));
  });

  test('search_fills_the_hits_and_an_empty_query_clears_them', () async {
    await state.load();
    await state.search('chat');
    expect(state.hits!.map((h) => h.item.id), ['a']);
    await state.search('  ');
    expect(state.hits, isNull);
  });

  test('edit_delete_and_forget_all_update_the_list_and_the_count', () async {
    await state.load();
    await state.edit('a', 'chat blanc');
    expect(state.items.firstWhere((m) => m.id == 'a').text, 'chat blanc');
    await state.delete('b');
    expect(state.items.map((m) => m.id), ['a']);
    expect(state.status!.count, 1);
    await state.forgetAll();
    expect(state.items, isEmpty);
    expect(state.status!.count, 0);
  });

  test('a_disabled_memory_refuses_an_edit_with_a_readable_error', () async {
    await state.load();
    await state.setEnabled(false);
    await state.edit('a', 'x');
    expect(state.error, contains('désactivée'));
    expect(state.items.firstWhere((m) => m.id == 'a').text, 'chat noir');
  });

  test('reindex_clears_the_stale_count_and_reports_the_total', () async {
    engine.memory.stale = 2;
    await state.load();
    expect(state.status!.stale, 2);
    await state.reindex();
    expect(state.status!.stale, 0);
    expect(state.reindexed, 2);
  });
}
