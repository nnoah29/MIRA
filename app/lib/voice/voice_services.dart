//! Services de dictée de l'app : assemblage et accès depuis l'interface.

import 'dart:io';

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/widgets.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'dictation_controller.dart';
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
  });

  final DictationController dictation;
  final VoiceModelStore models;
  final VoicePreferences preferences;
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

/// Assemble les vrais services (micro, sherpa-onnx, modèles dans le dossier de l'app) ; null dans un navigateur.
Future<VoiceServices?> openVoiceServices(
  SharedPreferences prefs, {
  required String uiLocale,
}) async {
  if (kIsWeb) return null;
  final support = await getApplicationSupportDirectory();
  final store = VoiceModelStore(
    root: Directory('${support.path}/voice-models'),
  );
  final preferences = VoicePreferences(prefs);
  return VoiceServices(
    dictation: DictationController(
      audio: RecordAudioSource(),
      recognizer: SherpaRecognizer(store),
      models: store,
      language: preferences.language(uiLocale: uiLocale),
    ),
    models: store,
    preferences: preferences,
  );
}
