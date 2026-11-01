//! Titre et identifiant d'une conversation enregistrée avec l'assistant IA.

import 'dart:math';

import '../i18n/i18n.dart';

/// Nombre de caractères du titre que le moteur accepte.
const conversationTitleMax = 80;

/// Titre tiré de la première question : espaces aplatis, coupé à 80 caractères comptés comme le moteur.
String conversationTitle(String firstQuestion) {
  final flat = firstQuestion.replaceAll(RegExp(r'\s+'), ' ').trim();
  if (flat.isEmpty) return t('Nouvelle conversation');
  return String.fromCharCodes(flat.runes.take(conversationTitleMax));
}

/// Identifiant aléatoire au format UUID v4, tiré du générateur sûr du système.
String newConversationId() {
  final random = Random.secure();
  final bytes = List<int>.generate(16, (_) => random.nextInt(256));
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  final hex = [for (final b in bytes) b.toRadixString(16).padLeft(2, '0')];
  final text = hex.join();
  return '${text.substring(0, 8)}-${text.substring(8, 12)}-'
      '${text.substring(12, 16)}-${text.substring(16, 20)}-${text.substring(20)}';
}
