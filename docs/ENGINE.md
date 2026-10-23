# MIRA — moteur (engine) Rust

MIRA (*Mail Interface & Retrieval Application*) est un client mail Linux natif, multi-comptes et **offline-first** : le moteur tient un miroir local (SQLite + index plein texte FTS5) de chaque boîte IMAP, toutes les lectures se font sur ce miroir, et toutes les écritures sont d'abord enregistrées dans une file d'actions locale avant d'être rejouées vers le serveur (IMAP pour l'état des messages, SMTP pour l'envoi).

Le **moteur** est une bibliothèque asynchrone (la façade `Engine`, contrat d'API versionné). Son seul client est l'application Flutter (`app/`), qui l'appelle par le pont FFI (`bridge/`). Le client en ligne de commande `mira` et le client plein écran (TUI) ont été retirés (AG-63).

- Architecture de référence (source de vérité, lecture seule) : `docs/MIRA-architecture-reference.md`.
- Ce qui a réellement été implémenté : `docs/ARCHITECTURE-IMPLEMENTEE.md`.
- Écarts proposés à la référence : `docs/PROPOSITIONS-REFERENCE.md`.
- Lire le code en venant du C++ : `docs/GUIDE-RUST-POUR-CPP.md`.

## Prérequis

- `rustup` : la toolchain **Rust 1.99.0** (édition 2024) est installée automatiquement grâce à `engine/rust-toolchain.toml` (avec `rustfmt` et `clippy`).
- `python3` (les garde-fous `tools/check-norms` et `tools/check-architecture` sont des scripts Python).
- Aucune bibliothèque système : SQLite est compilé avec le moteur et le trousseau passe par D-Bus en Rust pur (`keyring`). Avec de vrais comptes, il faut en revanche un service *Secret Service* (par exemple GNOME Keyring ou KWallet) ; les tests, eux, n'y touchent jamais.

## Construire, tester, vérifier

```bash
cd engine
cargo build --workspace            # construit le moteur
cargo test --workspace             # tous les tests
```

Depuis la racine, `./ci.sh` (ou `make check`, point d'entrée prévu par `docs/GIT_WORKFLOW.md`) exécute toute l'intégration continue locale ; tout doit être vert avant chaque commit. Il contrôle, dans l'ordre :

1. les auto-tests des garde-fous (`tools/test-check-norms`, `tools/test-check-architecture`) ;
2. `tools/check-norms` : normes de codage (≤ 30 lignes par fonction, ≤ 6 fonctions par fichier, aucun commentaire dans un corps de fonction, une ligne `///` avant chaque fonction, pas de fichier `utils`/`helpers`/`common`/`misc`) ;
3. `tools/check-architecture` : frontière cœur / providers (le cœur ne dépend d'aucun provider et ne nomme aucun fournisseur, les providers ne dépendent que de `mira-vocabulary`, chaque bibliothèque externe reste dans son module d'encapsulation, pas de chargement dynamique) ;
4. `tools/check-forbidden-patterns` : pas d'`unwrap`/`expect`/`panic!`/`todo!`/`dbg!`/`unsafe` hors tests, pas de `TODO` non référencé, pas de marqueur de conflit git, pas de `libloading` ;
5. `cargo fmt --all --check` ;
6. `cargo clippy --workspace --all-targets --all-features -- -D warnings` ;
7. les tests du workspace ;
8. `cargo deny check bans licenses sources` si `cargo-deny` est installé.

Deux précisions sur cet environnement :

- `ci.sh` utilise `cargo nextest` s'il est présent et **sinon `cargo test`** ; c'est `cargo test` qui sert ici, car `cargo-nextest` n'a pas pu être installé. Le journal des tests est écrit dans `target/rapports/tests.log`.
- `cargo deny` est lancé **sans les avis de sécurité** (`advisories`) : les avis de sécurité sont exclus car ils demandent un accès réseau (`ci.sh` le dit : « avis de sécurité hors ligne exclus ») ; seuls les interdits (`libloading`, `openssl`…), les licences et les sources de dépendances sont vérifiés (voir `engine/deny.toml`).

Tous les tests réseau tournent contre les faux serveurs du crate `mira-testkit` (IMAP, SMTP, OAuth2) sur `127.0.0.1`. **Aucun compte réel n'est utilisé dans ce dépôt**, et aucun test automatique ne touche au vrai trousseau.

## Organisation du dépôt

```
CLAUDE.md  Makefile  ci.sh
tools/      garde-fous : check-norms, check-architecture, check-forbidden-patterns (+ leurs auto-tests)
docs/       référence, décisions (DECISIONS-AGENT.md), specs/ par jalon
engine/     workspace Cargo (toolchain 1.99.0, édition 2024)
  crates/mira-vocabulary   types du domaine, erreurs et événements du contrat, interface Provider (serde + thiserror seulement)
  crates/mira-core         le moteur : stockage SQLite/FTS5, profil et verrou, client IMAP, analyse MIME, SMTP,
                           OAuth2 PKCE, HTML sûr, synchro, file d'actions et rejeu, événements, façade api::Engine
  crates/mira-provider-*   gmail, outlook, yahoo, generic : descriptions pures (aucune entrée-sortie)
  crates/mira-assembly     composition root : enregistre les providers, fabrique la configuration de production
  crates/mira-testkit      faux serveurs IMAP / SMTP / OAuth pour les tests
bridge/     pont FFI (flutter_rust_bridge) entre le moteur et l'application
app/        application Flutter (Linux, Android)
packaging/  paquets deb, rpm, AppImage, archive AUR, APK
```

Règle de frontière : le cœur ne nomme aucun fournisseur, les providers sont purs, seul `mira-assembly` connaît tout le monde (vérifié par `tools/check-architecture`).

## Comptes OAuth2 (Gmail, Outlook)

> **Identifiants d'application OAuth** : un moteur construit hors des paquets n'en contient aucun. Ils se déclarent dans un fichier TOML, `$MIRA_CONFIG` ou à défaut `$XDG_CONFIG_HOME/mira/config.toml` (`~/.config/mira/config.toml`) :
>
> ```toml
> [oauth.gmail]
> client_id = "xxxx.apps.googleusercontent.com"
> client_secret = "xxxx"   # facultatif
>
> [oauth.outlook]
> client_id = "xxxx-xxxx-xxxx"
> ```
>
> `client_secret` est facultatif : à renseigner si le service l'exige à l'échange des jetons (c'est en général le cas des clients « application de bureau » de Google, d'après nos connaissances, non vérifiées en ligne ; un client public Microsoft n'en a pas). Le fichier contient alors un secret : `chmod 600` dessus. Les paquets construits par `packaging/` intègrent ces identifiants, chiffrés dans le moteur (`packaging/oauth-credentials.sh`). Les comptes à mot de passe ou mot de passe d'application ne sont pas concernés.

## Limites connues

- Aucun test contre un vrai serveur ni un vrai trousseau (règle de la session de développement) ; le comportement face aux particularités réelles des fournisseurs reste à valider.
- L'effacement d'un compte est durci (`secure_delete`, checkpoint du journal) mais ne purge pas rétroactivement les traces d'une base créée avant ce correctif (AG-33).

## Documentation

| Fichier | Contenu |
|---|---|
| `docs/MIRA-architecture-reference.md` | Architecture de référence (lecture seule) |
| `docs/ARCHITECTURE-IMPLEMENTEE.md` | Ce qui est réellement implémenté, module par module |
| `docs/GUIDE-RUST-POUR-CPP.md` | Lire le code Rust quand on vient du C++ |
| `docs/DECISIONS-AGENT.md` | Décisions d'implémentation (contexte / choix / raison / retour arrière) |
| `docs/PROPOSITIONS-REFERENCE.md` | Corrections proposées à la référence |
| `docs/specs/` | Spécifications par jalon (dont `J8-J9-facade-cli.md`, CLI retiré depuis) |
| `docs/GIT_WORKFLOW.md` | Conventions Git (lecture seule) |
