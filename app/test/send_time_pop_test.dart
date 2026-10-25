import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/send_time_pop.dart';

Widget host(ValueChanged<DateTime> onPick, DateTime initial) => MaterialApp(
  theme: buildTheme(true),
  home: Scaffold(
    body: Center(
      child: SendTimePicker(onPick: onPick, initial: initial),
    ),
  ),
);

void main() {
  setUpAll(() => initializeDateFormatting());

  test('l’heure proposée est dans une heure, arrondie aux 5 minutes', () {
    final at = defaultSendTime(DateTime(2030, 1, 1, 10, 2));
    expect(at, DateTime(2030, 1, 1, 11, 5));
  });

  testWidgets('choisir un jour et une heure renvoie l’instant exact', (
    tester,
  ) async {
    DateTime? picked;
    final start = DateTime.now().add(const Duration(days: 40));
    await tester.pumpWidget(host((d) => picked = d, start));
    await tester.tap(find.text('15').first);
    await tester.tap(find.text('PROGRAMMER'));
    expect(picked?.day, 15);
    expect(picked?.hour, start.hour);
  });

  testWidgets('un instant passé désactive le bouton', (tester) async {
    DateTime? picked;
    final past = DateTime.now().subtract(const Duration(hours: 2));
    await tester.pumpWidget(host((d) => picked = d, past));
    await tester.tap(find.text('PROGRAMMER'), warnIfMissed: false);
    expect(picked, isNull);
  });
}
