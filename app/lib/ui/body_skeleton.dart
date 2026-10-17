import 'package:flutter/material.dart';

import '../theme/app_theme.dart';

/// Barres grises qui pulsent doucement pendant le chargement du corps d'un message.
class BodySkeleton extends StatefulWidget {
  final double height;
  const BodySkeleton({super.key, this.height = 96});

  @override
  State<BodySkeleton> createState() => _BodySkeletonState();
}

class _BodySkeletonState extends State<BodySkeleton>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 900),
  )..repeat(reverse: true);

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  Widget _bar(Color c, double factor) => FractionallySizedBox(
    widthFactor: factor,
    child: Container(
      height: 12,
      decoration: BoxDecoration(
        color: c,
        borderRadius: BorderRadius.circular(3),
      ),
    ),
  );

  @override
  Widget build(BuildContext context) {
    final c = context.pal.line;
    return SizedBox(
      height: widget.height,
      child: FadeTransition(
        opacity: Tween(begin: .35, end: .8).animate(_c),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [_bar(c, .92), _bar(c, .78), _bar(c, .55)],
        ),
      ),
    );
  }
}
