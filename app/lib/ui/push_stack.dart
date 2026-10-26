import 'dart:async';

import 'package:flutter/material.dart';

import '../theme/app_theme.dart';
import 'step_animator.dart';

const _openDuration = Duration(milliseconds: 340);
const _closeDuration = Duration(milliseconds: 280);
const _edgeWidth = 44.0;
const _flingSpeed = 700.0;

/// Page de base recouverte par une page de détail qui glisse par-dessus ; le retour suit le doigt.
class PushStack extends StatefulWidget {
  final Widget base;
  final Widget? detail;
  final VoidCallback onBack;
  const PushStack({
    super.key,
    required this.base,
    required this.detail,
    required this.onBack,
  });

  @override
  State<PushStack> createState() => _PushStackState();
}

class _PushStackState extends State<PushStack> with TickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this);
  late final StepAnimator _anim = StepAnimator(
    this,
    onValue: (v) => _c.value = v,
  );
  Widget? _detail;
  Timer? _later;
  double _target = 0;
  bool _dragging = false, _reduceMotion = false;

  @override
  void initState() {
    super.initState();
    _detail = widget.detail;
    if (_detail != null) _c.value = _anim.value = _target = 1;
    _c.addStatusListener(_released);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _reduceMotion = MediaQuery.disableAnimationsOf(context);
  }

  @override
  void didUpdateWidget(PushStack old) {
    super.didUpdateWidget(old);
    if (widget.detail != null) _detail = widget.detail;
    final want = widget.detail != null ? 1.0 : 0.0;
    if (_dragging || _target == want) return;
    _animateAfterFrame(want);
  }

  @override
  void dispose() {
    _later?.cancel();
    _anim.dispose();
    _c.dispose();
    super.dispose();
  }

  /// Libère le détail une fois la page entièrement ressortie.
  void _released(AnimationStatus status) {
    if (status == AnimationStatus.dismissed && widget.detail == null) {
      setState(() => _detail = null);
    }
  }

  /// Lance l'animation une fois l'image courante terminée : sa construction ne compte pas dans la durée.
  void _animateAfterFrame(double to) {
    _later?.cancel();
    _later = Timer(Duration.zero, () {
      if (mounted) _animate(to);
    });
  }

  /// Anime vers 1 (page ouverte) ou 0 (page fermée), sans durée si les animations sont réduites.
  void _animate(double to) {
    _target = to;
    _anim.animateTo(
      to,
      to == 1 ? _openDuration : _closeDuration,
      instant: _reduceMotion,
    );
  }

  void _dragStart(DragStartDetails d) {
    if (widget.detail == null) return;
    _dragging = true;
    _later?.cancel();
    _anim.stop();
  }

  void _dragUpdate(DragUpdateDetails d, double width) {
    if (!_dragging) return;
    _anim.jumpTo((_c.value - d.delta.dx / width).clamp(0.0, 1.0));
  }

  /// Au relâchement : ferme si le geste est rapide ou a dépassé la moitié, sinon rouvre.
  void _dragEnd(DragEndDetails d) {
    if (!_dragging) return;
    _dragging = false;
    final v = d.velocity.pixelsPerSecond.dx;
    final close = v > _flingSpeed || (v > -_flingSpeed && _c.value < .5);
    if (!close) return _animate(1);
    widget.onBack();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted && widget.detail != null) _animate(1);
    });
  }

  void _dragCancel() {
    if (!_dragging) return;
    _dragging = false;
    _animate(1);
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (_, box) => AnimatedBuilder(
      animation: _c,
      builder: (_, _) => _stack(box.maxWidth, _c.value),
    ),
  );

  /// Superpose la base (décalée et assombrie) et le détail (qui glisse depuis la droite).
  Widget _stack(double w, double t) {
    final p = context.pal;
    return Stack(
      children: [
        Positioned.fill(
          child: Offstage(
            offstage: t >= 1,
            child: Transform.translate(
              offset: Offset(-w * .22 * t, 0),
              child: IgnorePointer(
                ignoring: _detail != null,
                child: RepaintBoundary(child: widget.base),
              ),
            ),
          ),
        ),
        if (t > 0 && t < 1)
          Positioned.fill(
            child: IgnorePointer(
              child: ColoredBox(color: Color.fromRGBO(0, 0, 0, .3 * t)),
            ),
          ),
        if (_detail != null) ...[
          Positioned.fill(
            child: Transform.translate(
              offset: Offset(w * (1 - t), 0),
              child: _page(p, _detail!),
            ),
          ),
          if (widget.detail != null) _edge(w),
        ],
      ],
    );
  }

  /// Page de détail opaque, avec une ombre sur son bord gauche pendant le glissement.
  Widget _page(Pal p, Widget child) => DecoratedBox(
    decoration: BoxDecoration(
      boxShadow: [
        BoxShadow(
          color: const Color(0x59000000),
          blurRadius: 24,
          offset: const Offset(-6, 0),
          blurStyle: BlurStyle.normal,
        ),
      ],
    ),
    child: ColoredBox(
      color: p.bg,
      child: RepaintBoundary(child: child),
    ),
  );

  /// Bande au bord gauche qui capte le glissement de retour sans gêner les touchers.
  Widget _edge(double w) => Positioned(
    left: 0,
    top: 0,
    bottom: 0,
    width: _edgeWidth,
    child: GestureDetector(
      behavior: HitTestBehavior.translucent,
      onHorizontalDragStart: _dragStart,
      onHorizontalDragUpdate: (d) => _dragUpdate(d, w),
      onHorizontalDragEnd: _dragEnd,
      onHorizontalDragCancel: _dragCancel,
    ),
  );
}
