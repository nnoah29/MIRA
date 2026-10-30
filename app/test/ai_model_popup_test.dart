import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'ai_settings_harness.dart';
import 'fake_ai_engine.dart';

Future<FakeAiEngine> pumpWithModels(
  WidgetTester tester, {
  String? model = 'm1',
  List<String>? models = const ['m1', 'm2', 'm3'],
}) => pumpSettings(
  tester,
  connections: [connection('ai-1', hasKey: true, model: model, models: models)],
);

Future<void> openPopup(WidgetTester tester) =>
    tapKey(tester, 'ai-model-button-ai-1');

bool checked(WidgetTester tester, String model) =>
    tester
        .widget<Opacity>(
          find.descendant(
            of: byKey('ai-model-ai-1-$model'),
            matching: find.byType(Opacity),
          ),
        )
        .opacity ==
    1;

void main() {
  testWidgets('the_button_shows_the_current_model', (tester) async {
    await pumpWithModels(tester);
    expect(modelButtonText(tester, 'ai-1'), 'm1');
  });

  testWidgets('the_button_shows_a_placeholder_without_model', (tester) async {
    await pumpWithModels(tester, model: null);
    expect(modelButtonText(tester, 'ai-1'), 'Choisir un modèle');
  });

  testWidgets('the_button_opens_the_popup_with_models_in_order_and_a_check', (
    tester,
  ) async {
    await pumpWithModels(tester);
    expect(byKey('ai-model-search-ai-1'), findsNothing);
    await openPopup(tester);
    expect(byKey('ai-model-search-ai-1'), findsOneWidget);
    final ys = [
      for (final m in ['m1', 'm2', 'm3'])
        tester.getTopLeft(byKey('ai-model-ai-1-$m')).dy,
    ];
    expect(ys, orderedEquals([...ys]..sort()));
    expect(checked(tester, 'm1'), isTrue);
    expect(checked(tester, 'm2'), isFalse);
  });

  testWidgets('search_filters_the_list_as_you_type', (tester) async {
    await pumpWithModels(tester, models: ['gpt-4', 'gpt-5', 'o3']);
    await openPopup(tester);
    await tester.enterText(byKey('ai-model-search-ai-1'), 'GPT-5');
    await tester.pumpAndSettle();
    expect(byKey('ai-model-ai-1-gpt-5'), findsOneWidget);
    expect(byKey('ai-model-ai-1-gpt-4'), findsNothing);
    expect(byKey('ai-model-ai-1-o3'), findsNothing);
    await tester.enterText(byKey('ai-model-search-ai-1'), 'zzz');
    await tester.pumpAndSettle();
    expect(find.text('Aucun modèle ne correspond.'), findsOneWidget);
    expect(byKey('ai-model-custom-ai-1'), findsOneWidget);
  });

  testWidgets('selecting_a_model_persists_it_and_closes_the_popup', (
    tester,
  ) async {
    final engine = await pumpWithModels(tester);
    await openPopup(tester);
    await tapKey(tester, 'ai-model-ai-1-m2');
    final sent = engine.requestsOf('update_ai_connection').single;
    expect(sent['default_model'], 'm2');
    expect(byKey('ai-model-search-ai-1'), findsNothing);
    expect(modelButtonText(tester, 'ai-1'), 'm2');
  });

  testWidgets('selecting_the_current_model_only_closes', (tester) async {
    final engine = await pumpWithModels(tester);
    await openPopup(tester);
    await tapKey(tester, 'ai-model-ai-1-m1');
    expect(engine.requestsOf('update_ai_connection'), isEmpty);
    expect(byKey('ai-model-search-ai-1'), findsNothing);
  });

  testWidgets('the_free_text_row_commits_a_typed_name_on_enter', (
    tester,
  ) async {
    final engine = await pumpWithModels(tester);
    await openPopup(tester);
    await tapKey(tester, 'ai-model-custom-ai-1');
    expect(byKey('ai-model-search-ai-1'), findsNothing);
    await tester.enterText(byKey('ai-model-field-ai-1'), '  mon-modele ');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    final sent = engine.requestsOf('update_ai_connection').single;
    expect(sent['default_model'], 'mon-modele');
    expect(byKey('ai-model-field-ai-1'), findsNothing);
    expect(modelButtonText(tester, 'ai-1'), 'mon-modele');
  });

  testWidgets('escape_closes_the_free_text_mode_without_saving', (
    tester,
  ) async {
    final engine = await pumpWithModels(tester);
    await openPopup(tester);
    await tapKey(tester, 'ai-model-custom-ai-1');
    await tester.enterText(byKey('ai-model-field-ai-1'), 'abandonne');
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(byKey('ai-model-field-ai-1'), findsNothing);
    expect(engine.requestsOf('update_ai_connection'), isEmpty);
  });

  testWidgets('tapping_outside_closes_the_popup', (tester) async {
    final engine = await pumpWithModels(tester);
    await openPopup(tester);
    await tester.tapAt(const Offset(5, 5));
    await tester.pumpAndSettle();
    expect(byKey('ai-model-search-ai-1'), findsNothing);
    expect(engine.requestsOf('update_ai_connection'), isEmpty);
  });

  testWidgets('refresh_asks_the_engine_again_and_updates_the_open_list', (
    tester,
  ) async {
    final engine = await pumpWithModels(tester, models: ['m1']);
    engine.models.addAll([
      {'id': 'm1', 'created_at': 1},
      {'id': 'nouveau', 'created_at': 2},
    ]);
    await openPopup(tester);
    expect(byKey('ai-model-ai-1-nouveau'), findsNothing);
    await tapKey(tester, 'ai-model-refresh-ai-1');
    expect(engine.requestsOf('list_ai_models').single['refresh'], isTrue);
    expect(byKey('ai-model-ai-1-nouveau'), findsOneWidget);
    expect(byKey('ai-model-search-ai-1'), findsOneWidget);
  });

  testWidgets('an_empty_list_shows_the_empty_state_and_the_free_text_row', (
    tester,
  ) async {
    await pumpWithModels(tester, models: null);
    await openPopup(tester);
    expect(
      find.text('Aucun modèle relevé : saisis son nom, ou actualise la liste.'),
      findsOneWidget,
    );
    expect(byKey('ai-model-custom-ai-1'), findsOneWidget);
    expect(byKey('ai-model-refresh-ai-1'), findsOneWidget);
  });
}
