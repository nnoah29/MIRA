# Dernier résultat global de ./ci.sh

Relevé : 2026-10-11T02:08:05+01:00 — commit 2d12671.

| Contrôle | Résultat |
|---|---|
| Auto-tests des garde-fous | OK |
| check-norms | 492 fichiers, 0 violation |
| check-architecture (A0–A9) | 9 crates, 0 violation |
| Motifs interdits (unwrap/expect/panic/unsafe/TODO) | OK |
| cargo fmt --check | OK |
| cargo clippy -D warnings | OK |
| Tests | **317 réussis, 0 échec, 1 ignoré** (test manuel du vrai trousseau, autorisé par la décision de départ 18) |
| cargo deny (bans, licences, sources) | OK |
| Couverture | non mesurée encore (cargo-llvm-cov installé, composant llvm-tools à ajouter) |
