import 'package:flutter_test/flutter_test.dart';

import 'package:flutter/material.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/memory_settings.dart';
import 'package:mira/ui/overlays.dart';

import 'ai_settings_harness.dart' show byKey, connection, tapKey;
import 'fake_ai_engine.dart';

Map<String, dynamic> row(String id, String text, int at) => {
  'id': id,
  'text': text,
  'kind': 'fact',
  'source': 'user',
  'created_at': at,
  'model_id': 'embed',
};

Future<FakeAiEngine> open(
  WidgetTester tester, {
  bool enabled = true,
  void Function(FakeAiEngine e)? setup,
}) async {
  final engine = FakeAiEngine();
  engine.memory.settings = {
    'enabled': enabled,
    'embedder': {'kind': 'local'},
  };
  engine.memory.records
    ..add(row('a', 'CHAT-NOIR', 1))
    ..add(row('b', 'CHIEN-GRIS', 2));
  setup?.call(engine);
  engine.connections.add(connection('c1', label: 'MON-SERVICE', hasKey: true));
  await tester.binding.setSurfaceSize(const Size(900, 2400));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  addTearDown(Pops.close);
  await tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: SingleChildScrollView(
          child: MemorySettings(admin: AiAdmin(engine)),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return engine;
}

void main() {
  testWidgets('la section dit l’état et ne lance rien à l’ouverture', (
    tester,
  ) async {
    final engine = await open(tester, enabled: false);
    expect(byKey('ai-memory-section'), findsOneWidget);
    expect(byKey('ai-memory-toggle'), findsOneWidget);
    expect(engine.requestsOf('install_memory_model'), isEmpty);
    expect(engine.requestsOf('search_memories'), isEmpty);
    expect(engine.requestsOf('remember_memory'), isEmpty);
    expect(find.textContaining('2 souvenirs'), findsOneWidget);
  });

  testWidgets('activer enregistre le réglage', (tester) async {
    final engine = await open(tester, enabled: false);
    await tester.tap(
      find.descendant(
        of: byKey('ai-memory-toggle'),
        matching: find.text('ACTIVÉE'),
      ),
    );
    await tester.pumpAndSettle();
    final sent = engine.requestsOf('set_memory_settings').last;
    expect((sent['settings'] as Map)['enabled'], true);
  });

  testWidgets('modèle local absent : bouton d’installation, jamais seul', (
    tester,
  ) async {
    final engine = await open(
      tester,
      setup: (e) => e.memory.modelInstalled = false,
    );
    expect(byKey('ai-memory-install'), findsOneWidget);
    expect(engine.requestsOf('install_memory_model'), isEmpty);
    await tapKey(tester, 'ai-memory-install');
    expect(engine.requestsOf('install_memory_model'), hasLength(1));
  });

  testWidgets('modèle local présent : pas de bouton d’installation', (
    tester,
  ) async {
    await open(tester);
    expect(byKey('ai-memory-install'), findsNothing);
  });

  testWidgets('le mode fournisseur avertit que les souvenirs sortent', (
    tester,
  ) async {
    await open(tester);
    expect(byKey('ai-memory-privacy-note'), findsNothing);
    await tapKey(tester, 'ai-memory-provider-mode');
    expect(byKey('ai-memory-privacy-note'), findsOneWidget);
    expect(byKey('ai-memory-connection-c1'), findsOneWidget);
    expect(find.text('MON-SERVICE'), findsWidgets);
  });

  testWidgets('choisir une connexion et un modèle les enregistre', (
    tester,
  ) async {
    final engine = await open(tester);
    await tapKey(tester, 'ai-memory-provider-mode');
    await tapKey(tester, 'ai-memory-connection-c1');
    await tester.enterText(byKey('ai-memory-model-field'), 'embed-1');
    await tester.pump();
    await tapKey(tester, 'ai-memory-model-save');
    final embedder =
        (engine.requestsOf('set_memory_settings').last['settings']
                as Map)['embedder']
            as Map;
    expect(embedder, {
      'kind': 'provider',
      'connection': 'c1',
      'model': 'embed-1',
    });
  });

  testWidgets('la liste montre les souvenirs et la recherche les filtre', (
    tester,
  ) async {
    await open(tester);
    await tapKey(tester, 'ai-memory-show-list');
    expect(find.text('CHAT-NOIR'), findsOneWidget);
    expect(find.text('CHIEN-GRIS'), findsOneWidget);
    await tester.enterText(byKey('ai-memory-search-field'), 'chat');
    await tester.testTextInput.receiveAction(TextInputAction.search);
    await tester.pumpAndSettle();
    expect(find.text('CHAT-NOIR'), findsOneWidget);
    expect(find.text('CHIEN-GRIS'), findsNothing);
  });

  testWidgets('supprimer un souvenir demande confirmation puis l’oublie', (
    tester,
  ) async {
    final engine = await open(tester);
    await tapKey(tester, 'ai-memory-show-list');
    await tapKey(tester, 'ai-memory-delete-a');
    expect(engine.requestsOf('forget_memory'), isEmpty);
    await tapKey(tester, 'confirm-forget-memory');
    expect(engine.requestsOf('forget_memory').single['id'], 'a');
    expect(find.text('CHAT-NOIR'), findsNothing);
  });

  testWidgets('modifier un souvenir enregistre le nouveau texte', (
    tester,
  ) async {
    final engine = await open(tester);
    await tapKey(tester, 'ai-memory-show-list');
    await tapKey(tester, 'ai-memory-edit-a');
    await tester.enterText(byKey('ai-memory-edit-field'), 'CHAT-BLANC');
    await tapKey(tester, 'confirm-edit-memory');
    expect(engine.requestsOf('update_memory').single['text'], 'CHAT-BLANC');
    expect(find.text('CHAT-BLANC'), findsOneWidget);
  });

  testWidgets('tout oublier demande confirmation puis vide la liste', (
    tester,
  ) async {
    final engine = await open(tester);
    await tapKey(tester, 'ai-memory-forget-all');
    expect(engine.requestsOf('forget_all_memories'), isEmpty);
    await tapKey(tester, 'confirm-forget-all-memories');
    expect(engine.requestsOf('forget_all_memories'), hasLength(1));
    expect(engine.memory.records, isEmpty);
  });

  testWidgets('réindexer n’apparaît que s’il y a des souvenirs périmés', (
    tester,
  ) async {
    await open(tester);
    expect(byKey('ai-memory-reindex'), findsNothing);
  });

  testWidgets('des souvenirs périmés font apparaître la réindexation', (
    tester,
  ) async {
    final engine = await open(tester, setup: (e) => e.memory.stale = 2);
    await tapKey(tester, 'ai-memory-reindex');
    expect(engine.requestsOf('reindex_memories'), hasLength(1));
    expect(byKey('ai-memory-reindex'), findsNothing);
  });
}
