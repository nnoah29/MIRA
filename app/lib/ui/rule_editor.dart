import 'package:flutter/material.dart';

import '../data/rule_models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'rule_condition_row.dart';
import 'rule_labels.dart';
import 'rule_widgets.dart';
import 'widgets.dart';

/// Éditeur d'une règle : nom, portée, blocs « ET » séparés par « OU », actions, aperçu.
class RuleEditor extends StatefulWidget {
  final Rule rule;
  final RuleCatalog catalog;
  final Map<String, String> accounts;
  final Map<String, List<RuleChoice>> mailboxes;
  final Future<void> Function(Rule) onSave;
  final Future<RulePreview> Function(Rule) onPreview;
  final VoidCallback? onCancel;
  const RuleEditor({
    super.key,
    required this.rule,
    required this.catalog,
    required this.accounts,
    this.mailboxes = const {},
    required this.onSave,
    required this.onPreview,
    this.onCancel,
  });

  @override
  State<RuleEditor> createState() => _RuleEditorState();
}

class _RuleEditorState extends State<RuleEditor> {
  late final Rule rule = Rule.fromJson(widget.rule.toJson());
  RulePreview? preview;
  String? error;
  bool _saving = false;
  final _conditionsScroll = ScrollController();
  final _actionsScroll = ScrollController();

  @override
  void dispose() {
    _conditionsScroll.dispose();
    _actionsScroll.dispose();
    super.dispose();
  }

  void _scrollToEnd(ScrollController c) =>
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!c.hasClients) return;
        c.animateTo(
          c.position.maxScrollExtent,
          duration: const Duration(milliseconds: 180),
          curve: Curves.easeOut,
        );
      });

  bool get _single => rule.scope.isSingleAccount;

  List<String> get _scopeAccounts =>
      rule.scope.accounts ?? widget.accounts.keys.toList();

  String _accountLabel(String id) => widget.accounts[id] ?? id;

  /// Comptes et boîtes proposés aux conditions : ceux de la portée.
  Map<String, List<RuleChoice>> get _conditionOptions {
    final several = _scopeAccounts.length > 1;
    return {
      'account': [
        for (final a in _scopeAccounts) RuleChoice(a, _accountLabel(a)),
      ],
      'mailbox': [
        for (final a in _scopeAccounts)
          for (final b in widget.mailboxes[a] ?? const <RuleChoice>[])
            RuleChoice(
              b.id,
              several ? '${_accountLabel(a)} · ${b.label}' : b.label,
            ),
      ],
    };
  }

  /// Boîtes proposées aux actions : celles du seul compte de la portée.
  Map<String, List<RuleChoice>> get _actionOptions => {
    'mailbox': _single
        ? widget.mailboxes[_scopeAccounts.single] ?? const []
        : const [],
  };

  List<CatalogEntry> get _conditionChoices => [
    for (final e in widget.catalog.conditions)
      if (_single || !e.singleAccount) e,
  ];

  List<CatalogEntry> get _actionChoices => [
    for (final e in widget.catalog.actions)
      if (_single || !(e.needsMailbox || e.singleAccount)) e,
  ];

  void _touch() => setState(() => preview = null);

  void _addAlternative() {
    final first = _conditionChoices.firstOrNull;
    if (first == null) return;
    rule.alternatives.add(
      ConditionGroup([Condition(test: defaultsFor(first))]),
    );
    _touch();
    _scrollToEnd(_conditionsScroll);
  }

  void _addCondition(int group) {
    final first = _conditionChoices.firstOrNull;
    if (first == null) return;
    rule.alternatives[group].conditions.add(
      Condition(test: defaultsFor(first)),
    );
    _touch();
    _scrollToEnd(_conditionsScroll);
  }

  void _addAction() {
    final first = _actionChoices.firstOrNull;
    if (first == null) return;
    rule.actions.add(RuleAction(defaultsFor(first)));
    _touch();
    _scrollToEnd(_actionsScroll);
  }

  void _setAccounts(List<String>? ids) {
    rule.scope = ids == null ? const RuleScope.all() : RuleScope.accounts(ids);
    if (!_single) {
      final hidden = {
        for (final e in widget.catalog.actions)
          if (e.needsMailbox || e.singleAccount) e.kind,
      };
      rule.actions.removeWhere((a) => hidden.contains(a.type));
      final hiddenTests = {
        for (final e in widget.catalog.conditions)
          if (e.singleAccount) e.kind,
      };
      for (final g in rule.alternatives) {
        g.conditions.removeWhere((c) => hiddenTests.contains(c.test['type']));
      }
    }
    _touch();
  }

  Future<void> _save() async {
    if (_saving) return;
    _saving = true;
    try {
      await widget.onSave(rule);
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      _saving = false;
    }
  }

  Future<void> _runPreview() async {
    try {
      final p = await widget.onPreview(rule);
      setState(() => preview = p);
    } catch (e) {
      setState(() => error = '$e');
    }
  }

  Widget _scope() {
    final ids = widget.accounts.keys.toList();
    final chosen = rule.scope.accounts;
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        RuleChip(
          key: const ValueKey('scope-all'),
          label: t('Tous les comptes'),
          selected: rule.scope.isAll,
          onTap: () => _setAccounts(null),
        ),
        for (final id in ids)
          RuleChip(
            key: ValueKey('scope-account-$id'),
            label: widget.accounts[id] ?? id,
            selected: chosen?.contains(id) ?? false,
            onTap: () {
              final next = [...?chosen];
              next.contains(id) ? next.remove(id) : next.add(id);
              _setAccounts(next);
            },
          ),
      ],
    );
  }

  Widget _group(int g) {
    final group = rule.alternatives[g];
    return RuleBlock(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (var c = 0; c < group.conditions.length; c++)
            ConditionRow(
              key: ObjectKey(group.conditions[c]),
              group: g,
              index: c,
              condition: group.conditions[c],
              entries: _conditionChoices,
              options: _conditionOptions,
              onChanged: _touch,
              onRemove: () {
                group.conditions.removeAt(c);
                if (group.conditions.isEmpty) rule.alternatives.removeAt(g);
                _touch();
              },
            ),
          Align(
            alignment: Alignment.centerLeft,
            child: Btn(
              key: ValueKey('add-condition-$g'),
              icon: 'plus',
              label: t('Ajouter une condition'),
              onTap: () => _addCondition(g),
            ),
          ),
        ],
      ),
    );
  }

  Widget _alternatives() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        for (var g = 0; g < rule.alternatives.length; g++) ...[
          if (g > 0) RuleOrDivider(t('OU')),
          KeyedSubtree(key: ObjectKey(rule.alternatives[g]), child: _group(g)),
        ],
        const SizedBox(height: 12),
        Align(
          alignment: Alignment.centerLeft,
          child: Btn(
            key: const ValueKey('add-alternative'),
            icon: 'plus',
            label: t('Ajouter une alternative (OU)'),
            onTap: _addAlternative,
          ),
        ),
      ],
    );
  }

  Widget _actions() => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      for (var a = 0; a < rule.actions.length; a++)
        ActionRow(
          key: ObjectKey(rule.actions[a]),
          index: a,
          action: rule.actions[a],
          entries: _actionChoices,
          options: _actionOptions,
          onChanged: _touch,
          onRemove: () {
            rule.actions.removeAt(a);
            _touch();
          },
        ),
      Align(
        alignment: Alignment.centerLeft,
        child: Btn(
          key: const ValueKey('add-action'),
          icon: 'plus',
          label: t('Ajouter une action'),
          onTap: _addAction,
        ),
      ),
    ],
  );

  Widget _preview() {
    final p = preview;
    if (p == null) return const SizedBox.shrink();
    final c = context.pal;
    return Container(
      key: const ValueKey('preview-result'),
      margin: const EdgeInsets.only(top: 18),
      padding: const EdgeInsets.fromLTRB(12, 4, 0, 4),
      decoration: BoxDecoration(
        border: Border(left: BorderSide(color: c.accent, width: 2)),
      ),
      child: Text(
        p.samples.isEmpty
            ? t('{0} messages touchés sur {1} examinés.', [
                p.matched,
                p.scanned,
              ])
            : t('{0} messages touchés sur {1} examinés, par exemple : {2}', [
                p.matched,
                p.scanned,
                p.samples.join(', '),
              ]),
        style: Fonts.sans(size: 13.5, color: c.muted),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          RuleLabel(
            t('Nom de la règle'),
            padding: const EdgeInsets.only(bottom: 10),
          ),
          RuleField(
            fieldKey: const ValueKey('rule-name'),
            initialValue: rule.name,
            onChanged: (v) => rule.name = v,
          ),
          RuleLabel(t('Portée')),
          _scope(),
          RuleLabel(t('Quand le message correspond à')),
          RuleScroll(
            maxHeight: 340,
            controller: _conditionsScroll,
            child: _alternatives(),
          ),
          RuleLabel(t('Alors')),
          RuleScroll(
            maxHeight: 220,
            controller: _actionsScroll,
            child: _actions(),
          ),
          _preview(),
          if (error != null)
            Padding(
              padding: const EdgeInsets.only(top: 14),
              child: Text(
                error!,
                style: Fonts.sans(size: 13.5, color: p.accent),
              ),
            ),
          const SizedBox(height: 22),
          Container(
            padding: const EdgeInsets.only(top: 18),
            decoration: BoxDecoration(
              border: Border(top: BorderSide(color: p.lineSoft)),
            ),
            child: Wrap(
              spacing: 10,
              runSpacing: 10,
              children: [
                Btn(
                  key: const ValueKey('save-rule'),
                  label: t('Enregistrer'),
                  go: true,
                  onTap: _save,
                ),
                Btn(
                  key: const ValueKey('preview-rule'),
                  label: t('Aperçu'),
                  onTap: _runPreview,
                ),
                if (widget.onCancel != null)
                  Btn(label: t('Annuler'), onTap: widget.onCancel),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
