import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../i18n/i18n.dart';
import '../../state/store.dart';
import 'onboarding_controller.dart';
import 'onboarding_frame.dart';
import 'step_privacy.dart';
import 'step_ready.dart';
import 'step_welcome.dart';

/// Page de l'étape ; les étapes pas encore écrites ne montrent rien.
Widget stepPage(OnboardingStep step, OnboardingController c) => switch (step) {
  OnboardingStep.welcome => WelcomeStep(controller: c),
  OnboardingStep.privacy => PrivacyStep(controller: c),
  OnboardingStep.appearance => const SizedBox.shrink(),
  OnboardingStep.account => const SizedBox.shrink(),
  OnboardingStep.ai => const SizedBox.shrink(),
  OnboardingStep.notifications => const SizedBox.shrink(),
  OnboardingStep.ready => ReadyStep(controller: c),
};

/// Parcours du premier lancement : pages plein écran, ferme l'accueil du magasin à la fin ou au passage.
class OnboardingFlow extends StatefulWidget {
  const OnboardingFlow({super.key});

  @override
  State<OnboardingFlow> createState() => _OnboardingFlowState();
}

class _OnboardingFlowState extends State<OnboardingFlow> {
  late final OnboardingController _controller;

  @override
  void initState() {
    super.initState();
    final store = context.read<MailStore>();
    _controller = OnboardingController(
      onboardingSteps(ai: store.aiAdmin != null, notifications: true),
      onDone: () => store.finishWelcome(addAccount: false),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  String? _nextLabel(OnboardingStep s) =>
      s == OnboardingStep.welcome ? t('Commencer') : null;

  @override
  Widget build(BuildContext context) {
    context.select<MailStore, AppLang>((s) => s.lang);
    return ListenableBuilder(
      listenable: _controller,
      builder: (_, _) {
        final step = _controller.step;
        return OnboardingFrame(
          controller: _controller,
          showNext: step != OnboardingStep.ready,
          nextLabel: _nextLabel(step),
          child: AnimatedSwitcher(
            duration: const Duration(milliseconds: 250),
            child: KeyedSubtree(
              key: ValueKey(step),
              child: stepPage(step, _controller),
            ),
          ),
        );
      },
    );
  }
}
