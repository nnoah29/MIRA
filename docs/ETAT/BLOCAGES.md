# Blocages

Format : `BLOQUÉ-<id>` · description · tentatives · contournement · statut.

## BLOQUÉ-01 — cargo-nextest ne s'installe pas
- **Description** : `cargo install --locked cargo-nextest` échoue sur le script de build de `aws-lc-sys 0.45` (dépendance C de sa pile TLS).
- **Tentatives** : 1 (installation `--locked`).
- **Contournement** : `./ci.sh` détecte l'absence de nextest et utilise `cargo test --workspace`. Aucun impact fonctionnel.
- **Statut** : contourné, non bloquant.
