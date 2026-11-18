//! Services de dictée de l'app : assemblage et accès depuis l'interface.

import 'dart:io';

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/widgets.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'dictation_controller.dart';
import 'model_keeper.dart';
import 'record_audio_source.dart';
import 'sherpa_recognizer.dart';
import 'voice_model_store.dart';
import 'voice_preferences.dart';

/// Ce que l'interface utilise pour dicter : contrôleur, modèles installés, préférences.
class VoiceServices {
  const VoiceServices({
    required this.dictation,
    required this.models,
    required this.preferences,
    this.warm,
  });

  final DictationController dictation;
  final VoiceModelStore models;
  final VoicePreferences preferences;

  /// Réglage de la garde du modèle en mémoire ; absent quand le reconnaisseur n'en garde pas.
  final KeepWarmSetting? warm;
}

/// Rend les services de dictée aux écrans en dessous ; absents, la dictée n'est simplement pas proposée.
class VoiceScope extends InheritedWidget {
  const VoiceScope({super.key, required this.services, required super.child});

  final VoiceServices services;

  @override
  bool updateShouldNotify(VoiceScope old) => services != old.services;
}

/// Services de dictée visibles depuis [context], ou null s'il n'y en a pas.
VoiceServices? voiceServicesOf(BuildContext context) =>
    context.dependOnInheritedWidgetOfExactType<VoiceScope>()?.services;

/// Assemble les vrais services (micro, sherpa-onnx, modèles dans le dossier de l'app) ; null dans un navigateur
/// ou si quoi que ce soit échoue : la dictée n'est alors pas proposée, le reste de l'app démarre.
Future<VoiceServices?> openVoiceServices(
  SharedPreferences prefs, {
  required String uiLocale,
}) async {
  if (kIsWeb) return null;
  try {
    return await _assemble(prefs, uiLocale);
  } on Object {
    return null;
  }
}

/// Construit les services sur le dossier de support de l'app.
Future<VoiceServices> _assemble(
  SharedPreferences prefs,
  String uiLocale,
) async {
  final support = await getApplicationSupportDirectory();
  final store = VoiceModelStore(
    root: Directory('${support.path}/voice-models'),
  );
  final preferences = VoicePreferences(prefs);
  final recognizer = SherpaRecognizer(store, keepWarm: preferences.keepWarm);
  return VoiceServices(
    warm: recognizer,
    dictation: DictationController(
      audio: RecordAudioSource(),
      recognizer: recognizer,
      models: store,
      language: preferences.language(uiLocale: uiLocale),
    ),
    models: store,
    preferences: preferences,
  );
}
