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

## BLOQUÉ-03 — Redémarrage de session et seconde limite d'usage (02:10 → 06:40)
- **Description** : la session principale s'est arrêtée pendant la fusion de J7a (~02:10), puis, après reprise à 02:53, une seconde limite d'usage (HTTP 429) a stoppé tous les agents jusqu'à 06:40. Environ **3 h 50 perdues** sur le temps de travail.
- **Contournement** : fusion J7a terminée à la reprise (conflits Cargo.toml / lib.rs résolus), agents J5a et J6 relancés depuis leur historique, périmètre de fin de nuit **réduit** : priorité à J5a, J6, J7 local, puis façade comptes et CLI ; la synchronisation complète (J5b) et S1–S14 de bout en bout risquent de ne pas être terminés avant 09:08.
- **Statut** : contourné, impact sur le périmètre consigné dans PLAN.md et le rapport final.
