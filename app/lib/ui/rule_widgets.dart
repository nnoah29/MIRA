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
  const RuleChip({
    super.key,
    required this.label,
    required this.selected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      builder: (_, h) => AnimatedContainer(
        duration: const Duration(milliseconds: 130),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
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

/// Interrupteur carré, à l'image des autres commandes de l'app.
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
        width: 38,
        height: 20,
        padding: const EdgeInsets.all(3),
        alignment: value ? Alignment.centerRight : Alignment.centerLeft,
        decoration: BoxDecoration(
          color: value ? p.accentSoft : p.bg,
          border: Border.all(color: value ? p.accent : (h ? p.dim : p.line)),
        ),
        child: Container(
          width: 12,
          height: 12,
          color: value ? p.accent : (h ? p.muted : p.dim),
        ),
      ),
    );
  }
}

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
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
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
      constraints: const BoxConstraints(minWidth: 180, maxWidth: 360),
      shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
      onSelected: onChanged,
      itemBuilder: (_) => [
        for (final o in options)
          PopupMenuItem<String>(
            key: o.key,
            value: o.value,
            height: 38,
            child: Text(
              o.label,
              overflow: TextOverflow.ellipsis,
              style: Fonts.sans(
                size: 14,
                color: o.value == value ? p.accent : p.text,
              ),
            ),
          ),
      ],
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
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
