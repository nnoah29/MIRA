# MIRA — moteur (engine) Rust

MIRA (*Mail Interface & Retrieval Application*) est un client mail Linux natif, multi-comptes et offline-first. Ce dépôt contient le **moteur** en Rust, utilisable sans interface (CLI d'abord, Flutter plus tard). L'architecture de référence est `docs/MIRA-architecture-reference.md` ; ce qui a été réellement implémenté est décrit dans `docs/ARCHITECTURE-IMPLEMENTEE.md`, et l'état de la session de développement dans `docs/RAPPORT-FINAL.md`.

## Construire et tester

Prérequis : rustup (la toolchain `1.99.0` est installée automatiquement grâce à `engine/rust-toolchain.toml`), `python3`. Aucune bibliothèque système n'est nécessaire (SQLite est compilé avec le moteur, le trousseau passe par D-Bus en pur Rust).

```bash
./ci.sh            # tout : garde-fous, normes, architecture, fmt, clippy -D warnings, tests, cargo-deny
make check         # identique (point d'entrée de docs/GIT_WORKFLOW.md)
cd engine && cargo test --workspace          # tests seuls
cd engine && cargo build -p mira-cli         # binaire engine/target/debug/mira
```

`./ci.sh` exécute aussi :
- `tools/check-norms` : normes de codage (≤ 30 lignes par fonction, ≤ 6 fonctions par fichier, aucun commentaire dans un corps, une ligne `///` par fonction, pas de fichier fourre-tout) ;
- `tools/check-architecture` : frontière cœur/providers (le cœur ne dépend d'aucun provider et ne nomme aucun fournisseur, les providers ne dépendent que du vocabulaire, chaque bibliothèque externe reste dans son module, pas de chargement dynamique) ;
- `tools/check-forbidden-patterns` : pas d'`unwrap`/`expect`/`panic!`/`unsafe` hors tests, pas de `TODO` non référencé, pas de marqueur de conflit.

Tous les tests réseau tournent contre les faux serveurs du crate `mira-testkit` (IMAP, SMTP, OAuth2) sur `127.0.0.1`. Aucun compte réel n'est utilisé, et aucun test automatique ne touche au vrai trousseau.

## Organisation

```
engine/crates/
  mira-vocabulary         types du domaine, erreurs et événements du contrat, interface Provider (serde + thiserror seulement)
  mira-core               le moteur : stockage SQLite/FTS5, ports (horloge, ids, trousseau), profil et verrou,
                          registre et capacités, client IMAP, analyse MIME, SMTP, OAuth2 PKCE, HTML sûr,
                          lectures et fils, bus d'événements, opérations longues, façade `api::Engine`
  mira-provider-*         gmail, outlook, yahoo, generic : descriptions pures (aucune entrée-sortie)
  mira-assembly           composition root : enregistre les providers
  mira-cli                binaire `mira`
  mira-testkit            faux serveurs IMAP / SMTP / OAuth pour les tests
```

## Utiliser le CLI (état actuel)

```bash
cd engine
cargo run -q -p mira-cli -- provider list
cargo run -q -p mira-cli -- provider detect alice@gmail.com
cargo run -q -p mira-cli -- --json provider detect alice@exemple.fr     # enveloppe {contract_version, ok, data}
cargo run -q -p mira-cli -- --json provider detect "pas une adresse"    # code de sortie 10, enveloppe d'erreur
```

Codes de sortie : 0 succès, 10 entrée invalide, 11 introuvable, 12 état incompatible, 13 réseau, 14 TLS, 15 authentification, 16 serveur, 17 stockage local, 18 secrets, 19 annulé, 84 erreur interne.

Les autres opérations du contrat ne sont pas encore exposées par le CLI : voir `docs/RAPPORT-FINAL.md`.
