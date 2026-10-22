import 'dart:async';

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
      kind: 'subject',
      family: 'content',
      fields: ['matcher'],
      needsMailbox: false,
      singleAccount: false,
    ),
    CatalogEntry(
      kind: 'preview',
      family: 'content',
      fields: ['matcher'],
      needsMailbox: false,
      singleAccount: false,
    ),
    CatalogEntry(
      kind: 'in_mailbox',
      family: 'mailbox',
      fields: ['mailbox'],
      needsMailbox: true,
      singleAccount: false,
    ),
    CatalogEntry(
      kind: 'from_account',
      family: 'account',
      fields: ['account'],
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
    CatalogEntry(
      kind: 'has_local_label',
      family: 'local_labels',
      fields: ['label'],
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
    CatalogEntry(
      kind: 'add_local_label',
      family: 'local_labels',
      fields: ['label'],
      needsMailbox: false,
      singleAccount: false,
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

Condition subjectContains(String text) => Condition(
  test: {
    'type': 'subject',
    'matcher': {
      'op': 'contains',
      'values': [text],
    },
  },
);

/// Textes affichés par tous les champs de saisie, dans l'ordre (nom de la règle d'abord).
List<String> fieldTexts(WidgetTester t) => [
  for (final e in t.widgetList<EditableText>(find.byType(EditableText)))
    e.controller.text,
];

Future<List<Rule>> pumpEditor(
  WidgetTester tester,
  Rule rule, {
  Future<void>? gate,
  List<RuleChoice> labels = const [],
}) async {
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
            mailboxes: const {
              'a1': [RuleChoice('mb-a1', 'Factures')],
              'a2': [RuleChoice('mb-a2', 'Perso')],
            },
            labels: labels,
            onSave: (r) async {
              saved.add(r);
              await gate;
            },
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
    expect(
      find.text('3 messages touchés sur 40 examinés, par exemple : m1, m2'),
      findsOneWidget,
    );
  });

  testWidgets(
    'retirer une condition puis changer de variante : le champ affiché est la donnée enregistrée',
    (t) async {
      final rule = draft(const RuleScope.all())
        ..alternatives = [
          ConditionGroup([subjectContains('facture'), subjectContains('bob')]),
        ];
      final saved = await pumpEditor(t, rule);
      expect(fieldTexts(t), ['Test', 'facture', 'bob']);
      await t.tap(find.byKey(const ValueKey('remove-condition-0-0')));
      await t.pump();
      expect(fieldTexts(t), ['Test', 'bob']);
      await t.tap(find.byKey(const ValueKey('condition-0-0-kind')));
      await t.pumpAndSettle();
      await t.tap(
        find.byKey(const ValueKey('condition-0-0-option-preview')).last,
      );
      await t.pumpAndSettle();
      expect(fieldTexts(t), ['Test', '']);
      await t.tap(find.byKey(const ValueKey('save-rule')));
      await t.pump();
      final test = saved.single.alternatives.single.conditions.single.test;
      expect(test['type'], 'preview');
      expect((test['matcher'] as Map)['values'], isEmpty);
    },
  );

  testWidgets('passer à plusieurs comptes retire les actions de boîte', (
    t,
  ) async {
    final rule = draft(const RuleScope.accounts(['a1']))
      ..actions.add(RuleAction({'type': 'move_to', 'mailbox': 'mb-a1'}));
    final saved = await pumpEditor(t, rule);
    await t.tap(find.byKey(const ValueKey('scope-account-a2')));
    await t.pump();
    await t.tap(find.byKey(const ValueKey('save-rule')));
    await t.pump();
    expect(saved.single.scope.accounts, ['a1', 'a2']);
    expect(saved.single.actions.map((a) => a.type), ['mark_read']);
  });

  testWidgets('un double clic sur « Enregistrer » n\'enregistre qu\'une fois', (
    t,
  ) async {
    final gate = Completer<void>();
    final saved = await pumpEditor(
      t,
      draft(const RuleScope.all()),
      gate: gate.future,
    );
    await t.tap(find.byKey(const ValueKey('save-rule')));
    await t.tap(find.byKey(const ValueKey('save-rule')));
    await t.pump();
    gate.complete();
    await t.pumpAndSettle();
    expect(saved, hasLength(1));
  });

  testWidgets(
    'une action de boîte propose les boîtes du seul compte de la portée',
    (t) async {
      final rule = draft(const RuleScope.accounts(['a1']))
        ..actions = [
          RuleAction({'type': 'move_to', 'mailbox': ''}),
        ];
      final saved = await pumpEditor(t, rule);
      await t.tap(find.byKey(const ValueKey('action-0-mailbox')));
      await t.pumpAndSettle();
      expect(find.text('Perso'), findsNothing);
      await t.tap(find.text('Factures').last);
      await t.pumpAndSettle();
      await t.tap(find.byKey(const ValueKey('save-rule')));
      await t.pump();
      expect(saved.single.actions.single.data['mailbox'], 'mb-a1');
    },
  );

  testWidgets(
    '« reçu sur le compte » et « dans la boîte » proposent les comptes de la portée',
    (t) async {
      final rule = draft(const RuleScope.all())
        ..alternatives = [
          ConditionGroup([
            Condition(test: {'type': 'from_account', 'account': ''}),
            Condition(test: {'type': 'in_mailbox', 'mailbox': ''}),
          ]),
        ];
      final saved = await pumpEditor(t, rule);
      await t.tap(find.byKey(const ValueKey('condition-0-0-account')));
      await t.pumpAndSettle();
      await t.tap(find.text('a2@example.org').last);
      await t.pumpAndSettle();
      await t.tap(find.byKey(const ValueKey('condition-0-1-mailbox')));
      await t.pumpAndSettle();
      expect(find.text('a1@example.org · Factures'), findsWidgets);
      await t.tap(find.text('a2@example.org · Perso').last);
      await t.pumpAndSettle();
      await t.tap(find.byKey(const ValueKey('save-rule')));
      await t.pump();
      final conditions = saved.single.alternatives.single.conditions;
      expect(conditions[0].test['account'], 'a2');
      expect(conditions[1].test['mailbox'], 'mb-a2');
    },
  );

  testWidgets(
    'un libellé local se choisit dans la liste, même avec tous les comptes',
    (t) async {
      final saved = await pumpEditor(
        t,
        draft(const RuleScope.all()),
        labels: const [RuleChoice('l1', 'À payer')],
      );
      await t.tap(find.byKey(const ValueKey('action-0-kind')));
      await t.pumpAndSettle();
      await t.tap(
        find.byKey(const ValueKey('action-0-option-add_local_label')),
      );
      await t.pumpAndSettle();
      await t.tap(find.byKey(const ValueKey('action-0-label')));
      await t.pumpAndSettle();
      await t.tap(find.text('À payer').last);
      await t.pumpAndSettle();
      await t.tap(find.byKey(const ValueKey('save-rule')));
      await t.pumpAndSettle();
      expect(saved.single.actions.single.data, {
        'type': 'add_local_label',
        'label': 'l1',
      });
    },
  );
}
