//! Mémorisation de la langue de dictée choisie par l'utilisateur.

import 'package:shared_preferences/shared_preferences.dart';

import 'dictation_types.dart';

/// Réglages de la dictée enregistrés sur l'appareil.
class VoicePreferences {
  VoicePreferences(this._prefs);

  final SharedPreferences _prefs;
  static const _languageKey = 'voice.language';

  /// Langue choisie, ou celle de l'interface tant que rien n'est enregistré.
  DictationLanguage language({required String uiLocale}) {
    final saved = _prefs.getString(_languageKey);
    for (final l in DictationLanguage.values) {
      if (l.code == saved) return l;
    }
    return DictationLanguage.fromLocale(uiLocale);
  }

  /// Enregistre la langue de dictée.
  Future<void> setLanguage(DictationLanguage language) =>
      _prefs.setString(_languageKey, language.code);
}
