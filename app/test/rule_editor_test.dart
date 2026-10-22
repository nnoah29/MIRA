import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/rule_models.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/rule_editor.dart';

const catalog = RuleCatalog(
  [
    CatalogEntry(
      kind: 'sender',
      family: 'sender',
      fields: ['field', 'matcher'],
      needsMailbox: false,
      singleAccount: false,
    ),
    CatalogEntry(
      kind: 'has_attachment',
      family: 'content',
      fields: [],
      needsMailbox: false,
      singleAccount: false,
    ),
  ],
  [
    CatalogEntry(
      kind: 'mark_read',
      family: 'flags',
      fields: ['read'],
      needsMailbox: false,
      singleAccount: false,
    ),
    CatalogEntry(
      kind: 'move_to',
      family: 'filing',
      fields: ['mailbox'],
      needsMailbox: true,
      singleAccount: true,
    ),
  ],
);

Rule draft(RuleScope scope) => Rule(
  id: 'r1',
  name: 'Test',
  scope: scope,
  alternatives: [
    ConditionGroup([
      Condition(test: {'type': 'has_attachment'}),
    ]),
  ],
  actions: [
    RuleAction({'type': 'mark_read', 'read': true}),
  ],
);

Future<List<Rule>> pumpEditor(WidgetTester tester, Rule rule) async {
  final saved = <Rule>[];
  await tester.binding.setSurfaceSize(const Size(900, 1600));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: SingleChildScrollView(
          child: RuleEditor(
            rule: rule,
            catalog: catalog,
            accounts: const {'a1': 'a1@example.org', 'a2': 'a2@example.org'},
            onSave: (r) async => saved.add(r),
            onPreview: (r) async => const RulePreview(3, 40, ['m1', 'm2']),
          ),
        ),
      ),
    ),
  );
  return saved;
}

void main() {
  testWidgets('ajoute une alternative OU puis une condition ET', (t) async {
    final saved = await pumpEditor(t, draft(const RuleScope.all()));
    await t.tap(find.byKey(const ValueKey('add-alternative')));
    await t.pump();
    await t.tap(find.byKey(const ValueKey('add-condition-0')));
    await t.pump();
    await t.tap(find.byKey(const ValueKey('save-rule')));
    await t.pump();
    final json = saved.single.toJson();
    final groups = json['alternatives'] as List;
    expect(groups, hasLength(2));
    expect((groups[0]['conditions'] as List), hasLength(2));
    expect((groups[1]['conditions'] as List), hasLength(1));
  });

  testWidgets('bascule l\'inversion d\'une condition', (t) async {
    final saved = await pumpEditor(t, draft(const RuleScope.all()));
    await t.tap(find.byKey(const ValueKey('negate-0-0')));
    await t.pump();
    await t.tap(find.byKey(const ValueKey('save-rule')));
    await t.pump();
    expect(saved.single.alternatives[0].conditions[0].negated, isTrue);
  });

  testWidgets('actions de boîte masquées hors compte unique', (t) async {
    await pumpEditor(t, draft(const RuleScope.all()));
    await t.tap(find.byKey(const ValueKey('action-0-kind')));
    await t.pumpAndSettle();
    expect(
      find.byKey(const ValueKey('action-0-option-mark_read')),
      findsWidgets,
    );
    expect(find.byKey(const ValueKey('action-0-option-move_to')), findsNothing);
  });

  testWidgets('actions de boîte proposées pour un compte unique', (t) async {
    await pumpEditor(t, draft(const RuleScope.accounts(['a1'])));
    await t.tap(find.byKey(const ValueKey('action-0-kind')));
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('action-0-option-move_to')), findsWidgets);
  });

  testWidgets('l\'aperçu affiche le nombre de messages touchés', (t) async {
    await pumpEditor(t, draft(const RuleScope.all()));
    await t.tap(find.byKey(const ValueKey('preview-rule')));
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('preview-result')), findsOneWidget);
    expect(find.textContaining('3'), findsWidgets);
    expect(find.textContaining('m1'), findsWidgets);
  });
}
