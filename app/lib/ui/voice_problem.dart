//! Message montré à l'utilisateur quand la dictée ne peut pas démarrer ou s'arrête sur une erreur.

import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';

import '../i18n/i18n.dart';
import '../state/store.dart';
import '../voice/dictation_types.dart';
import '../voice/voice_services.dart';
import 'voice_model_dialog.dart';

/// Texte de la raison d'un échec de dictée.
String voiceProblemText(DictationProblem problem) => switch (problem) {
  DictationProblem.modelMissing => t(
    'Modèle de dictée absent : installez-le dans Réglages › Dictée vocale',
  ),
  DictationProblem.microphoneDenied => t(
    'Accès au micro refusé : autorisez-le dans les réglages du système',
  ),
  DictationProblem.microphoneUnavailable => t(
    'Micro introuvable ou indisponible',
  ),
  DictationProblem.recognizerFailed => t('La dictée a rencontré une erreur'),
};

/// Réagit à un échec de dictée : modèle absent, on propose de le télécharger ; sinon on en donne la raison.
void reportVoiceProblem(BuildContext context, DictationProblem problem) {
  final services = voiceServicesOf(context);
  final store = context.read<MailStore>();
  if (problem == DictationProblem.modelMissing && services != null) {
    offerVoiceModelInstall(
      context,
      services,
      notify: (message, {bool err = false}) => store.toast(message, err: err),
    );
    return;
  }
  store.toast(voiceProblemText(problem), err: true);
}
