//! Noms affichés des types de contenu qu'un service d'IA peut recevoir.

import '../engine/ai_privacy_models.dart';
import '../i18n/i18n.dart';

/// Nom court d'un type de contenu, pour une case à cocher.
String aiKindName(String kind) => switch (kind) {
  aiKindMail => t('Mail ouvert'),
  _ => kind,
};

/// Type de contenu au complément d'une phrase (« recevoir : le mail ouvert »).
String aiKindObject(String kind) => switch (kind) {
  aiKindMail => t('le mail ouvert'),
  _ => kind,
};
