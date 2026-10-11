# Environnement — relevé de la Phase 0

Relevé du 2026-10-11 vers 00:25 (+01:00).

## Machine

| Élément | Valeur | Conséquence |
|---|---|---|
| CPU | 8 cœurs | `-j 8` pour un build seul |
| RAM | 14 Gio, ~3–4 Gio disponibles (10 Gio déjà utilisés, swap 3,2 Gio utilisé) | **Facteur limitant** : pas plus de 2–3 compilations simultanées |
| Disque `/home` | 81 Gio libres (82 % utilisé) | ~1,1 Gio par répertoire `target/` avec les dépendances lourdes |
| OS | Arch Linux, noyau 7.2.9 | — |

## Chaîne Rust

| Outil | Version |
|---|---|
| Toolchain par défaut | **nightly** 1.94.0 (2025-12-10) — ancienne, à ne pas utiliser |
| Stable installée | 1.99.0 (2026-09-28) |
| Toolchain épinglée testée | `1.99.0` (installée automatiquement par `rust-toolchain.toml`), édition **2024**, resolver 3 |
| cargo | 1.99.0 |
| clippy / rustfmt | présents (composants de 1.99.0) |
| cargo-nextest, cargo-deny, cargo-audit, cargo-llvm-cov, cargo-tarpaulin | **absents** |
| cargo-binstall, sccache, mold | absents |

## Dépendances système

| Élément | Résultat |
|---|---|
| pkg-config | 3.0.7 |
| libdbus | 1.16.2 (inutile : `keyring` 4.x utilise zbus, pur Rust) |
| sqlite3 système | 3.53.4 (inutile : `rusqlite` en `bundled`) |
| SecretService | `org.freedesktop.secrets` fourni par gnome-keyring-daemon, collection par défaut **déverrouillée** (`Locked=false`) ; KWallet 6 activable |

## Accès réseau et compilation d'essai

Workspace d'essai (3 crates) dans le scratchpad, toolchain 1.99.0 :

- crates.io joignable (`cargo add` OK).
- Squelette vide : build en 0,3 s, tests OK.
- Avec `tokio`(full), `rusqlite 0.40`(bundled), `zstd 0.14`, `mail-parser 0.11`, `mail-builder 1.0`, `ammonia 4.2`, `keyring 4.2`, `async-imap 0.12`(runtime-tokio), `lettre 0.11`(rustls, tokio1), `rustls 0.23`, `tokio-rustls 0.26`, `serde`, `thiserror 2`, `tracing`, `uuid` :
  **build à froid 1 min 32 s** (5 min 38 s CPU), `target/` = 1,1 Gio.
- Test de fumée vert : **FTS5 disponible**, aller-retour zstd, analyse MIME, `ammonia` retire `<script>`.
- `cargo clippy`, `cargo fmt --check` : fonctionnent.
- `git commit` sur un fichier jetable (dépôt jetable) : OK, sans invite.

## Dépôt

- Branche courante `dev` (main et dev poussées sur `origin`). Workflow : `docs/GIT_WORKFLOW.md` (main / dev / feat/E-XX-nom, `make check`).
- `docs/MIRA-architecture-reference.md` **non suivi** par git.
- Dossier vide `engine_/` à la racine. Pas de dossier `mira/` (ancien brouillon) présent.

## Points de vigilance pour la nuit

1. **Hooks de plugins** : le hook `GateGuard` (plugin `ecc`) bloque la première commande Bash et la première écriture de chaque fichier (« Fact-Forcing Gate »). Il bloquera aussi chaque sous-agent, des centaines de fois sur une nuit. Un hook de détection de boucle signale à tort les lectures par tranches. Ce ne sont pas des refus de permission, mais ils coûtent beaucoup de temps et de tokens.
2. **RAM** : avec plusieurs worktrees qui compilent en parallèle (chacun ~1 Gio de `target/` et une forte charge mémoire au build à froid), le risque de swap est réel.
3. Les outils `cargo-nextest`, `cargo-deny`, etc. sont absents : il faut une autorisation pour les installer (`cargo install` compile, ~2–5 min chacun).
