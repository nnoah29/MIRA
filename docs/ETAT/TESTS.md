# Dernier résultat global de ./ci.sh

Relevé : 2026-10-14T13:08:09+00:00 — commit f013a56 (branche feat/E-01-engine-overnight).

| Contrôle | Résultat |
|---|---|
| Auto-tests des garde-fous | OK |
| check-norms | 0 violation |
| check-architecture (A0–A9) | 9 crates, 0 violation |
| Motifs interdits (unwrap/expect/panic/unsafe/TODO/marqueurs de conflit) | OK |
| cargo fmt --check | OK |
| cargo clippy -D warnings | OK |
| Tests | **1080 réussis, 0 échec, 1 ignoré** (test manuel du vrai trousseau, décision de départ 18) |
| cargo deny (bans, licences, sources) | OK (avis de sécurité exclus, AG-07) |
| Couverture | non mesurée |
| Invariants 1–33 / scénarios S1–S14 avec test nommé | tous (voir INVARIANTS.md) ; S1–S14 aussi via le CLI (`_via_cli`) |

## Performance

- Liste paginée : 3 à 5 ms par page sur 20 000 messages (`reading_performance`).
- **Mesure à 100 000 messages : non faite** (reste de J10). La cible D-32 (< 50 ms) n'est donc pas vérifiée à cette échelle.
