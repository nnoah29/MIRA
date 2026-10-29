import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/theme/app_theme.dart';

import 'ai_settings_harness.dart';
import 'fake_ai_engine.dart';

const bestEffort = 'Détection au mieux : elle peut se tromper.';
const sentAsIs = 'Les données sensibles sont envoyées telles quelles.';
const localOnly = 'Connexion locale : rien ne quitte cet appareil.';
const nothingMasked = 'Aucune catégorie : rien n’est masqué.';
const maskedBefore = 'Masquées avant l’envoi';

Map<String, dynamic> masking(String mode, [List<String>? categories]) => {
  'mode': mode,
  'categories': categories ?? ['iban', 'card', 'secret', 'otp'],
};

class GatedEngine extends FakeAiEngine {
  final String op;
  final gate = Completer<void>();
  bool held = false;
  GatedEngine(this.op);

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    if ((request['request'] as Map)['op'] == op) {
      held = true;
      await gate.future;
    }
    return super.call(request);
  }
}

Color colorOf(WidgetTester tester, String text) =>
    tester.widget<Text>(find.text(text)).style!.color!;

Pal palOf(WidgetTester tester) => tester.element(byKey('ai-privacy-ai-0')).pal;

void main() {
  testWidgets('masking_off_on_a_remote_connection_warns_in_accent', (
    tester,
  ) async {
    await pumpSettings(
      tester,
      connections: [connection('ai-0', hasKey: true, masking: masking('off'))],
    );
    expect(find.text(sentAsIs), findsOneWidget);
    expect(colorOf(tester, sentAsIs), palOf(tester).accent);
    expect(find.text(bestEffort), findsNothing);
    expect(find.text(localOnly), findsNothing);
    expect(find.text(maskedBefore), findsNothing);
  });

  testWidgets('masking_off_on_a_loopback_connection_says_nothing_leaves', (
    tester,
  ) async {
    await pumpSettings(
      tester,
      connections: [
        connection(
          'ai-0',
          provider: 'generic',
          isDefault: true,
          baseUrl: 'http://127.0.0.1:11434/v1',
          masking: masking('off'),
        ),
      ],
    );
    expect(find.text(localOnly), findsOneWidget);
    expect(colorOf(tester, localOnly), palOf(tester).dim);
    expect(find.text(sentAsIs), findsNothing);
    expect(find.text(bestEffort), findsNothing);
  });

  testWidgets('masking_on_shows_the_best_effort_notice_only', (tester) async {
    await pumpSettings(tester, connections: [connection('ai-0', hasKey: true)]);
    expect(find.text(bestEffort), findsOneWidget);
    expect(find.text(maskedBefore), findsOneWidget);
    expect(find.text(sentAsIs), findsNothing);
    expect(find.text(localOnly), findsNothing);
    expect(find.text(nothingMasked), findsNothing);
  });

  testWidgets('masking_on_without_category_says_nothing_is_masked', (
    tester,
  ) async {
    await pumpSettings(
      tester,
      connections: [
        connection('ai-0', hasKey: true, masking: masking('mask', [])),
      ],
    );
    expect(find.text(nothingMasked), findsOneWidget);
    expect(find.text(maskedBefore), findsNothing);
    expect(find.text(bestEffort), findsNothing);
  });

  testWidgets('section_dims_only_for_its_own_saves', (tester) async {
    final engine = GatedEngine('test_ai_connection');
    await pumpSettings(
      tester,
      using: engine,
      connections: [connection('ai-0', hasKey: true)],
    );
    bool absorbing() =>
        tester.widget<AbsorbPointer>(byKey('ai-privacy-ai-0')).absorbing;
    await tester.tap(byKey('ai-test-ai-0'));
    await tester.pump();
    expect(engine.held, isTrue);
    expect(absorbing(), isFalse);
    engine.gate.complete();
    await tester.pumpAndSettle();
  });

  testWidgets('section_dims_while_its_own_save_runs', (tester) async {
    final engine = GatedEngine('set_ai_masking');
    await pumpSettings(
      tester,
      using: engine,
      connections: [connection('ai-0', hasKey: true)],
    );
    await tester.tap(byKey('ai-privacy-mask-ai-0'));
    await tester.pump();
    expect(engine.held, isTrue);
    expect(
      tester.widget<AbsorbPointer>(byKey('ai-privacy-ai-0')).absorbing,
      isTrue,
    );
    engine.gate.complete();
    await tester.pumpAndSettle();
  });

  testWidgets('failed_save_shows_its_error_inside_the_section', (tester) async {
    final engine = FakeAiEngine()
      ..failOpName = 'set_ai_masking'
      ..failOp = const EngineException('server.ai_unavailable', '');
    await pumpSettings(
      tester,
      using: engine,
      connections: [connection('ai-0', hasKey: true)],
    );
    await tapKey(tester, 'ai-privacy-mask-ai-0');
    final inside = find.descendant(
      of: byKey('ai-privacy-ai-0'),
      matching: find.text('Service d’IA indisponible'),
    );
    expect(inside, findsOneWidget);
    expect(byKey('ai-error'), findsOneWidget);
    engine.failOp = null;
    await tapKey(tester, 'ai-privacy-mask-ai-0');
    expect(inside, findsNothing);
  });
}
