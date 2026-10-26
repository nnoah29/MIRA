import 'package:flutter/material.dart';

import 'step_animator.dart';

const _glide = Duration(milliseconds: 280);
const _edgeWidth = 32.0;
const _flingSpeed = 600.0;

/// Tiroir latéral qui suit le doigt : s'ouvre depuis le bord gauche, se ferme en glissant ou en touchant le fond.
class SideDrawer extends StatefulWidget {
  final bool open, edgeEnabled;
  final VoidCallback onOpen, onClose;
  final double width;
  final Widget child;
  const SideDrawer({
    super.key,
    required this.open,
    required this.edgeEnabled,
    required this.onOpen,
    required this.onClose,
    required this.width,
    required this.child,
  });

  @override
  State<SideDrawer> createState() => _SideDrawerState();
}

class _SideDrawerState extends State<SideDrawer> with TickerProviderStateMixin {
  late final ValueNotifier<double> _t = ValueNotifier(widget.open ? 1 : 0);
  late final StepAnimator _anim = StepAnimator(
    this,
    value: _t.value,
    onValue: (v) => _t.value = v,
  );
  bool _dragging = false, _reduceMotion = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _reduceMotion = MediaQuery.disableAnimationsOf(context);
  }

  @override
  void didUpdateWidget(SideDrawer old) {
    super.didUpdateWidget(old);
    if (old.open != widget.open && !_dragging) _toward(widget.open);
  }

  @override
  void dispose() {
    _anim.dispose();
    _t.dispose();
    super.dispose();
  }

  void _toward(bool open) =>
      _anim.animateTo(open ? 1 : 0, _glide, instant: _reduceMotion);

  void _dragStart(DragStartDetails d) {
    _dragging = true;
    _anim.stop();
  }

  void _dragUpdate(DragUpdateDetails d) {
    if (!_dragging) return;
    _anim.jumpTo((_t.value + d.delta.dx / widget.width).clamp(0.0, 1.0));
  }

  /// Au relâchement : geste rapide ou passé la moitié, on suit son sens ; sinon on revient.
  void _dragEnd(DragEndDetails d) {
    if (!_dragging) return;
    _dragging = false;
    final v = d.velocity.pixelsPerSecond.dx;
    final wantOpen = v.abs() > _flingSpeed ? v > 0 : _t.value > .5;
    if (wantOpen == widget.open) return _toward(wantOpen);
    wantOpen ? widget.onOpen() : widget.onClose();
  }

  void _dragCancel() {
    if (!_dragging) return;
    _dragging = false;
    _toward(widget.open);
  }

  @override
  Widget build(BuildContext context) => ValueListenableBuilder<double>(
    valueListenable: _t,
    builder: (_, t, _) => Stack(
      children: [
        if (t > 0) Positioned.fill(child: _scrim(t)),
        Positioned(
          left: -widget.width * (1 - t),
          top: 0,
          bottom: 0,
          width: widget.width,
          child: _draggable(widget.child),
        ),
        if (t == 0 && widget.edgeEnabled && !widget.open) _edge(),
      ],
    ),
  );

  /// Fond assombri qui suit l'ouverture ; un toucher ou un glissement le referme.
  Widget _scrim(double t) => _draggable(
    GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: widget.onClose,
      child: ColoredBox(color: Color.fromRGBO(0, 0, 0, .5 * t)),
    ),
  );

  /// Bande au bord gauche qui capte le glissement d'ouverture sans gêner les touchers.
  Widget _edge() => Positioned(
    left: 0,
    top: 0,
    bottom: 0,
    width: _edgeWidth,
    child: _draggable(const SizedBox.expand(), translucent: true),
  );

  Widget _draggable(Widget child, {bool translucent = false}) =>
      GestureDetector(
        behavior: translucent ? HitTestBehavior.translucent : null,
        onHorizontalDragStart: _dragStart,
        onHorizontalDragUpdate: _dragUpdate,
        onHorizontalDragEnd: _dragEnd,
        onHorizontalDragCancel: _dragCancel,
        child: child,
      );
}
