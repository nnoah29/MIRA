import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/resident.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/resident_row.dart';

/// Contrôleur factice : disponibilité et état réglables, changements comptés.
class FakeResident extends ResidentController {
  FakeResident({required this.available, this.enabled = false});
  final bool available;
  bool enabled;
  final changes = <bool>[];

  @override
  Future<bool> isAvailable() async => available;

  @override
  Future<bool> isEnabled() async => enabled;

  @override
  Future<void> enable() async => changes.add(enabled = true);

  @override
  Future<void> disable() async => changes.add(enabled = false);
}

Widget host(ResidentController controller) => MaterialApp(
  theme: buildTheme(false),
  home: Scaffold(body: ResidentRow(controller: controller)),
);

void main() {
  testWidgets('the_row_is_hidden_without_the_unit', (tester) async {
    await tester.pumpWidget(host(FakeResident(available: false)));
    await tester.pumpAndSettle();
    expect(find.byType(SwitchListTile), findsNothing);
  });

  testWidgets('the_switch_shows_and_changes_the_unit_state', (tester) async {
    final resident = FakeResident(available: true, enabled: true);
    await tester.pumpWidget(host(resident));
    await tester.pumpAndSettle();
    final tile = find.byType(SwitchListTile);
    expect(tester.widget<SwitchListTile>(tile).value, isTrue);
    await tester.tap(tile);
    await tester.pumpAndSettle();
    expect(resident.changes, [false]);
    expect(tester.widget<SwitchListTile>(tile).value, isFalse);
  });
}
