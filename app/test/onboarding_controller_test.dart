import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/onboarding/onboarding_controller.dart';

OnboardingController make({
  List<OnboardingStep>? steps,
  void Function()? done,
}) => OnboardingController(
  steps ?? onboardingSteps(ai: true, notifications: true),
  onDone: done ?? () {},
);

void main() {
  test('toutes les étapes dans l’ordre de l’énumération', () {
    expect(
      onboardingSteps(ai: true, notifications: true),
      OnboardingStep.values,
    );
  });

  test('l’étape IA et l’étape notifications se retirent', () {
    final noAi = onboardingSteps(ai: false, notifications: true);
    expect(noAi, isNot(contains(OnboardingStep.ai)));
    final noNotifications = onboardingSteps(ai: true, notifications: false);
    expect(noNotifications, isNot(contains(OnboardingStep.notifications)));
    expect(onboardingSteps(ai: false, notifications: false), hasLength(5));
  });

  test('next avance puis s’arrête à la dernière étape', () {
    final c = make();
    for (var i = 0; i < 10; i++) {
      c.next();
    }
    expect(c.index, OnboardingStep.values.length - 1);
    expect(c.isLast, isTrue);
  });

  test('back s’arrête à la première étape', () {
    final c = make()
      ..next()
      ..back()
      ..back();
    expect(c.index, 0);
    expect(c.isFirst, isTrue);
  });

  test('visitedMax retient l’index le plus loin atteint', () {
    final c = make()
      ..next()
      ..next()
      ..back();
    expect(c.visitedMax, 2);
  });

  test('goTo ignore une étape jamais visitée et rejoint une étape vue', () {
    final c = make();
    c.goTo(3);
    expect(c.index, 0);
    c
      ..next()
      ..next()
      ..next()
      ..goTo(0)
      ..goTo(3);
    expect(c.index, 3);
  });

  test('skip et finish n’appellent onDone qu’une fois', () {
    var calls = 0;
    final c = make(done: () => calls++);
    c.skip();
    c.skip();
    c.finish();
    expect(calls, 1);
  });

  test('chaque changement d’index notifie les écouteurs', () {
    var notified = 0;
    final c = make()..addListener(() => notified++);
    c.next();
    c.back();
    expect(notified, 2);
  });
}
