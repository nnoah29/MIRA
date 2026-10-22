import 'dart:math';

import '../data/rule_models.dart';
import 'engine_client.dart';

/// Règles « condition → action » : lecture, écritures idempotentes, validation, aperçu, catalogue.
class RuleAdmin {
  final EngineClient client;
  int _seq = 0;
  final String _run = Random().nextInt(1 << 32).toRadixString(36);
  RuleAdmin(this.client);

  String _key(String what) =>
      'ui-rule-$what-$_run-${DateTime.now().microsecondsSinceEpoch}-${_seq++}';

  Map<String, dynamic> _inner(Map<String, dynamic> response) =>
      ((response['data'] as Map)['data'] as Map).cast<String, dynamic>();

  /// Toutes les règles, dans l'ordre d'évaluation, avec leur statut.
  Future<List<RuleView>> list() async {
    final r = await client.call(req('rule', 'list_rules'));
    final items = ((r['data'] as Map)['data'] as List);
    return [
      for (final v in items)
        RuleView.fromJson((v as Map).cast<String, dynamic>()),
    ];
  }

  /// Crée une règle (le moteur choisit l'identifiant).
  Future<RuleView> create(Rule rule) => _write('create_rule', rule, 'create');

  /// Remplace une règle existante.
  Future<RuleView> update(Rule rule) => _write('update_rule', rule, 'update');

  Future<RuleView> _write(String op, Rule rule, String what) async {
    final r = await client.call(
      req('rule', op, {'rule': rule.toJson(), 'idempotency_key': _key(what)}),
    );
    return RuleView.fromJson(_inner(r));
  }

  /// Supprime une règle.
  Future<void> delete(String id) => client.call(
    req('rule', 'delete_rule', {'rule': id, 'idempotency_key': _key('delete')}),
  );

  /// Active ou désactive une règle.
  Future<void> setEnabled(String id, bool enabled) => client.call(
    req('rule', 'set_rule_enabled', {
      'rule': id,
      'enabled': enabled,
      'idempotency_key': _key('enable'),
    }),
  );

  /// Fixe l'ordre d'évaluation des règles.
  Future<void> reorder(List<String> order) => client.call(
    req('rule', 'reorder_rules', {
      'order': order,
      'idempotency_key': _key('reorder'),
    }),
  );

  /// Statut d'une règle en cours d'édition, sans l'enregistrer.
  Future<RuleStatus> validate(Rule rule) async {
    final r = await client.call(
      req('rule', 'validate_rule', {'rule': rule.toJson()}),
    );
    return RuleStatus.fromJson(_inner(r));
  }

  /// Combien de messages existants la règle toucherait, avec quelques exemples.
  Future<RulePreview> preview(Rule rule, {int limit = 5}) async {
    final r = await client.call(
      req('rule', 'preview_rule', {'rule': rule.toJson(), 'limit': limit}),
    );
    return RulePreview.fromJson(_inner(r));
  }

  /// Conditions et actions connues du moteur.
  Future<RuleCatalog> catalog() async {
    final r = await client.call(req('rule', 'list_rule_catalog'));
    return RuleCatalog.fromJson(_inner(r));
  }
}
