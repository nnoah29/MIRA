import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/onboarding/onboarding_controller.dart';

void main() {
  test('l’étape À propos de toi suit l’IA et disparaît sans elle', () {
    final steps = onboardingSteps(ai: true, notifications: true);
    expect(steps[steps.indexOf(OnboardingStep.ai) + 1], OnboardingStep.about);
    final noAi = onboardingSteps(ai: false, notifications: true);
    expect(noAi, isNot(contains(OnboardingStep.about)));
  });
}
