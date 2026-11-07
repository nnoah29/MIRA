import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/resident.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/resident_prompt.dart';
import 'package:mira/ui/widgets.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Contrôleur factice : disponibilité réglable, appels comptés.
class FakeResident extends ResidentController {
  FakeResident({this.available = true});
  final bool available;
  int enables = 0;

  @override
  Future<bool> isAvailable() async => available;

  @override
  Future<void> enable() async => enables++;
}

/// Affiche une page qui pose la question au premier appel de [ask].
Future<void> pump(
  WidgetTester tester,
  Future<void> Function(BuildContext) ask,
) {
  return tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Builder(
        builder: (context) =>
            TextButton(onPressed: () => ask(context), child: const Text('go')),
      ),
    ),
  );
}

void main() {
  testWidgets('yes_enables_the_unit_and_remembers_the_answer', (tester) async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final resident = FakeResident();
    await pump(
      tester,
      (c) => askResidentOnce(context: c, prefs: prefs, controller: resident),
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
    expect(find.text('Garder MIRA actif en arrière-plan ?'), findsOneWidget);
    await tester.tap(find.byType(Btn).last);
    await tester.pumpAndSettle();
    expect(resident.enables, 1);
    expect(prefs.getString('residentAnswered'), '1');
  });

  testWidgets('no_does_not_enable_but_remembers_the_answer', (tester) async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final resident = FakeResident();
    await pump(
      tester,
      (c) => askResidentOnce(context: c, prefs: prefs, controller: resident),
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
    await tester.tap(find.byType(Btn).first);
    await tester.pumpAndSettle();
    expect(resident.enables, 0);
    expect(prefs.getString('residentAnswered'), '1');
  });

  testWidgets('the_question_uses_the_app_dialog_and_buttons', (tester) async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    await pump(
      tester,
      (c) =>
          askResidentOnce(context: c, prefs: prefs, controller: FakeResident()),
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('resident-dialog')), findsOneWidget);
    expect(find.byType(AlertDialog), findsNothing);
    expect(find.byType(FilledButton), findsNothing);
    expect(find.byType(Btn), findsNWidgets(2));
  });

  testWidgets('the_question_is_asked_only_once', (tester) async {
    SharedPreferences.setMockInitialValues({'residentAnswered': '1'});
    final prefs = await SharedPreferences.getInstance();
    await pump(
      tester,
      (c) =>
          askResidentOnce(context: c, prefs: prefs, controller: FakeResident()),
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
    expect(find.text('Garder MIRA actif en arrière-plan ?'), findsNothing);
  });

  testWidgets('nothing_is_asked_when_the_unit_is_not_installed', (
    tester,
  ) async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    await pump(
      tester,
      (c) => askResidentOnce(
        context: c,
        prefs: prefs,
        controller: FakeResident(available: false),
      ),
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
    expect(find.text('Garder MIRA actif en arrière-plan ?'), findsNothing);
    expect(prefs.getString('residentAnswered'), isNull);
  });
}
