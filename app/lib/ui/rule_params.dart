import 'package:flutter/material.dart';

import '../data/rule_models.dart';
import '../i18n/i18n.dart';
import 'rule_widgets.dart';

const _choices = <String, List<String>>{
  'field': ['address', 'name', 'domain'],
  'which': ['to', 'cc', 'bcc'],
  'cmp': ['less', 'equal', 'greater'],
};

const _matcherOps = [
  'contains',
  'equals',
  'starts_with',
  'ends_with',
  'regex',
  'in_list',
];

String _choiceLabel(String v) => switch (v) {
  'address' => t('adresse'),
  'name' => t('nom'),
  'domain' => t('domaine'),
  'to' => t('À'),
  'cc' => t('Cc'),
  'bcc' => t('Cci'),
  'less' => t('<'),
  'equal' => t('='),
  'greater' => t('>'),
  'contains' => t('contient'),
  'equals' => t('est égal à'),
  'starts_with' => t('commence par'),
  'ends_with' => t('finit par'),
  'regex' => t('expression régulière'),
  'in_list' => t('dans la liste'),
  _ => v,
};

/// Valeurs d'un filtre lues dans la saisie : découpées sur la virgule pour
/// « dans la liste » seulement, sans valeur vide ni faite d'espaces.
List<String> matcherValues(String op, String text) {
  final parts = op == 'in_list' ? text.split(',').map((p) => p.trim()) : [text];
  return [
    for (final p in parts)
      if (p.trim().isNotEmpty) p,
  ];
}

/// Un paramètre d'un test ou d'une action : liste, nombre, interrupteur, texte,
/// filtre texte, ou choix d'une boîte ou d'un compte parmi `options`.
class RuleParam extends StatelessWidget {
  final String keyPrefix;
  final String name;
  final Map<String, dynamic> data;
  final Map<String, List<RuleChoice>> options;
  final VoidCallback onChanged;
  const RuleParam({
    super.key,
    required this.keyPrefix,
    required this.name,
    required this.data,
    this.options = const {},
    required this.onChanged,
  });

  Widget _dropdown(
    String current,
    List<String> options,
    void Function(String) set,
  ) => RuleSelect(
    value: options.contains(current) ? current : null,
    hint: _choiceLabel(current),
    options: [for (final o in options) RuleOption(o, _choiceLabel(o))],
    onChanged: (v) {
      set(v);
      onChanged();
    },
  );

  /// Liste des boîtes ou des comptes ; une valeur absente de la liste reste
  /// affichée, signalée « introuvable ».
  Widget _choice(String current, List<RuleChoice> choices) {
    final known = choices.any((c) => c.id == current);
    return RuleSelect(
      key: ValueKey('$keyPrefix-$name'),
      value: current.isEmpty ? null : current,
      hint: t('Choisir…'),
      options: [
        for (final c in choices) RuleOption(c.id, c.label),
        if (!known && current.isNotEmpty)
          RuleOption(current, t('{0} (introuvable)', [current])),
      ],
      onChanged: (v) {
        data[name] = v;
        onChanged();
      },
    );
  }

  Widget _text(
    String initial,
    void Function(String) set, {
    bool number = false,
    String variant = '',
  }) => RuleField(
    width: 200,
    fieldKey: ValueKey(
      '${identityHashCode(data)}-${data['type']}-$name$variant',
    ),
    initialValue: initial,
    number: number,
    onChanged: set,
  );

  Widget _matcher() {
    final m = (data['matcher'] as Map).cast<String, dynamic>();
    final values = (m['values'] as List).map((v) => '$v').toList();
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        _dropdown('${m['op']}', _matcherOps, (op) {
          final shown = (m['values'] as List).map((v) => '$v').join(', ');
          m['op'] = op;
          m['values'] = matcherValues(op, shown);
        }),
        _text(
          values.join(', '),
          (v) => m['values'] = matcherValues('${m['op']}', v),
          variant: '-${m['op']}',
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final v = data[name];
    if (name == 'matcher') return _matcher();
    final choices = options[name];
    if (choices != null) return _choice('${v ?? ''}', choices);
    if (_choices.containsKey(name)) {
      return _dropdown('$v', _choices[name]!, (s) => data[name] = s);
    }
    if (v is bool) {
      return RuleToggle(
        value: v,
        onChanged: (b) {
          data[name] = b;
          onChanged();
        },
      );
    }
    if (v is num) {
      return _text(
        '$v',
        (s) => data[name] = int.tryParse(s) ?? v,
        number: true,
      );
    }
    return _text('${v ?? ''}', (s) => data[name] = s);
  }
}
