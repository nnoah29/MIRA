# Propositions de modification de la référence

La référence est en lecture seule. Les incohérences et idées relevées pendant l'implémentation sont listées ici.

## PR-01 — Un crate par provider
L'annexe E présente « un crate par provider » comme un écart avec l'architecture, qui « retient un bloc providers regroupant les modules ». L'implémentation garde un crate par provider (décision de départ 34) : c'est le seul moyen de vérifier **mécaniquement** qu'un provider ne dépend ni du cœur ni d'un autre provider (invariant 2). Proposition : préciser en 5.3 que le bloc providers est composé d'un crate par provider.
