import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/rule_models.dart';

Map<String, dynamic> fullRule(Map<String, dynamic> scope) => {
  'id': 'rule-1',
  'name': 'Lettres',
  'enabled': true,
  'scope': scope,
  'alternatives': [
    {
      'conditions': [
        {
          'negated': false,
          'test': {
            'type': 'sender',
            'field': 'domain',
            'matcher': {
              'op': 'equals',
              'values': ['example.org'],
            },
          },
        },
        {
          'negated': true,
          'test': {'type': 'has_attachment'},
        },
      ],
    },
    {
      'conditions': [
        {
          'negated': false,
          'test': {'type': 'size_compare', 'cmp': 'greater', 'bytes': 1000},
        },
      ],
    },
  ],
  'actions': [
    {'type': 'mark_read', 'read': true},
    {'type': 'move_to', 'mailbox': 'mb-1'},
    {'type': 'suppress_notification'},
    {'type': 'reply_with_template', 'template': 'tpl-1'},
  ],
  'schema_version': 1,
};

void main() {
  test('règle complète : aller-retour exact, portée comptes', () {
    final json = fullRule({
      'type': 'accounts',
      'accounts': ['account-1'],
    });
    final rule = Rule.fromJson(json);
    expect(rule.scope.isSingleAccount, isTrue);
    expect(rule.alternatives, hasLength(2));
    expect(rule.alternatives[0].conditions[1].negated, isTrue);
    expect(rule.actions[1].type, 'move_to');
    expect(rule.toJson(), json);
  });

  test('portée tous les comptes : aller-retour', () {
    final json = fullRule({'type': 'all_accounts'});
    final rule = Rule.fromJson(json);
    expect(rule.scope.isAll, isTrue);
    expect(rule.scope.isSingleAccount, isFalse);
    expect(rule.toJson(), json);
  });

  test('statuts valide et invalide', () {
    final ok = RuleStatus.fromJson({'type': 'valid'});
    final bad = RuleStatus.fromJson({'type': 'invalid', 'reason': 'vide'});
    expect(ok.valid, isTrue);
    expect(bad.valid, isFalse);
    expect(bad.reason, 'vide');
    expect(bad.toJson(), {'type': 'invalid', 'reason': 'vide'});
    expect(ok.toJson(), {'type': 'valid'});
  });

  test('RuleView, aperçu et catalogue : aller-retour', () {
    final view = {
      'rule': fullRule({'type': 'all_accounts'}),
      'status': {'type': 'invalid', 'reason': 'portée vide'},
    };
    expect(RuleView.fromJson(view).toJson(), view);
    final preview = {
      'matched': 2,
      'scanned': 40,
      'samples': ['message-1'],
    };
    expect(RulePreview.fromJson(preview).toJson(), preview);
    final catalog = {
      'conditions': [
        {
          'kind': 'sender',
          'family': 'sender',
          'fields': ['field', 'matcher'],
          'needs_mailbox': false,
          'single_account': false,
        },
      ],
      'actions': [
        {
          'kind': 'move_to',
          'family': 'filing',
          'fields': ['mailbox'],
          'needs_mailbox': true,
          'single_account': true,
        },
      ],
    };
    final parsed = RuleCatalog.fromJson(catalog);
    expect(parsed.actions.single.needsMailbox, isTrue);
    expect(parsed.toJson(), catalog);
  });

  test('la copie est profonde : modifier le modèle ne touche pas le JSON', () {
    final json = fullRule({'type': 'all_accounts'});
    final rule = Rule.fromJson(json);
    rule.alternatives[0].conditions[0].test['field'] = 'name';
    expect(
      (json['alternatives'] as List)[0]['conditions'][0]['test']['field'],
      'domain',
    );
  });
}
