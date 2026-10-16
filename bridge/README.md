# mira-bridge — pont FFI Flutter ↔ moteur

Crate `cdylib` (flutter_rust_bridge 2.11.1) au-dessus de `mira-assembly`. Hors du workspace `engine/`
pour que `unsafe_code = "forbid"` reste vrai côté moteur (voir `docs/DECISIONS-AGENT.md`, AG-45).

| Fonction Dart | Rôle |
| --- | --- |
| `contractVersion()` | version du contrat compilée (`1.0`) |
| `openEngine(profile)` | ouvre le profil (trousseau et dossier de données du système, `MIRA_HOME`) |
| `execute(requestJson)` | une requête du contrat → enveloppe `{contract_version, ok, data \| error}` |
| `watchEvents(after)` | flux des événements du moteur (JSON) |
| `closeEngine()` | ferme le moteur et libère le verrou du profil |

```bash
cargo build --release                      # → target/release/libmira_bridge.so
cargo test                                  # JSON → moteur réel → faux serveur IMAP 127.0.0.1
cd ../app && RUSTC_BOOTSTRAP=1 flutter_rust_bridge_codegen generate   # après un changement de src/api
```

Le codegen doit être en version **2.11.1** (`cargo install flutter_rust_bridge_codegen --version 2.11.1 --locked`).
`src/frb_generated.rs` est généré : ne pas le modifier.
