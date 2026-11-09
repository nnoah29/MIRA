import 'package:flutter/foundation.dart';

/// Pages du parcours d'accueil, dans leur ordre d'affichage.
enum OnboardingStep {
  welcome,
  privacy,
  appearance,
  account,
  ai,
  notifications,
  ready,
}

/// Étapes à montrer : toutes, sans l'IA ou sans les notifications quand elles ne s'appliquent pas.
List<OnboardingStep> onboardingSteps({
  required bool ai,
  required bool notifications,
}) => [
  for (final s in OnboardingStep.values)
    if ((s != OnboardingStep.ai || ai) &&
        (s != OnboardingStep.notifications || notifications))
      s,
];

/// État du parcours : étape courante, navigation, fin ; `onDone` n'est appelé qu'une fois.
class OnboardingController extends ChangeNotifier {
  final List<OnboardingStep> steps;
  final VoidCallback onDone;
  int _index = 0;
  int _visitedMax = 0;
  bool _done = false;

  OnboardingController(this.steps, {required this.onDone});

  int get index => _index;
  int get visitedMax => _visitedMax;
  OnboardingStep get step => steps[_index];
  bool get isFirst => _index == 0;
  bool get isLast => _index == steps.length - 1;

  void _move(int to) {
    _index = to;
    if (to > _visitedMax) _visitedMax = to;
    notifyListeners();
  }

  /// Page suivante ; sans effet sur la dernière.
  void next() {
    if (!isLast) _move(_index + 1);
  }

  /// Page précédente ; sans effet sur la première.
  void back() {
    if (!isFirst) _move(_index - 1);
  }

  /// Rejoint une page déjà vue ; une page jamais atteinte est ignorée.
  void goTo(int i) {
    if (i >= 0 && i <= _visitedMax && i != _index) _move(i);
  }

  /// Quitte le parcours depuis n'importe quelle page.
  void skip() => finish();

  /// Termine le parcours ; `onDone` n'est appelé qu'une fois.
  void finish() {
    if (_done) return;
    _done = true;
    onDone();
  }
}
