import 'dart:async';

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/rule_admin.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/rule_editor.dart';
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
    if (op == 'list_mailboxes') {
      return {
        'type': 'mailboxes',
        'data': [
          {'id': 'mb-1', 'name': 'Factures', 'display_name': 'Factures'},
        ],
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

List<Map> requestsOf(FakeRulesEngine engine, String op) => [
  for (final c in engine.calls)
    if ((c['request'] as Map)['op'] == op) c['request'] as Map,
];

Future<FakeRulesEngine> pumpList(
  WidgetTester tester, {
  Stream<void> changes = const Stream.empty(),
}) async {
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
            changes: changes,
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
    final call = requestsOf(engine, 'delete_rule').single;
    expect(call['rule'], 'a');
    expect(call['idempotency_key'], isNotEmpty);
  });

  testWidgets('un changement de règle côté moteur relit la liste', (t) async {
    final changes = StreamController<void>.broadcast();
    addTearDown(changes.close);
    final engine = await pumpList(t, changes: changes.stream);
    final before = requestsOf(engine, 'list_rules').length;
    changes.add(null);
    await t.pumpAndSettle();
    expect(requestsOf(engine, 'list_rules'), hasLength(before + 1));
  });

  testWidgets('glisser-déposer envoie reorder_rules avec le nouvel ordre', (
    t,
  ) async {
    final engine = await pumpList(t);
    final handle = find.byIcon(Icons.drag_handle).first;
    final gesture = await t.startGesture(t.getCenter(handle));
    await t.pump(kLongPressTimeout);
    for (var i = 0; i < 10; i++) {
      await gesture.moveBy(const Offset(0, 15));
      await t.pump(const Duration(milliseconds: 16));
    }
    await gesture.up();
    await t.pumpAndSettle();
    final call = requestsOf(engine, 'reorder_rules').single;
    expect(call['order'], ['b', 'a']);
    expect(call['idempotency_key'], isNotEmpty);
  });

  testWidgets('les boîtes de chaque compte sont lues pour l\'éditeur', (
    t,
  ) async {
    final engine = await pumpList(t);
    final asked = requestsOf(engine, 'list_mailboxes');
    expect(asked.map((r) => r['account']), ['a1']);
    await t.tap(find.text('Règle a'));
    await t.pumpAndSettle();
    final editor = t.widget<RuleEditor>(find.byType(RuleEditor));
    expect(editor.mailboxes['a1']!.single.label, 'Factures');
  });
}
