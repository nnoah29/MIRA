import 'package:flutter/material.dart';

import '../i18n/i18n.dart';

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

/// Un paramètre d'un test ou d'une action : liste, nombre, interrupteur, texte ou filtre texte.
class RuleParam extends StatelessWidget {
  final String name;
  final Map<String, dynamic> data;
  final VoidCallback onChanged;
  const RuleParam({
    super.key,
    required this.name,
    required this.data,
    required this.onChanged,
  });

  Widget _dropdown(
    String current,
    List<String> options,
    void Function(String) set,
  ) => DropdownButton<String>(
    value: options.contains(current) ? current : null,
    items: [
      for (final o in options)
        DropdownMenuItem(value: o, child: Text(_choiceLabel(o))),
    ],
    onChanged: (v) {
      if (v != null) {
        set(v);
        onChanged();
      }
    },
  );

  Widget _text(
    String initial,
    void Function(String) set, {
    bool number = false,
  }) => SizedBox(
    width: 180,
    child: TextFormField(
      initialValue: initial,
      keyboardType: number ? TextInputType.number : TextInputType.text,
      onChanged: (v) => set(v),
    ),
  );

  Widget _matcher() {
    final m = (data['matcher'] as Map).cast<String, dynamic>();
    final values = (m['values'] as List).map((v) => '$v').toList();
    return Wrap(
      spacing: 8,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        _dropdown('${m['op']}', _matcherOps, (v) => m['op'] = v),
        _text(values.join(', '), (v) {
          m['values'] = [for (final s in v.split(',')) s.trim()];
        }),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final v = data[name];
    if (name == 'matcher') return _matcher();
    if (_choices.containsKey(name)) {
      return _dropdown('$v', _choices[name]!, (s) => data[name] = s);
    }
    if (v is bool) {
      return Switch(
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
