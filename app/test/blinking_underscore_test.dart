import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/blinking_underscore.dart';

Widget host() => MaterialApp(
  theme: buildTheme(false),
  home: const Scaffold(body: Center(child: BlinkingUnderscore())),
);

double opacity(WidgetTester tester) => tester
    .widget<Opacity>(
      find.descendant(
        of: find.byType(BlinkingUnderscore),
        matching: find.byType(Opacity),
      ),
    )
    .opacity;

void main() {
  testWidgets('clignote une fois la demi-période écoulée', (tester) async {
    await tester.pumpWidget(host());
    final first = opacity(tester);
    await tester.pump(const Duration(milliseconds: 540));
    expect(opacity(tester), isNot(first));
    await tester.pump(const Duration(milliseconds: 540));
    expect(opacity(tester), first);
  });

  testWidgets('au repos entre deux bascules, aucune image n’est demandée', (
    tester,
  ) async {
    await tester.pumpWidget(host());
    await tester.pump(const Duration(milliseconds: 540));
    await tester.pump(const Duration(milliseconds: 10));
    expect(SchedulerBinding.instance.hasScheduledFrame, isFalse);
  });

  testWidgets('fenêtre sans focus : plus de clignotement ni de minuteur', (
    tester,
  ) async {
    await tester.pumpWidget(host());
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    await tester.pump();
    final frozen = opacity(tester);
    await tester.pump(const Duration(seconds: 5));
    expect(opacity(tester), frozen);
    expect(SchedulerBinding.instance.hasScheduledFrame, isFalse);
  });

  testWidgets('le focus revient : le clignotement reprend', (tester) async {
    await tester.pumpWidget(host());
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    await tester.pump();
    final frozen = opacity(tester);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 540));
    expect(opacity(tester), isNot(frozen));
  });

  testWidgets('retiré de l’écran, le minuteur est libéré', (tester) async {
    await tester.pumpWidget(host());
    await tester.pumpWidget(const SizedBox());
    await tester.pump(const Duration(seconds: 2));
  });
}
