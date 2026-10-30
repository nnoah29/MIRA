import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/theme/app_theme.dart';

import 'ai_settings_harness.dart';
import 'fake_ai_engine.dart';

Map<String, dynamic> entry(
  int i, {
  bool local = false,
  String outcome = 'end_turn',
  int masked = 0,
  List<Map<String, Object>> tools = const [],
}) => {
  'id': 'j$i',
  'at': 1700000000000 - i * 1000,
  'provider': 'openai',
  'model': 'model-$i',
  'local': local,
  'kinds': ['mail_message'],
  'chars': 1234 + i,
  'masked': masked,
  'input_tokens': 321,
  'output_tokens': 45,
  'outcome': outcome,
  'tools': tools,
};

Future<FakeAiEngine> openJournal(
  WidgetTester tester,
  List<Map<String, dynamic>> rows,
) async {
  final engine = FakeAiEngine()..journal.addAll(rows);
  await pumpSettings(tester, using: engine);
  await tester.tap(find.text('JOURNAL'));
  await tester.pumpAndSettle();
  return engine;
}

Future<void> tapVisible(WidgetTester tester, String key) async {
  await tester.ensureVisible(byKey(key));
  await tester.pumpAndSettle();
  await tester.tap(byKey(key));
  await tester.pumpAndSettle();
}

class HeldEngine extends FakeAiEngine {
  final gate = Completer<void>();

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    final op = (request['request'] as Map)['op'];
    if (op == 'list_ai_journal') await gate.future;
    return super.call(request);
  }
}

String rowText(WidgetTester tester, int i) => tester
    .widgetList<Text>(
      find.descendant(
        of: byKey('ai-journal-row-$i'),
        matching: find.byType(Text),
      ),
    )
    .map((t) => t.data)
    .join(' | ');

void main() {
  setUpAll(() => initializeDateFormatting('fr_FR'));

  testWidgets('lists_rows_newest_first_with_counts', (tester) async {
    await openJournal(tester, [entry(0, masked: 2), entry(1)]);
    final first = tester.getTopLeft(byKey('ai-journal-row-0')).dy;
    final second = tester.getTopLeft(byKey('ai-journal-row-1')).dy;
    expect(first, lessThan(second));
    final text = rowText(tester, 0);
    expect(text, contains('OpenAI · model-0'));
    expect(text, contains('Mail ouvert'));
    expect(text, contains('1234 caractères'));
    expect(text, contains('2 masqués'));
    expect(text, contains('321 → 45 jetons'));
    expect(text, contains('Terminé'));
  });

  testWidgets('tools_line_lists_names_and_counts', (tester) async {
    await openJournal(tester, [
      entry(
        0,
        tools: [
          {'name': 'search_messages', 'count': 2},
          {'name': 'get_thread', 'count': 1},
        ],
      ),
      entry(1),
    ]);
    expect(
      rowText(tester, 0),
      contains('Outils : search_messages ×2, get_thread ×1'),
    );
    expect(rowText(tester, 1), isNot(contains('Outils')));
  });

  testWidgets('max_tool_rounds_reads_as_a_limit_not_an_error', (tester) async {
    await openJournal(tester, [entry(0, outcome: 'max_tool_rounds')]);
    expect(rowText(tester, 0), contains('Limite de tours atteinte'));
  });

  testWidgets('local_rows_are_marked', (tester) async {
    await openJournal(tester, [entry(0, local: true), entry(1)]);
    expect(rowText(tester, 0), contains('Local'));
    expect(rowText(tester, 1), isNot(contains('Local')));
  });

  testWidgets('more_button_loads_the_next_page', (tester) async {
    final engine = await openJournal(tester, [
      for (var i = 0; i < 60; i++) entry(i),
    ]);
    expect(engine.requestsOf('list_ai_journal').single['limit'], 51);
    expect(byKey('ai-journal-row-49'), findsOneWidget);
    expect(byKey('ai-journal-row-50'), findsNothing);
    await tapVisible(tester, 'ai-journal-more');
    expect(engine.requestsOf('list_ai_journal').last['before'], 'j49');
    expect(byKey('ai-journal-row-59'), findsOneWidget);
    expect(byKey('ai-journal-more'), findsNothing);
  });

  testWidgets('clear_asks_confirmation_then_empties', (tester) async {
    final engine = await openJournal(tester, [entry(0), entry(1)]);
    await tapVisible(tester, 'ai-journal-clear');
    await tester.tap(find.text('ANNULER'));
    await tester.pumpAndSettle();
    expect(engine.requestsOf('clear_ai_journal'), isEmpty);
    expect(byKey('ai-journal-row-0'), findsOneWidget);
    await tapVisible(tester, 'ai-journal-clear');
    await tapKey(tester, 'confirm-clear-journal');
    expect(engine.requestsOf('clear_ai_journal'), hasLength(1));
    expect(byKey('ai-journal-row-0'), findsNothing);
    expect(byKey('ai-journal-empty'), findsOneWidget);
  });

  testWidgets('empty_state_is_shown', (tester) async {
    await openJournal(tester, const []);
    expect(find.text('Aucun envoi pour l’instant.'), findsOneWidget);
    expect(byKey('ai-journal-clear'), findsNothing);
  });

  testWidgets('outcome_is_translated_through_the_engine_error_texts', (
    tester,
  ) async {
    await openJournal(tester, [
      entry(0, outcome: 'server.ai_unavailable'),
      entry(1, outcome: 'incompatible_state.ai_content_not_allowed'),
      entry(2, outcome: 'cancelled'),
    ]);
    expect(rowText(tester, 0), contains('Service d’IA indisponible'));
    expect(
      rowText(tester, 1),
      contains('Ce service d’IA n’est pas autorisé à recevoir ce type'),
    );
    expect(rowText(tester, 2), contains('Annulé'));
    expect(rowText(tester, 0), isNot(contains('server.ai_unavailable')));
  });

  testWidgets('toggle_switches_between_connections_and_journal', (
    tester,
  ) async {
    final engine = FakeAiEngine()..journal.add(entry(0));
    await pumpSettings(tester, using: engine);
    expect(byKey('ai-journal-view'), findsNothing);
    expect(byKey('ai-tab-openai'), findsOneWidget);
    await tester.tap(find.text('JOURNAL'));
    await tester.pumpAndSettle();
    expect(byKey('ai-journal-view'), findsOneWidget);
    expect(byKey('ai-tab-openai'), findsNothing);
    await tester.tap(find.text('CONNEXIONS'));
    await tester.pumpAndSettle();
    expect(byKey('ai-journal-view'), findsNothing);
    expect(byKey('ai-tab-openai'), findsOneWidget);
  });

  testWidgets('failed_first_load_shows_the_error_and_a_retry_button', (
    tester,
  ) async {
    final engine = FakeAiEngine()
      ..journal.add(entry(0))
      ..failOpName = 'list_ai_journal'
      ..failOp = const EngineException('server.ai_unavailable', '');
    await pumpSettings(tester, using: engine);
    await tester.tap(find.text('JOURNAL'));
    await tester.pumpAndSettle();
    expect(find.text('Service d’IA indisponible'), findsOneWidget);
    expect(byKey('ai-journal-empty'), findsNothing);
    expect(find.text('Aucun envoi pour l’instant.'), findsNothing);
    engine.failOp = null;
    await tapKey(tester, 'ai-journal-retry');
    expect(byKey('ai-journal-row-0'), findsOneWidget);
    expect(byKey('ai-journal-retry'), findsNothing);
    expect(find.text('Service d’IA indisponible'), findsNothing);
  });

  testWidgets('loading_text_is_shown_before_the_first_page', (tester) async {
    final engine = HeldEngine()..journal.add(entry(0));
    await pumpSettings(tester, using: engine);
    await tester.tap(find.text('JOURNAL'));
    await tester.pump();
    expect(byKey('ai-journal-loading'), findsOneWidget);
    expect(byKey('ai-journal-empty'), findsNothing);
    engine.gate.complete();
    await tester.pumpAndSettle();
    expect(byKey('ai-journal-loading'), findsNothing);
    expect(byKey('ai-journal-row-0'), findsOneWidget);
  });

  testWidgets('exactly_one_page_offers_no_show_more_button', (tester) async {
    await openJournal(tester, [for (var i = 0; i < 50; i++) entry(i)]);
    expect(byKey('ai-journal-row-49'), findsOneWidget);
    expect(byKey('ai-journal-more'), findsNothing);
  });

  testWidgets('one_row_past_a_page_offers_show_more_for_that_row', (
    tester,
  ) async {
    await openJournal(tester, [for (var i = 0; i < 51; i++) entry(i)]);
    expect(byKey('ai-journal-row-50'), findsNothing);
    await tapVisible(tester, 'ai-journal-more');
    expect(byKey('ai-journal-row-50'), findsOneWidget);
    expect(byKey('ai-journal-more'), findsNothing);
  });

  testWidgets('long_outcome_is_ellipsized_and_keeps_the_left_column', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(420, 2400));
    await openJournal(tester, [
      entry(0, outcome: 'incompatible_state.ai_content_not_allowed'),
    ]);
    expect(tester.takeException(), isNull);
    final outcome = tester.widget<Text>(
      find.descendant(
        of: byKey('ai-journal-row-0'),
        matching: find.text(
          'Ce service d’IA n’est pas autorisé à recevoir ce type de contenu',
        ),
      ),
    );
    expect(outcome.overflow, TextOverflow.ellipsis);
    final details = tester.getSize(
      find.descendant(
        of: byKey('ai-journal-row-0'),
        matching: find.textContaining('caractères'),
      ),
    );
    expect(details.height, lessThan(20));
  });

  testWidgets('accent_color_marks_failures_only', (tester) async {
    await openJournal(tester, [
      entry(0, outcome: 'end_turn'),
      entry(1, outcome: 'max_tokens'),
      entry(2, outcome: 'cancelled'),
      entry(3, outcome: 'interrupted'),
      entry(4, outcome: 'server.ai_unavailable'),
    ]);
    final pal = tester.element(byKey('ai-journal-view')).pal;
    Color? colorAt(String text) => tester
        .widget<Text>(
          find.descendant(
            of: find.byType(Container),
            matching: find.text(text),
          ),
        )
        .style
        ?.color;
    for (final calm in [
      'Terminé',
      'Coupé : limite de longueur',
      'Annulé',
      'Interrompu',
    ]) {
      expect(colorAt(calm), pal.muted, reason: calm);
    }
    expect(colorAt('Service d’IA indisponible'), pal.accent);
  });
}
