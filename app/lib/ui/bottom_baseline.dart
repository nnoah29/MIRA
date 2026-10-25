import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';

//! Une image en ligne pose son bord bas sur la ligne de base du texte, comme dans un navigateur.

/// Donne à son enfant (une image) le bas de sa boîte pour ligne de base, y compris au calcul à sec des tableaux.
class BottomBaseline extends SingleChildRenderObjectWidget {
  const BottomBaseline({super.key, super.child});

  @override
  RenderObject createRenderObject(BuildContext context) =>
      RenderBottomBaseline();
}

class RenderBottomBaseline extends RenderProxyBox {
  @override
  double? computeDryBaseline(
    BoxConstraints constraints,
    TextBaseline baseline,
  ) => child?.getDryLayout(constraints).height;

  @override
  double? computeDistanceToActualBaseline(TextBaseline baseline) =>
      child == null ? null : size.height;
}
