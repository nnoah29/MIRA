import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/i18n/engine_errors.dart';

import 'ai_settings_harness.dart';

void main() {
  testWidgets('a_card_does_not_repeat_the_provider_name', (tester) async {
    await pumpSettings(
      tester,
      connections: [
        connection('ai-1', label: 'OpenAI'),
        connection('ai-2', provider: 'anthropic', label: 'Pro'),
      ],
    );
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('ai-card-ai-1')),
        matching: find.text('OpenAI'),
      ),
      findsNothing,
    );
    await tapKey(tester, 'ai-tab-anthropic');
    expect(find.text('Pro'), findsOneWidget);
    expect(find.text('Anthropic'), findsOneWidget);
  });

  testWidgets('card_layout_puts_buttons_under_their_field', (tester) async {
    await pumpSettings(
      tester,
      connections: [connection('ai-1', label: 'OpenAI', hasKey: true)],
    );
    Offset at(String key) => tester.getTopLeft(find.byKey(ValueKey(key)));
    final card = at('ai-card-ai-1');
    final field = tester.getBottomLeft(
      find.byKey(const ValueKey('ai-key-field-ai-1')),
    );
    expect(at('ai-key-save-ai-1').dy, greaterThanOrEqualTo(field.dy));
    expect(at('ai-key-clear-ai-1').dy, at('ai-key-save-ai-1').dy);
    expect(at('ai-test-ai-1').dy, greaterThan(at('ai-model-button-ai-1').dy));
    expect(at('ai-key-field-ai-1').dy - card.dy, lessThan(75));
  });

  testWidgets('generic_shows_url_field_others_do_not', (tester) async {
    final engine = await pumpSettings(tester);
    await tester.tap(find.byKey(const ValueKey('ai-add-openai')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('ai-new-url')), findsNothing);
    expect(find.byKey(const ValueKey('ai-url-ai-0')), findsNothing);
    expect(find.text('Connexion OpenAI'), findsNothing);
    expect(engine.requestsOf('add_ai_connection').single['base_url'], isNull);
    await tapKey(tester, 'ai-tab-generic');
    await tester.tap(find.byKey(const ValueKey('ai-add-generic')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('ai-new-url')), findsOneWidget);
    await tester.enterText(
      find.byKey(const ValueKey('ai-new-label')),
      'Maison',
    );
    await tester.enterText(
      find.byKey(const ValueKey('ai-new-url')),
      'http://127.0.0.1:11434/v1',
    );
    await tester.tap(find.byKey(const ValueKey('ai-new-save')));
    await tester.pumpAndSettle();
    final sent = engine.requestsOf('add_ai_connection').last;
    expect(sent['provider'], 'generic');
    expect(sent['base_url'], 'http://127.0.0.1:11434/v1');
    expect(find.byKey(const ValueKey('ai-url-ai-1')), findsOneWidget);
    expect(find.byKey(const ValueKey('ai-url-ai-0')), findsNothing);
  });

  testWidgets('the_first_connection_becomes_the_default', (tester) async {
    final engine = await pumpSettings(tester);
    await tester.tap(find.byKey(const ValueKey('ai-add-openai')));
    await tester.pumpAndSettle();
    expect(engine.requestsOf('set_default_ai_connection'), hasLength(1));
    expect(engine.connections.single['is_default'], isTrue);
  });

  testWidgets('key_field_is_cleared_after_save_and_shows_saved_label', (
    tester,
  ) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-1')],
    );
    expect(find.byKey(const ValueKey('ai-key-saved-ai-1')), findsNothing);
    await tester.enterText(
      find.byKey(const ValueKey('ai-key-field-ai-1')),
      'sk-secret',
    );
    await tester.tap(find.byKey(const ValueKey('ai-key-save-ai-1')));
    await tester.pumpAndSettle();
    expect(engine.requestsOf('set_ai_api_key').single['key'], 'sk-secret');
    expect(fieldText(tester, 'ai-key-field-ai-1'), '');
    expect(find.byKey(const ValueKey('ai-key-saved-ai-1')), findsOneWidget);
    expect(find.text('sk-secret'), findsNothing);
  });

  testWidgets('key_is_never_prefilled_from_engine_data', (tester) async {
    final leaky = connection('ai-1', hasKey: true)
      ..['key'] = 'sk-leak'
      ..['api_key'] = 'sk-leak';
    await pumpSettings(tester, connections: [leaky]);
    expect(fieldText(tester, 'ai-key-field-ai-1'), '');
    expect(find.text('sk-leak'), findsNothing);
    expect(find.byKey(const ValueKey('ai-key-saved-ai-1')), findsOneWidget);
    final field = tester.widget<TextField>(
      find.byKey(const ValueKey('ai-key-field-ai-1')),
    );
    expect(field.obscureText, isTrue);
  });

  testWidgets(
    'test_button_shows_success_and_translated_error_for_key_rejected',
    (tester) async {
      final engine = await pumpSettings(
        tester,
        connections: [connection('ai-1', hasKey: true)],
      );
      await tester.tap(find.byKey(const ValueKey('ai-test-ai-1')));
      await tester.pumpAndSettle();
      expect(find.text('Connexion réussie.'), findsOneWidget);
      engine
        ..failOpName = 'test_ai_connection'
        ..failOp = const EngineException('authentication.ai_key_rejected', 'x');
      await tester.tap(find.byKey(const ValueKey('ai-test-ai-1')));
      await tester.pumpAndSettle();
      expect(find.text('Connexion réussie.'), findsNothing);
      expect(
        find.text(engineErrorTexts['authentication.ai_key_rejected']!),
        findsOneWidget,
      );
    },
  );

  testWidgets('delete_asks_confirmation', (tester) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-1')],
    );
    await tester.tap(find.byKey(const ValueKey('ai-delete-ai-1')));
    await tester.pumpAndSettle();
    expect(engine.requestsOf('remove_ai_connection'), isEmpty);
    await tester.tap(find.text('ANNULER'));
    await tester.pumpAndSettle();
    expect(engine.requestsOf('remove_ai_connection'), isEmpty);
    await tester.tap(find.byKey(const ValueKey('ai-delete-ai-1')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('confirm-delete-ai')));
    await tester.pumpAndSettle();
    expect(
      engine.requestsOf('remove_ai_connection').single['connection'],
      'ai-1',
    );
    expect(find.byKey(const ValueKey('ai-card-ai-1')), findsNothing);
  });

  testWidgets('set_default_updates_radio', (tester) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-1', isDefault: true), connection('ai-2')],
    );
    Finder radio(String id, IconData icon) => find.descendant(
      of: find.byKey(ValueKey('ai-default-$id')),
      matching: find.byIcon(icon),
    );
    expect(radio('ai-1', Icons.radio_button_checked), findsOneWidget);
    expect(radio('ai-2', Icons.radio_button_off), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('ai-default-ai-2')));
    await tester.pumpAndSettle();
    expect(
      engine.requestsOf('set_default_ai_connection').single['connection'],
      'ai-2',
    );
    expect(radio('ai-2', Icons.radio_button_checked), findsOneWidget);
    expect(radio('ai-1', Icons.radio_button_off), findsOneWidget);
  });

  testWidgets('add_key_test_preselects_and_persists_the_first_model', (
    tester,
  ) async {
    final engine = await pumpSettings(tester);
    engine.models.addAll([
      {'id': 'recent', 'created_at': 9},
      {'id': 'old', 'created_at': 1},
    ]);
    await tester.tap(find.byKey(const ValueKey('ai-add-openai')));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const ValueKey('ai-key-field-ai-0')),
      'sk-1',
    );
    await tester.tap(find.byKey(const ValueKey('ai-key-save-ai-0')));
    await tester.pumpAndSettle();
    expect(engine.connections.single['default_model'], 'recent');
    expect(modelButtonText(tester, 'ai-0'), 'recent');
    await tester.tap(find.byKey(const ValueKey('ai-test-ai-0')));
    await tester.pumpAndSettle();
    expect(engine.requestsOf('update_ai_connection'), hasLength(1));
    expect(engine.connections.single['default_model'], 'recent');
  });

  testWidgets('a_chosen_model_is_not_replaced_by_the_first_one', (
    tester,
  ) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-1', hasKey: true, model: 'mine')],
    );
    engine.models.add({'id': 'recent', 'created_at': 9});
    await tester.tap(find.byKey(const ValueKey('ai-test-ai-1')));
    await tester.pumpAndSettle();
    expect(engine.requestsOf('update_ai_connection'), isEmpty);
    expect(modelButtonText(tester, 'ai-1'), 'mine');
  });

  testWidgets('test_saves_the_typed_key_first_and_clears_the_field', (
    tester,
  ) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-1')],
    );
    await tester.enterText(
      find.byKey(const ValueKey('ai-key-field-ai-1')),
      'sk-typed',
    );
    await tester.tap(find.byKey(const ValueKey('ai-test-ai-1')));
    await tester.pumpAndSettle();
    final ops = [for (final c in engine.calls) (c['request'] as Map)['op']];
    expect(
      ops.indexOf('set_ai_api_key'),
      lessThan(ops.indexOf('test_ai_connection')),
    );
    expect(engine.requestsOf('set_ai_api_key').single['key'], 'sk-typed');
    expect(fieldText(tester, 'ai-key-field-ai-1'), '');
    expect(find.text('Connexion réussie.'), findsOneWidget);
  });

  testWidgets('failed_key_save_keeps_the_typed_text_and_skips_the_test', (
    tester,
  ) async {
    final engine = await pumpSettings(
      tester,
      connections: [connection('ai-1')],
    );
    engine
      ..failOpName = 'set_ai_api_key'
      ..failOp = const EngineException('invalid_input.ai_key_invalid', 'x');
    await tester.enterText(
      find.byKey(const ValueKey('ai-key-field-ai-1')),
      'sk-bad',
    );
    await tester.tap(find.byKey(const ValueKey('ai-test-ai-1')));
    await tester.pumpAndSettle();
    expect(fieldText(tester, 'ai-key-field-ai-1'), 'sk-bad');
    expect(engine.requestsOf('test_ai_connection'), isEmpty);
    expect(
      find.text(engineErrorTexts['invalid_input.ai_key_invalid']!),
      findsOneWidget,
    );
  });
}
