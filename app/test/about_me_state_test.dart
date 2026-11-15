import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/about_me_state.dart';

import 'fake_ai_engine.dart';

void main() {
  late FakeAiEngine engine;
  late AboutMeState state;
  setUp(() {
    engine = FakeAiEngine();
    state = AboutMeState(AiAdmin(engine));
  });

  test('load_reads_the_saved_text_into_the_draft', () async {
    engine.about.text = 'Je suis médecin.';
    await state.load();
    expect(
      (state.loaded, state.saved, state.draft),
      (true, 'Je suis médecin.', 'Je suis médecin.'),
    );
    expect(state.dirty, isFalse);
  });

  test('editing_marks_dirty_and_saving_sends_the_trimmed_text', () async {
    await state.load();
    state.edit('  Je signe « Noah ».  ');
    expect(state.dirty, isTrue);
    expect(await state.save(), isTrue);
    expect(engine.about.text, 'Je signe « Noah ».');
    expect(
      (state.saved, state.justSaved, state.dirty),
      ('Je signe « Noah ».', true, false),
    );
    expect(
      engine.requestsOf('set_about_me').single['text'],
      '  Je signe « Noah ».  ',
    );
  });

  test('a_text_over_the_limit_is_not_sent', () async {
    await state.load();
    state.edit('é' * 1001);
    expect(state.valid, isFalse);
    expect(await state.save(), isFalse);
    expect(engine.requestsOf('set_about_me'), isEmpty);
  });

  test('an_unchanged_draft_sends_nothing', () async {
    await state.load();
    expect(await state.save(), isFalse);
    expect(engine.requestsOf('set_about_me'), isEmpty);
  });

  test('a_refusal_keeps_the_saved_text_and_shows_the_error', () async {
    engine.about.text = 'avant';
    await state.load();
    state.edit('é' * 1000);
    engine.failOp = const EngineException('invalid_input.ai_about_text', '');
    engine.failOpName = 'set_about_me';
    expect(await state.save(), isFalse);
    expect(state.saved, 'avant');
    expect(state.error, contains('trop long'));
  });

  test('clearing_the_field_saves_an_empty_text', () async {
    engine.about.text = 'x';
    await state.load();
    state.edit('   ');
    expect(await state.save(), isTrue);
    expect(engine.about.text, '');
  });
}
