# Décisions prises par l'agent

Format : **AG-xx — titre** · contexte · options · choix · raison · retour arrière.

## AG-01 — Garde-fous écrits en Python 3 (bibliothèque standard)
- **Contexte** : `tools/check-norms` et `tools/check-architecture` doivent analyser le code Rust et `cargo metadata`.
- **Options** : crate Rust (avec `syn`) dans le workspace, ou script Python sans dépendance.
- **Choix** : Python 3, bibliothèque standard seulement.
- **Raison** : pas de nouvelle dépendance Rust ; l'outil n'est pas du code du moteur et n'est donc pas soumis à ses normes ; `python3` est présent ; un lexer maison suffit (chaînes, caractères, commentaires, accolades).
- **Retour arrière** : réécrire l'outil en crate Rust ; l'interface (code de sortie, format des messages) reste la même.
