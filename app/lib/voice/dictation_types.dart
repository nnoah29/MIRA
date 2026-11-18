//! Types et interfaces de la dictée vocale : langues, événements, problèmes, états.
//! Rien ici ne dépend d'une bibliothèque de reconnaissance ni du micro.

import 'dart:typed_data';

/// Langues dictables ; chacune a son propre modèle de reconnaissance.
enum DictationLanguage {
  fr('fr', 'Français'), // i18n:ignore
  en('en', 'English');

  const DictationLanguage(this.code, this.label);
  final String code;
  final String label;

  /// Langue de dictée pour un code de locale ; tout ce qui n'est pas anglais retombe sur le français.
  static DictationLanguage fromLocale(String localeCode) =>
      localeCode.toLowerCase().startsWith('en')
      ? DictationLanguage.en
      : DictationLanguage.fr;
}

/// Texte produit par le reconnaisseur : provisoire ou définitif.
sealed class DictationEvent {
  const DictationEvent(this.text);
  final String text;
}

/// Texte encore susceptible de changer pendant que l'utilisateur parle.
final class DictationPartial extends DictationEvent {
  const DictationPartial(super.text);
}

/// Fin de phrase : le texte ne changera plus.
final class DictationFinal extends DictationEvent {
  const DictationFinal(super.text);
}

/// Raison pour laquelle une dictée n'a pas démarré ou s'est arrêtée.
enum DictationProblem {
  modelMissing,
  microphoneDenied,
  microphoneUnavailable,
  recognizerFailed,
}

/// Erreur portant la raison d'un échec de dictée.
class DictationException implements Exception {
  const DictationException(this.problem);
  final DictationProblem problem;

  @override
  String toString() => 'DictationException($problem)';
}

/// Étape de la dictée.
enum DictationState { idle, preparing, listening, stopping }

/// Façon de dicter : en continu jusqu'à un nouvel appui, ou phrase par phrase tant qu'on maintient une touche.
enum DictationMode { continuous, pushToTalk }

/// Source audio : PCM mono 16 kHz, échantillons dans [-1, 1].
abstract interface class AudioSource {
  /// Demande l'autorisation d'utiliser le micro ; faux si elle est refusée.
  Future<bool> requestPermission();

  /// Ouvre le micro et rend le flux d'échantillons.
  Future<Stream<Float32List>> start();

  /// Ferme le micro.
  Future<void> stop();
}

/// Reconnaissance vocale en flux.
abstract interface class SpeechRecognizer {
  /// Textes provisoires et définitifs, dans l'ordre.
  Stream<DictationEvent> get events;

  /// Prépare la reconnaissance dans la langue donnée ; lève [DictationException].
  Future<void> start(DictationLanguage language);

  /// Transmet des échantillons audio.
  void feed(Float32List samples);

  /// Vide la fin de la parole, émet le dernier texte définitif et libère les ressources ;
  /// le futur se termine une fois ce texte livré aux abonnés de [events].
  Future<void> stop();
}

/// Dit si le modèle d'une langue est installé.
abstract interface class ModelAvailability {
  bool isInstalled(DictationLanguage language);
}
