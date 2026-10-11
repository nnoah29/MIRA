# Blocages

Format : `BLOQUÉ-<id>` · description · tentatives · contournement · statut.

## BLOQUÉ-01 — cargo-nextest ne s'installe pas
- **Description** : `cargo install --locked cargo-nextest` échoue sur le script de build de `aws-lc-sys 0.45` (dépendance C de sa pile TLS).
- **Tentatives** : 1 (installation `--locked`).
- **Contournement** : `./ci.sh` détecte l'absence de nextest et utilise `cargo test --workspace`. Aucun impact fonctionnel.
- **Statut** : contourné, non bloquant.

## BLOQUÉ-02 — Limite d'usage de l'API pendant la vague V2
- **Description** : à ~01:38, les trois sous-agents en cours ont reçu une erreur 429 (limite de session, réinitialisation 01:40).
- **Contournement** : reprise des agents depuis leur historique après la réinitialisation ; consigne renforcée de commiter chaque niveau dès qu'il est vert, pour ne pas perdre de travail.
- **Statut** : résolu (reprise à 01:42).
