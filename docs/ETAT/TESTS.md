# Dernier résultat global de ./ci.sh

Relevé : 2026-10-11T08:25:24+01:00 — commit 8452e15.

| Contrôle | Résultat |
|---|---|
| Auto-tests des garde-fous | OK |
| check-norms | 688 fichiers, 0 violation |
| check-architecture (A0–A9) | 9 crates, 0 violation |
| Motifs interdits (unwrap/expect/panic/unsafe/TODO/marqueurs de conflit) | OK |
| cargo fmt --check | OK |
| cargo clippy -D warnings | OK |
| Tests | **478 réussis, 0 échec, 1 ignoré** (test manuel du vrai trousseau, décision de départ 18) |
| cargo deny (bans, licences, sources) | OK |
| Couverture | non mesurée (cargo-llvm-cov installé ; composant llvm-tools non ajouté) |
| Invariants / scénarios avec test nommé | 24 sur 47 (voir INVARIANTS.md) |
