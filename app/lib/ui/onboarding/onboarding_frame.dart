import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../i18n/i18n.dart';
import '../../theme/app_theme.dart';
import '../mira_logo.dart';
import '../widgets.dart';
import 'onboarding_controller.dart';

/// Cadre d'une page du parcours : fond animé, passage, points de progression, retour et suivant, clavier.
class OnboardingFrame extends StatefulWidget {
  final OnboardingController controller;
  final Widget child;
  final bool showNext;
  final String? nextLabel;
  const OnboardingFrame({
    super.key,
    required this.controller,
    required this.child,
    this.showNext = true,
    this.nextLabel,
  });

  @override
  State<OnboardingFrame> createState() => _OnboardingFrameState();
}

class _OnboardingFrameState extends State<OnboardingFrame>
    with SingleTickerProviderStateMixin {
  late final AnimationController _halo = AnimationController(
    vsync: this,
    duration: const Duration(seconds: 14),
  )..repeat(reverse: true);

  @override
  void dispose() {
    _halo.dispose();
    super.dispose();
  }

  bool _typing() {
    final ctx = FocusManager.instance.primaryFocus?.context;
    if (ctx == null) return false;
    return ctx.widget is EditableText ||
        ctx.findAncestorWidgetOfExactType<EditableText>() != null;
  }

  KeyEventResult _key(FocusNode _, KeyEvent e) {
    if (e is! KeyDownEvent || _typing()) return KeyEventResult.ignored;
    final c = widget.controller;
    final key = e.logicalKey;
    if (key == LogicalKeyboardKey.arrowRight) {
      c.next();
    } else if (key == LogicalKeyboardKey.arrowLeft) {
      c.back();
    } else if (key == LogicalKeyboardKey.enter ||
        key == LogicalKeyboardKey.numpadEnter) {
      c.isLast ? c.finish() : c.next();
    } else if (key == LogicalKeyboardKey.escape) {
      c.skip();
    } else {
      return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  Widget _background(Pal p) => AnimatedBuilder(
    animation: _halo,
    builder: (_, _) => DecoratedBox(
      decoration: BoxDecoration(
        gradient: RadialGradient(
          center: Alignment(-0.6 + _halo.value * 1.2, -0.7 + _halo.value * 0.5),
          radius: 1.1,
          colors: [p.accentSoft, p.bg],
        ),
      ),
      child: const SizedBox.expand(),
    ),
  );

  Widget _top(Pal p) => Row(
    children: [
      MiraLogo(color: p.accent, size: 26),
      const Spacer(),
      Btn(
        key: const ValueKey('onb-skip'),
        label: t('Passer'),
        onTap: widget.controller.skip,
      ),
    ],
  );

  Widget _dots(Pal p) {
    final c = widget.controller;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (var i = 0; i < c.steps.length; i++)
          Hov(
            key: ValueKey('onb-dot-$i'),
            onTap: () => c.goTo(i),
            builder: (_, _) => Padding(
              padding: const EdgeInsets.all(7),
              child: Container(
                width: 9,
                height: 9,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: i == c.index
                      ? p.accent
                      : (i <= c.visitedMax ? p.muted : Colors.transparent),
                  border: Border.all(color: i == c.index ? p.accent : p.dim),
                ),
              ),
            ),
          ),
      ],
    );
  }

  Widget _back() => widget.controller.isFirst
      ? const SizedBox.shrink()
      : Btn(
          key: const ValueKey('onb-back'),
          label: t('Retour'),
          onTap: widget.controller.back,
        );

  Widget _next() => widget.showNext && !widget.controller.isLast
      ? Btn(
          key: const ValueKey('onb-next'),
          label: widget.nextLabel ?? t('Suivant'),
          go: true,
          onTap: widget.controller.next,
        )
      : const SizedBox.shrink();

  Widget _bottom(Pal p) {
    final narrow = MediaQuery.sizeOf(context).width < 560;
    if (narrow) {
      return Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _dots(p),
          const SizedBox(height: 8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [_back(), _next()],
          ),
        ],
      );
    }
    return Row(
      children: [
        Expanded(
          child: Align(alignment: Alignment.centerLeft, child: _back()),
        ),
        _dots(p),
        Expanded(
          child: Align(alignment: Alignment.centerRight, child: _next()),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Focus(
      autofocus: true,
      onKeyEvent: _key,
      child: Scaffold(
        backgroundColor: p.bg,
        body: Stack(
          children: [
            Positioned.fill(child: _background(p)),
            SafeArea(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(24, 16, 24, 16),
                child: Column(
                  children: [
                    _top(p),
                    Expanded(
                      child: Center(
                        child: SingleChildScrollView(
                          padding: const EdgeInsets.symmetric(vertical: 16),
                          child: ConstrainedBox(
                            constraints: const BoxConstraints(maxWidth: 720),
                            child: widget.child,
                          ),
                        ),
                      ),
                    ),
                    _bottom(p),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
