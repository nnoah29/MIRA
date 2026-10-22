import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/rule_admin.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/rules_settings.dart';

Map<String, dynamic> view(String id, bool enabled, Map<String, dynamic> st) => {
  'rule': {
    'id': id,
    'name': 'Règle $id',
    'enabled': enabled,
    'scope': {'type': 'all_accounts'},
    'alternatives': [],
    'actions': [
      {'type': 'archive'},
    ],
    'schema_version': 1,
  },
  'status': st,
};

class FakeRulesEngine implements EngineClient {
  final calls = <Map<String, dynamic>>[];

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    calls.add(request);
    final op = (request['request'] as Map)['op'];
    if (op == 'list_rules') {
      return {
        'type': 'rule',
        'data': {
          'type': 'rules',
          'data': [
            view('a', true, {'type': 'valid'}),
            view('b', false, {'type': 'invalid', 'reason': 'portée vide'}),
          ],
        },
      };
    }
    if (op == 'list_rule_catalog') {
      return {
        'type': 'rule',
        'data': {
          'type': 'catalog',
          'data': {'conditions': [], 'actions': []},
        },
      };
    }
    return {'type': 'done'};
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();

  @override
  Future<String?> originalHtml(String messageId) async => null;

  @override
  Future<void> close() async {}
}

Future<FakeRulesEngine> pumpList(WidgetTester tester) async {
  final engine = FakeRulesEngine();
  await tester.binding.setSurfaceSize(const Size(900, 1600));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: SingleChildScrollView(
          child: RulesSettings(
            admin: RuleAdmin(engine),
            accounts: const {'a1': 'a1@example.org'},
            changes: const Stream.empty(),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return engine;
}

void main() {
  testWidgets('liste les règles avec le badge invalide et sa raison', (
    t,
  ) async {
    await pumpList(t);
    expect(find.text('Règle a'), findsOneWidget);
    expect(find.text('Règle b'), findsOneWidget);
    expect(find.byKey(const ValueKey('rule-invalid-a')), findsNothing);
    expect(find.byKey(const ValueKey('rule-invalid-b')), findsOneWidget);
    expect(find.textContaining('portée vide'), findsOneWidget);
  });

  testWidgets('l\'interrupteur appelle set_rule_enabled', (t) async {
    final engine = await pumpList(t);
    await t.tap(find.byKey(const ValueKey('rule-enabled-b')));
    await t.pumpAndSettle();
    final call = engine.calls
        .map((c) => c['request'] as Map)
        .firstWhere((r) => r['op'] == 'set_rule_enabled');
    expect(call['rule'], 'b');
    expect(call['enabled'], true);
    expect(call['idempotency_key'], isNotEmpty);
  });

  testWidgets('la suppression appelle delete_rule', (t) async {
    final engine = await pumpList(t);
    await t.tap(find.byKey(const ValueKey('rule-delete-a')));
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('confirm-delete-rule')));
    await t.pumpAndSettle();
    final ops = engine.calls.map((c) => (c['request'] as Map)['op']);
    expect(ops, contains('delete_rule'));
  });
}
