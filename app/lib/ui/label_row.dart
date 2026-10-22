import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../engine/label_settings_admin.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'label_map_button.dart';
import 'labels_settings.dart' show nextLabelColor;
import 'rule_widgets.dart';
import 'widgets.dart';

/// Modification d'un ou plusieurs réglages d'un libellé.
typedef LabelSet = Future<void> Function({
  bool? hidden,
  bool? hideFromInbox,
  bool? notifications,
  CounterMode? counter,
  int? color,
});

/// Une ligne de l'onglet des libellés : identité, volume, réglages et actions.
class LabelRow extends StatelessWidget {
  final LabelOverview label;
  final bool canUp;
  final bool canDown;
  final LabelSet onSet;
  final void Function(int delta) onMove;
  final void Function(String name) onRename;
  final VoidCallback onDelete;
  const LabelRow({
    super.key,
    required this.label,
    required this.canUp,
    required this.canDown,
    required this.onSet,
    required this.onMove,
    required this.onRename,
    required this.onDelete,
  });

  Widget _option(String id, String text, bool on, VoidCallback tap) => RuleChip(
    key: ValueKey('$id-${label.key}'),
    label: text,
    selected: on,
    onTap: tap,
  );

  Widget _counter(BuildContext context) => Wrap(
    spacing: 6,
    children: [
      for (final (mode, text) in [
        (CounterMode.unread, t('Non lus')),
        (CounterMode.total, t('Total')),
        (CounterMode.none, t('Aucun')),
      ])
        RuleChip(
          key: ValueKey('counter-${label.key}-${mode.name}'),
          label: text,
          selected: label.prefs.counter == mode,
          onTap: () => onSet(counter: mode),
        ),
    ],
  );

  Future<void> _askRename(BuildContext context) async {
    final p = context.pal;
    final ctl = TextEditingController(text: label.name);
    final to = await showDialog<String>(
      context: context,
      builder: (c) => Dialog(
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 380),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  t('Renommer le libellé'),
                  style: Fonts.sans(
                    size: 16,
                    w: FontWeight.w600,
                    color: p.text,
                  ),
                ),
                const SizedBox(height: 14),
                RuleField(
                  fieldKey: const ValueKey('rename-label'),
                  initialValue: label.name,
                  onChanged: (v) => ctl.text = v,
                ),
                const SizedBox(height: 20),
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    Btn(label: t('Annuler'), onTap: () => Navigator.pop(c)),
                    const SizedBox(width: 10),
                    Btn(
                      label: t('Renommer'),
                      go: true,
                      onTap: () => Navigator.pop(c, ctl.text.trim()),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
    ctl.dispose();
    if (to != null && to.isNotEmpty && to != label.name) onRename(to);
  }

  Future<void> _askDelete(BuildContext context) async {
    final p = context.pal;
    final ok = await showDialog<bool>(
      context: context,
      builder: (c) => Dialog(
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 420),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  t('Supprimer le libellé « {0} » ?', [label.name]),
                  style: Fonts.sans(
                    size: 16,
                    w: FontWeight.w600,
                    color: p.text,
                  ),
                ),
                const SizedBox(height: 10),
                Text(
                  t('Les messages restent, seul le libellé disparaît.'),
                  style: Fonts.sans(size: 14, color: p.dim),
                ),
                const SizedBox(height: 20),
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    Btn(
                      label: t('Annuler'),
                      onTap: () => Navigator.pop(c, false),
                    ),
                    const SizedBox(width: 10),
                    Btn(
                      key: const ValueKey('confirm-delete-label'),
                      label: t('Supprimer'),
                      go: true,
                      onTap: () => Navigator.pop(c, true),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
    if (ok == true) onDelete();
  }

  Widget _head(BuildContext context) {
    final p = context.pal;
    return Row(
      children: [
        Hov(
          key: ValueKey('color-${label.key}'),
          onTap: () => onSet(color: nextLabelColor(label.color)),
          builder: (_, h) => Container(
            width: 18,
            height: 18,
            decoration: BoxDecoration(
              color: Color(label.color),
              border: Border.all(color: h ? p.text : p.line),
            ),
          ),
        ),
        const SizedBox(width: 14),
        Expanded(
          child: Hov(
            onTap: () => _askRename(context),
            builder: (_, h) => Text(
              label.name,
              overflow: TextOverflow.ellipsis,
              style: Fonts.sans(size: 15, color: h ? p.accent : p.text),
            ),
          ),
        ),
        if (label.unused)
          Padding(
            padding: const EdgeInsets.only(right: 12),
            child: Mono(
              t('Inutilisé'),
              size: 10.5,
              color: p.accent,
              height: null,
            ),
          )
        else
          Padding(
            padding: const EdgeInsets.only(right: 12),
            child: Mono('${label.messages}', size: 11, height: null),
          ),
        if (canUp)
          IBtn(
            'chev',
            key: ValueKey('up-${label.key}'),
            tip: t('Monter'),
            onTap: () => onMove(-1),
          ),
        if (canDown)
          Transform.rotate(
            angle: math.pi,
            child: IBtn(
              'chev',
              key: ValueKey('down-${label.key}'),
              tip: t('Descendre'),
              onTap: () => onMove(1),
            ),
          ),
        IBtn(
          'trash',
          key: ValueKey('delete-${label.key}'),
          tip: t('Supprimer'),
          onTap: () => _askDelete(context),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final prefs = label.prefs;
    return Container(
      padding: const EdgeInsets.fromLTRB(16, 14, 16, 14),
      decoration: BoxDecoration(
        border: Border(top: BorderSide(color: p.lineSoft)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _head(context),
          const SizedBox(height: 14),
          Wrap(
            spacing: 8,
            runSpacing: 10,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              _option(
                'sidebar',
                t('Dans la barre latérale'),
                !prefs.hidden,
                () => onSet(hidden: !prefs.hidden),
              ),
              _option(
                'inbox',
                t('Masquer de la Réception'),
                prefs.hideFromInbox,
                () => onSet(hideFromInbox: !prefs.hideFromInbox),
              ),
              _option(
                'notify',
                t('Notifications'),
                prefs.notifications,
                () => onSet(notifications: !prefs.notifications),
              ),
              _counter(context),
              if (!label.local) LabelMapButton(id: label.id),
            ],
          ),
        ],
      ),
    );
  }
}
