import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'ai_settings_harness.dart';
import 'fake_ai_engine.dart';

/// Moteur factice qui propose le modèle local, et la page de réglages ouverte dessus.
Future<FakeAiEngine> openLocal(
  WidgetTester tester, {
  void Function(FakeAiEngine engine)? setup,
}) async {
  final engine = FakeAiEngine()
    ..providers.add(provider('local', 'Modèle local', local: true));
  setup?.call(engine);
  await pumpSettings(tester, using: engine);
  if (engine.local.supported) await tapKey(tester, 'ai-tab-local');
  return engine;
}

void main() {
  testWidgets('trois cartes de palier avec taille et licence', (tester) async {
    await openLocal(tester);
    for (final id in ['mini', 'light', 'big']) {
      expect(byKey('ai-local-card-$id'), findsOneWidget);
    }
    expect(find.text('Mini'), findsWidgets);
    expect(find.text('Léger'), findsWidgets);
    expect(find.text('Avancé'), findsWidgets);
    expect(find.textContaining('Apache-2.0'), findsWidgets);
    expect(find.textContaining('5,3 Go'), findsOneWidget);
  });

  testWidgets('l’onglet est masqué quand le moteur ne le prend pas en charge', (
    tester,
  ) async {
    await openLocal(tester, setup: (e) => e.local.supported = false);
    expect(byKey('ai-tab-local'), findsNothing);
  });

  testWidgets('ouvrir l’onglet n’appelle que l’état, jamais l’installation', (
    tester,
  ) async {
    final engine = await openLocal(tester);
    expect(engine.requestsOf('install_local_model'), isEmpty);
    expect(engine.requestsOf('get_local_runtime_status'), isNotEmpty);
    expect(engine.requestsOf('remove_local_model'), isEmpty);
  });

  testWidgets(
    'télécharger demande d’abord confirmation avec taille et licence',
    (tester) async {
      final engine = await openLocal(tester);
      await tapKey(tester, 'ai-local-install-light');
      expect(byKey('ai-local-confirm'), findsOneWidget);
      expect(
        find.descendant(
          of: byKey('ai-local-confirm'),
          matching: find.textContaining('2,1 Go'),
        ),
        findsOneWidget,
      );
      expect(
        find.descendant(
          of: byKey('ai-local-confirm'),
          matching: find.textContaining('Apache-2.0'),
        ),
        findsOneWidget,
      );
      await tapKey(tester, 'ai-local-confirm-cancel');
      expect(engine.requestsOf('install_local_model'), isEmpty);
      await tapKey(tester, 'ai-local-install-light');
      engine.local.script.add({'type': 'finished'});
      await tapKey(tester, 'ai-local-confirm-accept');
      expect(
        engine.requestsOf('install_local_model').single['model_id'],
        'light',
      );
    },
  );

  testWidgets('la barre suit l’opération puis la carte devient installée', (
    tester,
  ) async {
    final engine = await openLocal(tester);
    engine.local.script.addAll([
      {'type': 'running', 'done': 25, 'total': 100},
      {'type': 'running', 'done': 70, 'total': 100},
      {'type': 'finished'},
    ]);
    await tapKey(tester, 'ai-local-install-mini');
    await tester.tap(byKey('ai-local-confirm-accept'));
    await tester.pump();
    await tester.pump();
    LinearProgressIndicator bar() => tester.widget<LinearProgressIndicator>(
      find.byType(LinearProgressIndicator),
    );
    expect(bar().value, closeTo(0.25, 0.01));
    await tester.pump(const Duration(milliseconds: 600));
    expect(bar().value, closeTo(0.7, 0.01));
    await tester.pumpAndSettle(const Duration(milliseconds: 600));
    expect(byKey('ai-local-remove-mini'), findsOneWidget);
    expect(find.byType(LinearProgressIndicator), findsNothing);
  });

  testWidgets('l’avertissement mémoire ne bloque pas le téléchargement', (
    tester,
  ) async {
    await openLocal(tester, setup: (e) => e.local.memoryWarning.add('big'));
    expect(byKey('ai-local-memory-big'), findsOneWidget);
    expect(byKey('ai-local-memory-mini'), findsNothing);
    await tapKey(tester, 'ai-local-install-big');
    expect(byKey('ai-local-confirm'), findsOneWidget);
  });

  testWidgets('un modèle installé se retire et la mémoire se libère', (
    tester,
  ) async {
    final engine = await openLocal(
      tester,
      setup: (e) => e.local
        ..installed.add('mini')
        ..running = 'mini',
    );
    await tapKey(tester, 'ai-local-stop');
    expect(engine.requestsOf('stop_local_runtime'), hasLength(1));
    await tapKey(tester, 'ai-local-remove-mini');
    expect(engine.requestsOf('remove_local_model').single['model_id'], 'mini');
  });
}
