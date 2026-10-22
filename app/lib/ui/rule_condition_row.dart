import 'package:flutter/material.dart';

import '../data/rule_models.dart';
import '../i18n/i18n.dart';
import 'rule_labels.dart';
import 'rule_params.dart';
import 'rule_widgets.dart';
import 'widgets.dart';

/// Choix de la variante (condition ou action) d'une ligne, puis ses paramètres.
class RuleKindPicker extends StatelessWidget {
  final String keyPrefix;
  final List<CatalogEntry> entries;
  final Map<String, dynamic> data;
  final Map<String, List<RuleChoice>> options;
  final VoidCallback onChanged;
  const RuleKindPicker({
    super.key,
    required this.keyPrefix,
    required this.entries,
    required this.data,
    this.options = const {},
    required this.onChanged,
  });

  void _switchTo(String kind) {
    final entry = entries.firstWhere((e) => e.kind == kind);
    data
      ..clear()
      ..addAll(defaultsFor(entry));
    onChanged();
  }

  @override
  Widget build(BuildContext context) {
    final current = '${data['type']}';
    final entry = entries.where((e) => e.kind == current).firstOrNull;
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        RuleSelect(
          key: ValueKey('$keyPrefix-kind'),
          value: entry == null ? null : current,
          hint: ruleKindLabel(current),
          options: [
            for (final e in entries)
              RuleOption(
                e.kind,
                ruleKindLabel(e.kind),
                key: ValueKey('$keyPrefix-option-${e.kind}'),
              ),
          ],
          onChanged: _switchTo,
        ),
        for (final f in entry?.fields ?? const <String>[])
          RuleParam(
            key: ValueKey('${identityHashCode(data)}-$current-$f'),
            keyPrefix: keyPrefix,
            name: f,
            data: data,
            options: options,
            onChanged: onChanged,
          ),
      ],
    );
  }
}

/// Une condition : variante et paramètres, bouton d'inversion (« NON »), suppression.
class ConditionRow extends StatelessWidget {
  final int group, index;
  final Condition condition;
  final List<CatalogEntry> entries;
  final Map<String, List<RuleChoice>> options;
  final VoidCallback onChanged, onRemove;
  const ConditionRow({
    super.key,
    required this.group,
    required this.index,
    required this.condition,
    required this.entries,
    this.options = const {},
    required this.onChanged,
    required this.onRemove,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (index > 0)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Mono(t('ET'), size: 10.5, ls: .16, height: null),
            ),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              RuleChip(
                key: ValueKey('negate-$group-$index'),
                label: t('NON'),
                height: ruleControlHeight,
                selected: condition.negated,
                onTap: () {
                  condition.negated = !condition.negated;
                  onChanged();
                },
              ),
              const SizedBox(width: 10),
              Expanded(
                child: RuleKindPicker(
                  keyPrefix: 'condition-$group-$index',
                  entries: entries,
                  data: condition.test,
                  options: options,
                  onChanged: onChanged,
                ),
              ),
              IBtn(
                'x',
                key: ValueKey('remove-condition-$group-$index'),
                box: ruleControlHeight,
                tip: t('Retirer la condition'),
                onTap: onRemove,
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// Une action : variante et paramètres, suppression.
class ActionRow extends StatelessWidget {
  final int index;
  final RuleAction action;
  final List<CatalogEntry> entries;
  final Map<String, List<RuleChoice>> options;
  final VoidCallback onChanged, onRemove;
  const ActionRow({
    super.key,
    required this.index,
    required this.action,
    required this.entries,
    this.options = const {},
    required this.onChanged,
    required this.onRemove,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: RuleKindPicker(
              keyPrefix: 'action-$index',
              entries: entries,
              data: action.data,
              options: options,
              onChanged: onChanged,
            ),
          ),
          IBtn(
            'x',
            key: ValueKey('remove-action-$index'),
            box: ruleControlHeight,
            tip: t('Retirer l\'action'),
            onTap: onRemove,
          ),
        ],
      ),
    );
  }
}
