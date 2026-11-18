//! Réglages de la dictée enregistrés sur l'appareil : langue, Espace maintenu, garde du modèle.

import 'package:shared_preferences/shared_preferences.dart';

import 'dictation_types.dart';

const _defaultHoldDelay = Duration(milliseconds: 400);
const _defaultKeepWarm = Duration(minutes: 3);
const _minHoldMs = 150;
const _maxHoldMs = 1500;
const _maxKeepWarmMinutes = 60;

/// Réglages de la dictée enregistrés sur l'appareil.
class VoicePreferences {
  VoicePreferences(this._prefs);

  final SharedPreferences _prefs;
  static const _languageKey = 'voice.language';
  static const _holdSpaceKey = 'voice.holdSpace';
  static const _holdDelayKey = 'voice.holdDelayMs';
  static const _keepWarmKey = 'voice.keepWarmMinutes';

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

  /// Dit si maintenir Espace dicte phrase par phrase (actif tant que rien n'est enregistré).
  bool get holdSpaceEnabled {
    final saved = _prefs.get(_holdSpaceKey);
    return saved is bool ? saved : true;
  }

  /// Active ou coupe la dictée par Espace maintenu.
  Future<void> setHoldSpaceEnabled(bool enabled) =>
      _prefs.setBool(_holdSpaceKey, enabled);

  /// Durée de maintien d'Espace avant que la dictée démarre ; 400 ms par défaut.
  Duration get holdDelay {
    final saved = _prefs.get(_holdDelayKey);
    if (saved is! int || saved < _minHoldMs || saved > _maxHoldMs) {
      return _defaultHoldDelay;
    }
    return Duration(milliseconds: saved);
  }

  /// Enregistre la durée de maintien d'Espace.
  Future<void> setHoldDelay(Duration delay) =>
      _prefs.setInt(_holdDelayKey, delay.inMilliseconds);

  /// Temps pendant lequel le modèle reste en mémoire après la dernière dictée ; 3 minutes par défaut, 0 : pas de garde.
  Duration get keepWarm {
    final saved = _prefs.get(_keepWarmKey);
    if (saved is! int || saved < 0 || saved > _maxKeepWarmMinutes) {
      return _defaultKeepWarm;
    }
    return Duration(minutes: saved);
  }

  /// Enregistre la durée de garde du modèle.
  Future<void> setKeepWarm(Duration keep) =>
      _prefs.setInt(_keepWarmKey, keep.inMinutes);
}
