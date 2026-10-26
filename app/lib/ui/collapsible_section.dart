import 'package:flutter/material.dart';

import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Section d'une page de réglages dont le titre bascule l'affichage du contenu.
class CollapsibleSection extends StatefulWidget {
  final String title;
  final Widget child;
  final bool initiallyOpen;
  const CollapsibleSection({
    super.key,
    required this.title,
    required this.child,
    this.initiallyOpen = false,
  });

  @override
  State<CollapsibleSection> createState() => _CollapsibleSectionState();
}

class _CollapsibleSectionState extends State<CollapsibleSection> {
  late bool open = widget.initiallyOpen;
  late bool built = widget.initiallyOpen;

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Hov(
          onTap: () => setState(() {
            open = !open;
            built = true;
          }),
          builder: (_, h) => Container(
            padding: const EdgeInsets.symmetric(vertical: 14),
            decoration: BoxDecoration(
              border: Border(bottom: BorderSide(color: p.lineSoft)),
            ),
            child: Row(
              children: [
                Expanded(
                  child: Mono(
                    widget.title,
                    size: 10.5,
                    ls: .12,
                    color: open ? p.accent : (h ? p.text : p.muted),
                    height: null,
                  ),
                ),
                AnimatedRotation(
                  turns: open ? .5 : 0,
                  duration: const Duration(milliseconds: 150),
                  child: Ico('chev', color: p.muted),
                ),
              ],
            ),
          ),
        ),
        if (open)
          Padding(padding: const EdgeInsets.only(top: 16), child: widget.child),
      ],
    );
  }
}
