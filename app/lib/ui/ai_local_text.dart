//! Textes du modèle local : taille lisible, nom du palier, ce que le palier sait faire.

import '../engine/ai_local_models.dart';
import '../i18n/i18n.dart';

/// Taille en Go (ou Mo sous le Go), avec la virgule décimale.
String localSize(int bytes) => bytes >= 1000000000
    ? '${(bytes / 1000000000).toStringAsFixed(1).replaceAll('.', ',')} Go'
    : '${(bytes / 1000000).round()} Mo';

/// Nom du palier.
String localTierName(String tier) => switch (tier) {
  'mini' => t('Mini'),
  'light' => t('Léger'),
  _ => t('Avancé'),
};

/// Ce que le palier sait faire, en une phrase.
String localTierAbility(LocalModelStatus model) => model.supportsTools
    ? t('Assistant complet : peut chercher et lire tes mails pour t’aider.')
    : model.tier == 'mini'
    ? t('Reformule, résume et propose une réponse à un mail court. Très léger.')
    : t('Résume, reformule et répond avec plus de justesse.');
