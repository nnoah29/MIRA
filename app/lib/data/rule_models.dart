/// Portée d'une règle : tous les comptes, ou une liste de comptes.
class RuleScope {
  final List<String>? accounts;
  const RuleScope.all() : accounts = null;
  const RuleScope.accounts(List<String> this.accounts);

  bool get isAll => accounts == null;
  bool get isSingleAccount => accounts != null && accounts!.length == 1;

  factory RuleScope.fromJson(Map<String, dynamic> j) => j['type'] == 'accounts'
      ? RuleScope.accounts([for (final a in (j['accounts'] as List)) '$a'])
      : const RuleScope.all();

  Map<String, dynamic> toJson() => accounts == null
      ? {'type': 'all_accounts'}
      : {'type': 'accounts', 'accounts': List<String>.of(accounts!)};
}

/// Une condition : un test du moteur (gardé tel quel) éventuellement inversé.
class Condition {
  bool negated;
  Map<String, dynamic> test;
  Condition({this.negated = false, required this.test});

  factory Condition.fromJson(Map<String, dynamic> j) => Condition(
    negated: j['negated'] == true,
    test: _deepCopy((j['test'] as Map).cast<String, dynamic>()),
  );

  Map<String, dynamic> toJson() => {
    'negated': negated,
    'test': _deepCopy(test),
  };
}

/// Un bloc de conditions liées par ET ; les blocs d'une règle sont liés par OU.
class ConditionGroup {
  List<Condition> conditions;
  ConditionGroup(this.conditions);

  factory ConditionGroup.fromJson(Map<String, dynamic> j) => ConditionGroup([
    for (final c in (j['conditions'] as List))
      Condition.fromJson((c as Map).cast<String, dynamic>()),
  ]);

  Map<String, dynamic> toJson() => {
    'conditions': [for (final c in conditions) c.toJson()],
  };
}

/// Une action du moteur, gardée telle quelle (`type` + paramètres).
class RuleAction {
  Map<String, dynamic> data;
  RuleAction(this.data);

  String get type => '${data['type']}';

  factory RuleAction.fromJson(Map<String, dynamic> j) =>
      RuleAction(_deepCopy(j));

  Map<String, dynamic> toJson() => _deepCopy(data);
}

/// Une règle « conditions → actions ».
class Rule {
  String id;
  String name;
  bool enabled;
  RuleScope scope;
  List<ConditionGroup> alternatives;
  List<RuleAction> actions;
  int schemaVersion;

  Rule({
    required this.id,
    required this.name,
    this.enabled = true,
    this.scope = const RuleScope.all(),
    List<ConditionGroup>? alternatives,
    List<RuleAction>? actions,
    this.schemaVersion = 1,
  }) : alternatives = alternatives ?? [],
       actions = actions ?? [];

  factory Rule.fromJson(Map<String, dynamic> j) => Rule(
    id: '${j['id']}',
    name: '${j['name']}',
    enabled: j['enabled'] != false,
    scope: RuleScope.fromJson((j['scope'] as Map).cast<String, dynamic>()),
    alternatives: [
      for (final g in (j['alternatives'] as List))
        ConditionGroup.fromJson((g as Map).cast<String, dynamic>()),
    ],
    actions: [
      for (final a in (j['actions'] as List))
        RuleAction.fromJson((a as Map).cast<String, dynamic>()),
    ],
    schemaVersion: (j['schema_version'] as num?)?.toInt() ?? 1,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'enabled': enabled,
    'scope': scope.toJson(),
    'alternatives': [for (final g in alternatives) g.toJson()],
    'actions': [for (final a in actions) a.toJson()],
    'schema_version': schemaVersion,
  };
}

/// Statut calculé à la lecture : valide, ou invalide avec la raison.
class RuleStatus {
  final bool valid;
  final String? reason;
  const RuleStatus.valid() : valid = true, reason = null;
  const RuleStatus.invalid(String this.reason) : valid = false;

  factory RuleStatus.fromJson(Map<String, dynamic> j) => j['type'] == 'invalid'
      ? RuleStatus.invalid('${j['reason']}')
      : const RuleStatus.valid();

  Map<String, dynamic> toJson() =>
      valid ? {'type': 'valid'} : {'type': 'invalid', 'reason': reason};
}

/// Une règle lue avec son statut.
class RuleView {
  final Rule rule;
  final RuleStatus status;
  const RuleView(this.rule, this.status);

  factory RuleView.fromJson(Map<String, dynamic> j) => RuleView(
    Rule.fromJson((j['rule'] as Map).cast<String, dynamic>()),
    RuleStatus.fromJson((j['status'] as Map).cast<String, dynamic>()),
  );

  Map<String, dynamic> toJson() => {
    'rule': rule.toJson(),
    'status': status.toJson(),
  };
}

/// Aperçu : messages touchés parmi les examinés, et quelques exemples.
class RulePreview {
  final int matched, scanned;
  final List<String> samples;
  const RulePreview(this.matched, this.scanned, this.samples);

  factory RulePreview.fromJson(Map<String, dynamic> j) => RulePreview(
    (j['matched'] as num).toInt(),
    (j['scanned'] as num).toInt(),
    [for (final s in (j['samples'] as List)) '$s'],
  );

  Map<String, dynamic> toJson() => {
    'matched': matched,
    'scanned': scanned,
    'samples': samples,
  };
}

/// Une variante de condition ou d'action du catalogue du moteur.
class CatalogEntry {
  final String kind, family;
  final List<String> fields;
  final bool needsMailbox, singleAccount;
  const CatalogEntry({
    required this.kind,
    required this.family,
    required this.fields,
    required this.needsMailbox,
    required this.singleAccount,
  });

  factory CatalogEntry.fromJson(Map<String, dynamic> j) => CatalogEntry(
    kind: '${j['kind']}',
    family: '${j['family']}',
    fields: [for (final f in (j['fields'] as List)) '$f'],
    needsMailbox: j['needs_mailbox'] == true,
    singleAccount: j['single_account'] == true,
  );

  Map<String, dynamic> toJson() => {
    'kind': kind,
    'family': family,
    'fields': fields,
    'needs_mailbox': needsMailbox,
    'single_account': singleAccount,
  };
}

/// Un choix proposé pour un paramètre opaque (boîte ou compte) : identifiant et libellé lisible.
class RuleChoice {
  final String id, label;
  const RuleChoice(this.id, this.label);
}

/// Conditions et actions connues du moteur.
class RuleCatalog {
  final List<CatalogEntry> conditions, actions;
  const RuleCatalog(this.conditions, this.actions);

  factory RuleCatalog.fromJson(Map<String, dynamic> j) => RuleCatalog(
    [
      for (final c in (j['conditions'] as List))
        CatalogEntry.fromJson((c as Map).cast<String, dynamic>()),
    ],
    [
      for (final c in (j['actions'] as List))
        CatalogEntry.fromJson((c as Map).cast<String, dynamic>()),
    ],
  );

  Map<String, dynamic> toJson() => {
    'conditions': [for (final c in conditions) c.toJson()],
    'actions': [for (final c in actions) c.toJson()],
  };
}

dynamic _copyValue(dynamic v) => v is Map
    ? {for (final e in v.entries) '${e.key}': _copyValue(e.value)}
    : v is List
    ? [for (final x in v) _copyValue(x)]
    : v;

Map<String, dynamic> _deepCopy(Map<String, dynamic> m) =>
    _copyValue(m) as Map<String, dynamic>;
