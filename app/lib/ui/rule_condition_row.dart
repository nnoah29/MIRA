import 'package:flutter/material.dart';

import '../data/rule_models.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'rule_labels.dart';
import 'rule_params.dart';

/// Choix de la variante (condition ou action) d'une ligne, puis ses paramètres.
class RuleKindPicker extends StatelessWidget {
  final String keyPrefix;
  final List<CatalogEntry> entries;
  final Map<String, dynamic> data;
  final VoidCallback onChanged;
  const RuleKindPicker({
    super.key,
    required this.keyPrefix,
    required this.entries,
    required this.data,
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
      spacing: 10,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        DropdownButton<String>(
          key: ValueKey('$keyPrefix-kind'),
          value: entry == null ? null : current,
          hint: Text(ruleKindLabel(current)),
          items: [
            for (final e in entries)
              DropdownMenuItem(
                key: ValueKey('$keyPrefix-option-${e.kind}'),
                value: e.kind,
                child: Text(ruleKindLabel(e.kind)),
              ),
          ],
          onChanged: (k) => k == null ? null : _switchTo(k),
        ),
        for (final f in entry?.fields ?? const <String>[])
          RuleParam(name: f, data: data, onChanged: onChanged),
      ],
    );
  }
}

/// Une condition : variante et paramètres, bouton d'inversion (« NON »), suppression.
class ConditionRow extends StatelessWidget {
  final int group, index;
  final Condition condition;
  final List<CatalogEntry> entries;
  final VoidCallback onChanged, onRemove;
  const ConditionRow({
    super.key,
    required this.group,
    required this.index,
    required this.condition,
    required this.entries,
    required this.onChanged,
    required this.onRemove,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Row(
      children: [
        Text(
          index == 0 ? '' : t('ET'),
          style: Fonts.sans(size: 12, color: p.dim),
        ),
        const SizedBox(width: 8),
        FilterChip(
          key: ValueKey('negate-$group-$index'),
          label: Text(t('NON')),
          selected: condition.negated,
          onSelected: (v) {
            condition.negated = v;
            onChanged();
          },
        ),
        const SizedBox(width: 8),
        Expanded(
          child: RuleKindPicker(
            keyPrefix: 'condition-$group-$index',
            entries: entries,
            data: condition.test,
            onChanged: onChanged,
          ),
        ),
        IconButton(
          key: ValueKey('remove-condition-$group-$index'),
          tooltip: t('Retirer la condition'),
          icon: const Icon(Icons.close, size: 18),
          onPressed: onRemove,
        ),
      ],
    );
  }
}

/// Une action : variante et paramètres, suppression.
class ActionRow extends StatelessWidget {
  final int index;
  final RuleAction action;
  final List<CatalogEntry> entries;
  final VoidCallback onChanged, onRemove;
  const ActionRow({
    super.key,
    required this.index,
    required this.action,
    required this.entries,
    required this.onChanged,
    required this.onRemove,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: RuleKindPicker(
            keyPrefix: 'action-$index',
            entries: entries,
            data: action.data,
            onChanged: onChanged,
          ),
        ),
        IconButton(
          key: ValueKey('remove-action-$index'),
          tooltip: t('Retirer l\'action'),
          icon: const Icon(Icons.close, size: 18),
          onPressed: onRemove,
        ),
      ],
    );
  }
}
