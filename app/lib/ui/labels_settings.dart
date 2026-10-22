import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../engine/label_settings_admin.dart';
import '../i18n/i18n.dart';
import '../state/local_labels.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'label_row.dart';
import 'rule_widgets.dart';
import 'widgets.dart';

/// Onglet des libellés : tous les libellés (locaux, puis ceux de chaque compte) et leurs réglages.
class LabelsSettings extends StatefulWidget {
  const LabelsSettings({super.key});

  @override
  State<LabelsSettings> createState() => _LabelsSettingsState();
}

class _LabelsSettingsState extends State<LabelsSettings> {
  final name = TextEditingController();
  String where = 'local';
  bool invalid = false;
  int fieldGen = 0;
  final collapsed = <String>{};

  @override
  void dispose() {
    name.dispose();
    super.dispose();
  }

  Future<void> _add(MailStore s) async {
    final text = name.text.trim();
    final src = s.source;
    var ok = text.isNotEmpty;
    if (ok && where == 'local') {
      ok = s.addLocalLabel(text);
    } else if (ok && src?.labelAdmin != null) {
      await src!.labelAdmin!.create(where, text);
      await s.refreshLabelSettings();
    }
    if (ok) name.clear();
    setState(() {
      invalid = !ok;
      if (ok) fieldGen++;
    });
  }

  /// Échange le libellé avec son voisin dans le même groupe et fixe le nouvel ordre.
  Future<void> _move(MailStore s, LabelOverview l, int delta) async {
    final all = [...s.labelOverviews];
    final group = [
      for (final o in all)
        if (o.local == l.local && o.account == l.account) o,
    ];
    final at = group.indexWhere((o) => o.key == l.key) + delta;
    if (at < 0 || at >= group.length) return;
    final a = all.indexWhere((o) => o.key == l.key);
    final b = all.indexWhere((o) => o.key == group[at].key);
    final moved = all.removeAt(a);
    all.insert(b, moved);
    await s.source!.labelSettingsAdmin!.reorder(all);
    await s.refreshLabelSettings();
  }

  Future<void> _set(
    MailStore s,
    LabelOverview l, {
    bool? hidden,
    bool? hideFromInbox,
    bool? notifications,
    CounterMode? counter,
    int? color,
  }) async {
    await s.source!.labelSettingsAdmin!.set(
      l,
      hidden: hidden,
      hideFromInbox: hideFromInbox,
      notifications: notifications,
      counter: counter,
      color: color,
    );
    await s.refreshLabelSettings();
  }

  Future<void> _rename(MailStore s, LabelOverview l, String to) async {
    if (l.local) {
      s.renameLocalLabel(l.key, to);
    } else {
      await s.source!.labelAdmin!.rename(l.id, to);
    }
    await s.refreshLabelSettings();
  }

  Future<void> _delete(MailStore s, LabelOverview l) async {
    if (l.local) {
      s.deleteLocalLabel(l.key);
    } else {
      await s.source!.labelAdmin!.delete(l.id);
    }
    await s.refreshLabelSettings();
  }

  Widget _adder(MailStore s) {
    final p = context.pal;
    final places = [
      RuleOption('local', t('Local')),
      for (final a in s.accounts)
        if (a.id != 'none') RuleOption(a.id, a.email),
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Wrap(
          spacing: 10,
          runSpacing: 10,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            KeyedSubtree(
              key: ValueKey('new-label-field-$fieldGen'),
              child: RuleField(
                fieldKey: const ValueKey('new-label-name'),
                initialValue: '',
                width: 240,
                hint: t('Nom du libellé'),
                onChanged: (v) => name.text = v,
              ),
            ),
            RuleSelect(
              key: const ValueKey('new-label-where'),
              value: where,
              hint: t('Local'),
              options: places,
              onChanged: (v) => setState(() => where = v),
            ),
            Btn(
              key: const ValueKey('add-label'),
              icon: 'plus',
              label: t('Ajouter'),
              onTap: () => _add(s),
            ),
          ],
        ),
        if (invalid)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(
              t('Nom vide ou déjà pris.'),
              style: Fonts.sans(size: 13, color: p.accent),
            ),
          ),
      ],
    );
  }

  /// Cadre d'un groupe (locaux ou un compte), pliable : titre, nombre de libellés, puis ses lignes.
  Widget _group(
    MailStore s,
    String id,
    String title,
    List<LabelOverview> items,
  ) {
    final p = context.pal;
    final open = !collapsed.contains(id);
    return Container(
      key: ValueKey('group-$id'),
      margin: const EdgeInsets.only(top: 16),
      decoration: BoxDecoration(
        color: p.bg,
        border: Border.all(color: p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Hov(
            key: ValueKey('fold-$id'),
            onTap: () =>
                setState(() => open ? collapsed.add(id) : collapsed.remove(id)),
            builder: (_, h) => Container(
              color: h ? p.surface : p.surface2,
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              child: Row(
                children: [
                  Transform.rotate(
                    angle: open ? 0 : -math.pi / 2,
                    child: Ico('chev', size: 14, color: h ? p.text : p.muted),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Mono(
                      title,
                      size: 11,
                      ls: .16,
                      height: null,
                      color: h ? p.text : p.muted,
                    ),
                  ),
                  Mono('${items.length}', size: 11, height: null),
                ],
              ),
            ),
          ),
          if (open)
            for (final (i, l) in items.indexed)
              LabelRow(
                key: ValueKey('label-${l.key}'),
                label: l,
                canUp: i > 0,
                canDown: i < items.length - 1,
                onSet:
                    ({hidden, hideFromInbox, notifications, counter, color}) =>
                        _set(
                          s,
                          l,
                          hidden: hidden,
                          hideFromInbox: hideFromInbox,
                          notifications: notifications,
                          counter: counter,
                          color: color,
                        ),
                onMove: (d) => _move(s, l, d),
                onRename: (to) => _rename(s, l, to),
                onDelete: () => _delete(s, l),
              ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final locals = [
      for (final o in s.labelOverviews)
        if (o.local) o,
    ];
    final byAccount = <String, List<LabelOverview>>{};
    for (final o in s.labelOverviews) {
      if (!o.local) byAccount.putIfAbsent(o.account ?? '', () => []).add(o);
    }
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 16, 20, 20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('Libellés'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          Text(
            t(
              'Tous tes libellés, locaux et ceux de tes comptes : masquer, ranger dans l’ordre, couper les notifications, choisir le compteur.',
            ),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          const SizedBox(height: 16),
          _adder(s),
          if (s.labelOverviews.isEmpty)
            Padding(
              padding: const EdgeInsets.only(top: 18),
              child: Text(
                t('Aucun libellé.'),
                style: Fonts.sans(size: 13.5, color: p.dim),
              ),
            ),
          if (locals.isNotEmpty) _group(s, 'local', t('Locaux'), locals),
          for (final e in byAccount.entries)
            _group(
              s,
              e.key,
              s.accounts.where((a) => a.id == e.key).firstOrNull?.email ??
                  e.key,
              e.value,
            ),
        ],
      ),
    );
  }
}

/// Couleur suivante dans la palette des libellés.
int nextLabelColor(int current) {
  final i = localLabelColors.indexOf(current);
  return localLabelColors[(i + 1) % localLabelColors.length];
}
