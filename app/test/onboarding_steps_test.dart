import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/resident.dart';
import 'package:mira/ui/onboarding/step_notifications.dart';
import 'package:mira/ui/resident_prompt.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'onboarding_flow_test.dart' show pumpFlow, tapKey;

Future<void> goTo(WidgetTester tester, int page) async {
  for (var i = 0; i < page; i++) {
    await tapKey(tester, 'onb-next');
  }
}

void main() {
  notificationTests();
  testWidgets('appearance_cards_apply_theme_immediately', (tester) async {
    final store = await pumpFlow(tester);
    await goTo(tester, 2);
    await tapKey(tester, 'onb-theme-light');
    expect(store.theme, 'light');
    expect(store.prefs.getString('theme'), 'light');
    await tapKey(tester, 'onb-theme-dark');
    expect(store.theme, 'dark');
  });

  testWidgets('appearance_marks_current_theme', (tester) async {
    await pumpFlow(tester);
    await goTo(tester, 2);
    expect(find.byKey(const ValueKey('onb-theme-dark-on')), findsOneWidget);
    expect(find.byKey(const ValueKey('onb-theme-light-on')), findsNothing);
    await tapKey(tester, 'onb-theme-light');
    expect(find.byKey(const ValueKey('onb-theme-light-on')), findsOneWidget);
    expect(find.byKey(const ValueKey('onb-theme-dark-on')), findsNothing);
  });
}

/// Contrôleur du mode résident factice : disponibilité réglable, appels gardés.
class StepResident extends ResidentController {
  StepResident({this.available = true});
  final bool available;
  bool on = false;
  int enables = 0, disables = 0;

  @override
  Future<bool> isAvailable() async => available;

  @override
  Future<bool> isEnabled() async => on;

  @override
  Future<void> enable() async {
    enables++;
    on = true;
  }

  @override
  Future<void> disable() async {
    disables++;
    on = false;
  }
}

void useResident(StepResident fake) {
  final real = residentForOnboarding;
  addTearDown(() => residentForOnboarding = real);
  residentForOnboarding = () => fake;
}

void notificationTests() {
  testWidgets('notifications_card_toggles_setting', (tester) async {
    useResident(StepResident(available: false));
    final store = await pumpFlow(tester);
    await goTo(tester, 5);
    final before = store.notifySettings.enabled;
    await tapKey(tester, 'notify-${before ? '0' : '1'}');
    expect(store.notifySettings.enabled, !before);
  });

  testWidgets('resident_card_hidden_when_unavailable', (tester) async {
    useResident(StepResident(available: false));
    await pumpFlow(tester);
    await goTo(tester, 5);
    expect(find.byKey(const ValueKey('onb-resident')), findsNothing);
  });

  testWidgets('resident_card_enables_and_marks_answered', (tester) async {
    final fake = StepResident();
    useResident(fake);
    final store = await pumpFlow(tester);
    await goTo(tester, 5);
    expect(find.byKey(const ValueKey('onb-resident')), findsOneWidget);
    await tapKey(tester, 'resident-1');
    expect(fake.enables, 1);
    expect(store.prefs.getString(residentAnsweredKey), '1');
    await tapKey(tester, 'resident-0');
    expect(fake.disables, 1);
    expect(store.prefs.getString(residentAnsweredKey), '1');
  });

  testWidgets('resident_prompt_not_shown_after_onboarding_answer', (
    tester,
  ) async {
    SharedPreferences.setMockInitialValues({residentAnsweredKey: '1'});
    final prefs = await SharedPreferences.getInstance();
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => TextButton(
            onPressed: () => askResidentOnce(
              context: context,
              prefs: prefs,
              controller: StepResident(),
            ),
            child: const Text('go'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('go'));
    await tester.pump(const Duration(milliseconds: 300));
    expect(find.byKey(const ValueKey('resident-dialog')), findsNothing);
  });
}
