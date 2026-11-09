import 'package:flutter/material.dart';

import '../../theme/app_theme.dart';
import '../widgets.dart';

/// Mise en page commune d'une page du parcours : grand titre, phrase d'appui, contenu, entrée échelonnée.
class StepLayout extends StatelessWidget {
  final String title;
  final String? lead;
  final List<Widget> children;
  const StepLayout({
    super.key,
    required this.title,
    this.lead,
    this.children = const [],
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final narrow = MediaQuery.sizeOf(context).width < 600;
    final parts = <Widget>[
      Text(
        title,
        style: Fonts.display(size: narrow ? 34 : 52, ls: .02, color: p.text),
      ),
      if (lead != null) ...[
        const SizedBox(height: 18),
        Text(lead!, style: Fonts.sans(size: 17, color: p.dim, height: 1.5)),
      ],
      const SizedBox(height: 28),
      ...children,
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (final (i, w) in parts.indexed)
          FadeSlideIn(
            delay: Duration(milliseconds: 70 * i),
            child: w,
          ),
      ],
    );
  }
}
