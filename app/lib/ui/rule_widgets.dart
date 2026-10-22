import 'package:flutter/material.dart';

import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Intitulé de section de l'éditeur de règles : capitales mono, discret.
class RuleLabel extends StatelessWidget {
  final String text;
  final EdgeInsets padding;
  const RuleLabel(
    this.text, {
    super.key,
    this.padding = const EdgeInsets.only(top: 22, bottom: 10),
  });

  @override
  Widget build(BuildContext context) => Padding(
    padding: padding,
    child: Mono(text, size: 10.5, ls: .2, height: null),
  );
}

/// Puce carrée à deux états (portée, inversion « NON ») : teinte d'accent quand elle est active.
class RuleChip extends StatelessWidget {
  final String label;
  final bool selected;
  final VoidCallback onTap;
  final double? height;
  const RuleChip({
    super.key,
    required this.label,
    required this.selected,
    required this.onTap,
    this.height,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      builder: (_, h) => AnimatedContainer(
        duration: const Duration(milliseconds: 130),
        height: height,
        alignment: height == null ? null : Alignment.center,
        padding: EdgeInsets.symmetric(
          horizontal: 12,
          vertical: height == null ? 8 : 0,
        ),
        decoration: BoxDecoration(
          color: selected ? p.accentSoft : (h ? p.surface : null),
          border: Border.all(color: selected ? p.accent : (h ? p.dim : p.line)),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (selected) ...[
              Ico('check', size: 12, color: p.accent),
              const SizedBox(width: 8),
            ],
            Text(
              label,
              style: Fonts.mono(
                size: 12,
                ls: .04,
                height: null,
                color: selected ? p.accent : (h ? p.text : p.muted),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Interrupteur carré : plein à l'accent quand il est actif, simple filet sinon.
class RuleToggle extends StatelessWidget {
  final bool value;
  final ValueChanged<bool> onChanged;
  final String? tip;
  const RuleToggle({
    super.key,
    required this.value,
    required this.onChanged,
    this.tip,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      tip: tip,
      onTap: () => onChanged(!value),
      builder: (_, h) => AnimatedContainer(
        duration: const Duration(milliseconds: 130),
        width: 44,
        height: 24,
        padding: const EdgeInsets.all(3),
        alignment: value ? Alignment.centerRight : Alignment.centerLeft,
        decoration: BoxDecoration(
          color: value ? p.accent : p.bg,
          border: Border.all(color: value ? p.accent : (h ? p.dim : p.line)),
        ),
        child: Container(
          width: 16,
          height: 16,
          color: value ? p.onAccent : (h ? p.muted : p.dim),
        ),
      ),
    );
  }
}

/// Choix oui / non à deux cases, comme les choix segmentés des autres réglages.
class RuleSegment extends StatelessWidget {
  final bool value;
  final ValueChanged<bool> onChanged;
  final String yes;
  final String no;
  const RuleSegment({
    super.key,
    required this.value,
    required this.onChanged,
    required this.yes,
    required this.no,
  });

  Widget _cell(BuildContext context, String label, bool on, bool first) {
    final p = context.pal;
    final selected = value == on;
    return Hov(
      onTap: () => onChanged(on),
      builder: (_, h) => Container(
        height: ruleControlHeight - 2,
        alignment: Alignment.center,
        padding: const EdgeInsets.symmetric(horizontal: 16),
        decoration: BoxDecoration(
          color: selected ? p.accentSoft : null,
          border: first ? null : Border(left: BorderSide(color: p.line)),
        ),
        child: Mono(
          label,
          ls: .12,
          height: null,
          color: selected ? p.accent : (h ? p.text : p.muted),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      decoration: BoxDecoration(
        color: p.bg,
        border: Border.all(color: p.line),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _cell(context, yes, true, true),
          _cell(context, no, false, false),
        ],
      ),
    );
  }
}

/// Hauteur commune des champs, listes et puces d'une même rangée.
const double ruleControlHeight = 40;

/// Zone de saisie à filet carré, comme les champs des autres réglages.
class RuleField extends StatelessWidget {
  final Key? fieldKey;
  final String initialValue;
  final ValueChanged<String> onChanged;
  final double? width;
  final String? hint;
  final bool number;
  const RuleField({
    super.key,
    this.fieldKey,
    required this.initialValue,
    required this.onChanged,
    this.width,
    this.hint,
    this.number = false,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      width: width,
      height: ruleControlHeight,
      alignment: Alignment.center,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: p.bg,
        border: Border.all(color: p.line),
      ),
      child: TextFormField(
        key: fieldKey,
        initialValue: initialValue,
        onChanged: onChanged,
        keyboardType: number ? TextInputType.number : TextInputType.text,
        cursorColor: p.accent,
        cursorWidth: 1,
        style: Fonts.sans(size: 14, color: p.text),
        decoration: InputDecoration(
          isDense: true,
          border: InputBorder.none,
          enabledBorder: InputBorder.none,
          focusedBorder: InputBorder.none,
          contentPadding: EdgeInsets.zero,
          hintText: hint,
          hintStyle: Fonts.sans(size: 14, color: p.dim),
        ),
      ),
    );
  }
}

/// Une entrée d'un [RuleSelect].
class RuleOption {
  final String value;
  final String label;
  final Key? key;
  const RuleOption(this.value, this.label, {this.key});
}

/// Liste déroulante carrée : champ à filet, menu à filet sur le fond latéral.
class RuleSelect extends StatelessWidget {
  final String? value;
  final String hint;
  final List<RuleOption> options;
  final ValueChanged<String> onChanged;
  const RuleSelect({
    super.key,
    required this.value,
    required this.hint,
    required this.options,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final shown = options.where((o) => o.value == value).firstOrNull;
    return PopupMenuButton<String>(
      tooltip: '',
      padding: EdgeInsets.zero,
      color: p.bgSide,
      elevation: 0,
      position: PopupMenuPosition.under,
      constraints: const BoxConstraints(
        minWidth: 200,
        maxWidth: 340,
        maxHeight: 300,
      ),
      shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
      onSelected: onChanged,
      itemBuilder: (_) => [
        for (final o in options)
          PopupMenuItem<String>(
            key: o.key,
            value: o.value,
            height: 32,
            child: Text(
              o.label,
              overflow: TextOverflow.ellipsis,
              style: Fonts.sans(
                size: 13.5,
                color: o.value == value ? p.accent : p.text,
              ),
            ),
          ),
      ],
      child: Container(
        height: ruleControlHeight,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: BoxDecoration(
          color: p.bg,
          border: Border.all(color: p.line),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Flexible(
              child: Text(
                shown?.label ?? hint,
                overflow: TextOverflow.ellipsis,
                style: Fonts.sans(
                  size: 14,
                  color: shown == null ? p.dim : p.text,
                ),
              ),
            ),
            const SizedBox(width: 10),
            Ico('chev', size: 14, color: p.muted),
          ],
        ),
      ),
    );
  }
}

/// Cadre d'une alternative (blocs « ET » empilés).
class RuleBlock extends StatelessWidget {
  final Widget child;
  const RuleBlock({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: p.surface,
        border: Border.all(color: p.lineSoft),
      ),
      child: child,
    );
  }
}

/// Séparateur « OU » entre deux alternatives : filet, étiquette d'accent, filet.
class RuleOrDivider extends StatelessWidget {
  final String label;
  const RuleOrDivider(this.label, {super.key});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final line = Expanded(child: Container(height: 1, color: p.lineSoft));
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 10),
      child: Row(
        children: [
          line,
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14),
            child: Mono(
              label,
              size: 10.5,
              ls: .24,
              height: null,
              color: p.accent,
            ),
          ),
          line,
        ],
      ),
    );
  }
}

/// Zone à hauteur bornée qui défile au-delà ; garde sa taille naturelle quand elle tient.
class RuleScroll extends StatelessWidget {
  final double maxHeight;
  final ScrollController controller;
  final Widget child;
  const RuleScroll({
    super.key,
    required this.maxHeight,
    required this.controller,
    required this.child,
  });

  @override
  Widget build(BuildContext context) => ConstrainedBox(
    constraints: BoxConstraints(maxHeight: maxHeight),
    child: Scrollbar(
      controller: controller,
      child: SingleChildScrollView(
        controller: controller,
        padding: const EdgeInsets.only(right: 12),
        child: child,
      ),
    ),
  );
}
