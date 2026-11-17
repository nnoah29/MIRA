//! Fournisseurs qui partagent un onglet : l'abonnement d'un service et sa clé d'API sont le même service.

import '../engine/ai_models.dart';

/// Fournisseur → onglet qui l'accueille.
const Map<String, String> aiTabFamily = {'chatgpt': 'openai'};

/// Onglet où se trouve ce fournisseur.
String aiTabOf(String providerId) => aiTabFamily[providerId] ?? providerId;

/// Fournisseurs qui ont leur propre onglet : ceux qui n'en rejoignent pas un autre présent.
List<AiProviderInfo> aiTabProviders(List<AiProviderInfo> all) => [
  for (final p in all)
    if (!all.any((o) => o.id == aiTabFamily[p.id])) p,
];

/// Contenu d'un onglet : son fournisseur, puis ceux qui l'ont rejoint.
List<AiProviderInfo> aiFamilyOf(List<AiProviderInfo> all, AiProviderInfo tab) =>
    [
      tab,
      for (final p in all)
        if (aiTabFamily[p.id] == tab.id) p,
    ];
