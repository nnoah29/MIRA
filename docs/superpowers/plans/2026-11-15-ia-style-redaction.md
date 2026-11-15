# Style de rédaction (lot 2) — plan d'implémentation

> Exécution native, tâche par tâche, test d'abord. Spec : `docs/superpowers/specs/2026-11-15-ia-style-redaction-design.md` (S1–S8).

**Contraintes** : mêmes normes que le lot 1 (fonction ≤ 30 lignes, fichier ≤ 6 fonctions, aucun commentaire dans un corps, `unwrap` interdit hors tests, tests sur `127.0.0.1`, `CARGO_BUILD_JOBS=2` si la mémoire est serrée), un commit par tâche sans trailer, `./ci.sh` seulement sur accord. Les types d'analyse sont **purs** (aucune E/S).

- **T1 Analyse pure** : `mira-core/src/ai/style/` (`clean.rs`, `register.rs`, `greetings.rs`, `measures.rs`, `analyze.rs`, `render.rs`) et types `StyleProfile`, `StyleSettings`, `StyleRegister` dans `mira-vocabulary`. Tests : nettoyage des citations et signatures, registre à 70 %, ouvertures sans nom, médianes, émojis, moins de 5 mails, rendu ≤ 800 caractères.
- **T2 Réglages et collecte** : stockage `ai_style` ; collecte des 200 derniers mails `Sent` par compte (corps via la façade) ; `Engine::analyze_style`.
- **T3 API et contrat 1.12** : quatre requêtes, réponses, risque (`GetStyleStatus` non lecture), exemples du contrat, instantané régénéré, versions 1.12.
- **T4 Injection dans le chat** : paragraphe ajouté au message système si activé et profil exploitable ; absent sinon.
- **T5 App** : `AiStyleAdmin`, `AiStyleState`, section des réglages, textes fr/en, tests.
- **T6 Vérification** : analyse, tests ciblés, normes, architecture, docs (§6.13 de `ARCHITECTURE-IMPLEMENTEE.md`).
