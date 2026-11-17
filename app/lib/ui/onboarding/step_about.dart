import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../i18n/i18n.dart';
import '../../state/about_me_state.dart';
import '../../state/store.dart';
import '../about_me_section.dart';
import 'onboarding_controller.dart';
import 'step_layout.dart';

/// Page 6 (optionnelle) : ce que l'assistant doit savoir de l'utilisateur ; enregistré en quittant la page s'il a été modifié.
class AboutStep extends StatefulWidget {
  final OnboardingController controller;
  const AboutStep({super.key, required this.controller});

  @override
  State<AboutStep> createState() => _AboutStepState();
}

class _AboutStepState extends State<AboutStep> {
  late final AboutMeState _state = AboutMeState(
    context.read<MailStore>().aiAdmin!,
  );

  @override
  void initState() {
    super.initState();
    unawaited(_state.load());
  }

  @override
  void dispose() {
    final state = _state;
    if (state.dirty && state.valid) {
      unawaited(state.save().whenComplete(state.dispose));
    } else {
      state.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => StepLayout(
    title: t('À PROPOS DE TOI'),
    lead: t(
      'Facultatif : dis à l’assistant ce qu’il doit toujours savoir de toi. Tu pourras le modifier plus tard dans Paramètres, onglet Mémoire.',
    ),
    children: [AboutMeSection(state: _state)],
  );
}
