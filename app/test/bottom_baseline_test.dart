import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/bottom_baseline.dart';

const _png =
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==';
const _loose = BoxConstraints(maxWidth: 300);

Image _image() => Image.memory(base64Decode(_png), width: 30, height: 30);

RenderBox _boxOf(WidgetTester tester, Finder finder) =>
    tester.renderObject<RenderBox>(finder);

void main() {
  testWidgets('une image seule ne sait pas donner sa ligne de base à sec', (
    tester,
  ) async {
    await tester.pumpWidget(
      Directionality(textDirection: TextDirection.ltr, child: _image()),
    );
    expect(
      () => _boxOf(
        tester,
        find.byType(Image),
      ).getDryBaseline(_loose, TextBaseline.alphabetic),
      throwsFlutterError,
    );
  });

  testWidgets('enveloppée, son bas sert de ligne de base à sec', (
    tester,
  ) async {
    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: BottomBaseline(child: _image()),
      ),
    );
    final box = _boxOf(tester, find.byType(BottomBaseline));
    expect(box.getDryBaseline(_loose, TextBaseline.alphabetic), 30);
  });
}
