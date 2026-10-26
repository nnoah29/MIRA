import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../i18n/i18n.dart';
import '../state/image_policy.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

String _label(ImageAction a) => switch (a) {
  ImageAction.ask => t('Demander'),
  ImageAction.always => t('Toujours afficher'),
  ImageAction.never => t('Ne jamais afficher'),
};

/// Choix « une bonne fois pour toutes » pour les images distantes, avec règles par adresse ou domaine.
class ImageRulesSection extends StatefulWidget {
  const ImageRulesSection({super.key});

  @override
  State<ImageRulesSection> createState() => _ImageRulesSectionState();
}

class _ImageRulesSectionState extends State<ImageRulesSection> {
  final ctl = TextEditingController();
  ImageAction action = ImageAction.never;
  bool invalid = false;

  @override
  void dispose() {
    ctl.dispose();
    super.dispose();
  }

  void add(MailStore s) {
    final ok = s.addImageRule(ctl.text, action);
    setState(() => invalid = !ok);
    if (ok) ctl.clear();
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('Images distantes'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          Text(
            t(
              'Les images hébergées sur Internet révèlent que tu lis un message. Choisis le comportement par défaut, puis ajoute des exceptions.',
            ),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          const SizedBox(height: 12),
          _Choice(
            value: s.imagePolicy.fallback,
            onChanged: s.setImageDefault,
            keyPrefix: 'default',
          ),
          const SizedBox(height: 18),
          for (final r in s.imagePolicy.rules) _RuleRow(rule: r, store: s),
          if (s.imagePolicy.rules.isEmpty)
            Text(
              t('Aucune exception.'),
              style: Fonts.sans(size: 13.5, color: p.dim),
            ),
          const SizedBox(height: 12),
          Wrap(
            spacing: 12,
            runSpacing: 10,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              SizedBox(
                width: 280,
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 10,
                  ),
                  decoration: BoxDecoration(
                    color: p.bg,
                    border: Border.all(color: invalid ? p.accent : p.line),
                  ),
                  child: TextField(
                    key: const ValueKey('rule-input'),
                    controller: ctl,
                    onSubmitted: (_) => add(s),
                    cursorColor: p.accent,
                    cursorWidth: 1,
                    style: Fonts.sans(size: 15, color: p.text),
                    decoration: InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                      hintText: t('adresse ou domaine (ex. news.exemple.com)'),
                      hintStyle: Fonts.sans(size: 15, color: p.dim),
                    ),
                  ),
                ),
              ),
              _Choice(
                value: action,
                onChanged: (a) => setState(() => action = a),
                keyPrefix: 'new',
                withAsk: false,
              ),
              Btn(icon: 'plus', label: t('Ajouter'), onTap: () => add(s)),
            ],
          ),
          if (invalid)
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: Text(
                t(
                  'Saisis une adresse (nom@domaine.com) ou un domaine (domaine.com).',
                ),
                style: Fonts.sans(size: 13, color: p.accent),
              ),
            ),
        ],
      ),
    );
  }
}

class _RuleRow extends StatelessWidget {
  final ImageRule rule;
  final MailStore store;
  const _RuleRow({required this.rule, required this.store});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 6),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: p.lineSoft)),
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              rule.match,
              style: Fonts.sans(size: 15, color: p.text),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          _Choice(
            value: rule.action,
            withAsk: false,
            keyPrefix: 'rule-${rule.match}',
            onChanged: (a) => store.addImageRule(rule.match, a),
          ),
          IBtn(
            'x',
            tip: t('Supprimer la règle'),
            onTap: () => store.removeImageRule(rule.match),
          ),
        ],
      ),
    );
  }
}

class _Choice extends StatelessWidget {
  final ImageAction value;
  final ValueChanged<ImageAction> onChanged;
  final String keyPrefix;
  final bool withAsk;
  const _Choice({
    required this.value,
    required this.onChanged,
    required this.keyPrefix,
    this.withAsk = true,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final opts = [
      if (withAsk) ImageAction.ask,
      ImageAction.always,
      ImageAction.never,
    ];
    final narrow = MediaQuery.sizeOf(context).width <= 760;
    return LayoutBuilder(
      builder: (_, box) {
        final stacked = narrow && box.hasBoundedWidth;
        final cells = [
          for (final (i, a) in opts.indexed) _cell(p, i, a, stacked),
        ];
        return Container(
          decoration: BoxDecoration(border: Border.all(color: p.line)),
          child: stacked
              ? Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: cells,
                )
              : Row(mainAxisSize: MainAxisSize.min, children: cells),
        );
      },
    );
  }

  /// Une option du choix, séparée de la précédente par un filet.
  Widget _cell(Pal p, int i, ImageAction a, bool mobile) => Hov(
    key: ValueKey('$keyPrefix-${a.name}'),
    onTap: () => onChanged(a),
    builder: (_, h) => Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
      decoration: BoxDecoration(
        color: value == a ? p.accentSoft : null,
        border: i == 0
            ? null
            : mobile
            ? Border(top: BorderSide(color: p.line))
            : Border(left: BorderSide(color: p.line)),
      ),
      child: Mono(
        _label(a),
        ls: .12,
        color: value == a ? p.accent : (h ? p.text : p.muted),
        height: null,
      ),
    ),
  );
}
