import 'package:flutter_test/flutter_test.dart';

import 'ai_settings_harness.dart';

void main() {
  testWidgets('tabs_list_the_engine_providers_in_order', (tester) async {
    await pumpSettings(tester);
    final xs = [
      for (final id in ['openai', 'anthropic', 'generic'])
        tester.getTopLeft(byKey('ai-tab-$id')).dx,
    ];
    expect(xs, orderedEquals([...xs]..sort()));
    expect(xs.toSet(), hasLength(3));
    Finder label(String id, String text) =>
        find.descendant(of: byKey('ai-tab-$id'), matching: find.text(text));
    expect(label('openai', 'OPENAI'), findsOneWidget);
    expect(label('anthropic', 'ANTHROPIC'), findsOneWidget);
    expect(label('generic', 'AUTRE (COMPATIBLE OPENAI)'), findsOneWidget);
  });

  testWidgets('a_connected_provider_shows_a_dot_and_its_card', (tester) async {
    await pumpSettings(tester, connections: [connection('ai-1')]);
    expect(byKey('ai-tab-dot-openai'), findsOneWidget);
    expect(byKey('ai-tab-dot-anthropic'), findsNothing);
    expect(byKey('ai-card-ai-1'), findsOneWidget);
  });

  testWidgets('the_default_connection_tab_is_marked_and_selected', (
    tester,
  ) async {
    await pumpSettings(
      tester,
      connections: [
        connection('ai-1'),
        connection('ai-2', provider: 'anthropic', isDefault: true),
      ],
    );
    expect(byKey('ai-tab-default-anthropic'), findsOneWidget);
    expect(byKey('ai-tab-default-openai'), findsNothing);
    expect(byKey('ai-tab-dot-openai'), findsOneWidget);
    expect(byKey('ai-card-ai-2'), findsOneWidget);
    expect(byKey('ai-card-ai-1'), findsNothing);
  });

  testWidgets('first_tab_is_selected_when_there_is_no_default', (tester) async {
    await pumpSettings(tester);
    expect(byKey('ai-add-openai'), findsOneWidget);
    expect(byKey('ai-add-anthropic'), findsNothing);
  });

  testWidgets('tapping_a_tab_shows_that_provider_and_keeps_it', (tester) async {
    await pumpSettings(
      tester,
      connections: [
        connection('ai-1', isDefault: true),
        connection('ai-2', provider: 'anthropic'),
      ],
    );
    await tapKey(tester, 'ai-tab-anthropic');
    expect(byKey('ai-card-ai-2'), findsOneWidget);
    expect(byKey('ai-card-ai-1'), findsNothing);
    await tapKey(tester, 'ai-default-ai-2');
    expect(byKey('ai-card-ai-2'), findsOneWidget);
  });

  testWidgets('an_unconnected_tab_adds_a_connection_and_reveals_its_card', (
    tester,
  ) async {
    final engine = await pumpSettings(tester);
    await tapKey(tester, 'ai-tab-anthropic');
    expect(byKey('ai-card-ai-0'), findsNothing);
    expect(find.textContaining('Anthropic'), findsWidgets);
    await tapKey(tester, 'ai-add-anthropic');
    expect(
      engine.requestsOf('add_ai_connection').single['provider'],
      'anthropic',
    );
    expect(byKey('ai-card-ai-0'), findsOneWidget);
    expect(byKey('ai-add-anthropic'), findsNothing);
    expect(byKey('ai-tab-default-anthropic'), findsOneWidget);
  });

  testWidgets('single_connection_providers_hide_add_once_connected', (
    tester,
  ) async {
    await pumpSettings(tester, connections: [connection('ai-1')]);
    expect(byKey('ai-add-openai'), findsNothing);
    expect(byKey('ai-card-ai-1'), findsOneWidget);
  });

  testWidgets('the_generic_tab_lists_several_connections_and_keeps_add', (
    tester,
  ) async {
    final engine = await pumpSettings(
      tester,
      connections: [
        connection('ai-1', provider: 'generic', baseUrl: 'http://a/v1'),
        connection('ai-2', provider: 'generic', baseUrl: 'http://b/v1'),
      ],
    );
    await tapKey(tester, 'ai-tab-generic');
    expect(byKey('ai-card-ai-1'), findsOneWidget);
    expect(byKey('ai-card-ai-2'), findsOneWidget);
    await tapKey(tester, 'ai-add-generic');
    await tester.enterText(byKey('ai-new-url'), 'http://c/v1');
    await tapKey(tester, 'ai-new-save');
    expect(
      engine.requestsOf('add_ai_connection').single['base_url'],
      'http://c/v1',
    );
    expect(byKey('ai-card-ai-0'), findsOneWidget);
    expect(byKey('ai-add-generic'), findsOneWidget);
  });
}
