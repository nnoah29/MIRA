# Architecture implémentée — correspondance décisions → code

État au 2026-10-11 ~08:30, branche `feat/E-01-engine-overnight`. Les chemins sont relatifs à `engine/crates/`. « Non implémenté » signifie que la décision n'a pas encore de code ; la référence reste valable.

| ID | Décision | Où dans le code | Tests / garde-fou |
|---|---|---|---|
| D-01 | Rust | tout le workspace (toolchain 1.99.0, édition 2024) | — |
| D-02 | Pas de plugins dynamiques | liaison statique des crates ; `deny.toml` interdit `libloading` | `check-architecture` A7, `check-forbidden-patterns` |
| D-03 | Trois blocs cœur / providers / assemblage | `mira-core`, `mira-provider-*`, `mira-assembly` | `check-architecture` A2–A4 |
| D-04 | CLI d'abord | `mira-cli` (`run(args, io) -> i32`) | tests `mira-cli/tests/` |
| D-05 | Provider = profil + particularités | `mira-vocabulary/src/provider/` (`Provider`, `ProviderDescriptor`, `ProviderQuirks`) ; `mira-provider-*/src/{descriptor,provider,roles,translate}.rs` | tests par tables de chaque provider |
| D-06 | Le cœur ne nomme aucun provider | `mira-core` (faux providers des tests : `fake_labels`, `fake_folders`) | `check-architecture` A6 (cœur, tests compris) |
| D-07 | Providers purs | structures unitaires, dépendance unique `mira-vocabulary` | A2 |
| D-08 | Capacités = annonce serveur − exceptions provider | `mira-core/src/capabilities/` (`parse_capabilities`, `EffectiveCapabilities::compute`, `choose_*`) | `capabilities_*.rs` |
| D-09, D-10 | Un profil, données et secrets par profil | `mira-core/src/profile/` (`ProfilePaths`, `ProfileEnvironment`) ; `SecretRef::storage_key` = `mira/<profil>/<compte>/<type>` | `profile_*.rs`, `domain.rs` (vocabulaire) |
| D-11 | Réglages application / profil | non implémenté | — |
| D-12 | Message unique, boîtes = vues | tables `messages` + `appearances` (`storage/migrations/v001_initial.sql`) ; `storage/repositories/{messages,appearances}/` ; `reading/` | `storage_messages.rs`, `storage_appearances.rs`, `reading_*.rs` |
| D-13 | Identifiant interne + correspondance | table `server_ids` ; `storage/repositories/server_ids/` (`map_server_id`, `find_by_server_id`, `find_by_fingerprint` qui rend tous les candidats) | `invariant_7_*`, `storage_server_ids.rs` |
| D-14 | Drapeaux au niveau du message, « supprimé » sur l'apparition | colonnes `messages.seen/flagged/…`, `appearances.deleted` | `invariant_12_*` |
| D-15 | Fils calculés par MIRA | `mira-core/src/threads/` (clé de fil, rattachement par `In-Reply-To`, indice serveur) ; table `threads` dérivée | `threads_*.rs` |
| D-16 | Source brute compressée, analyse en cache | `storage/repositories/{raw_sources,parsed_cache}/` (zstd) ; `mime/` (analyse tolérante) | `invariant_14_*`, `mime_*.rs` (dont proptest) |
| D-17 | Second libellé sur compte à dossiers refusé | code d'erreur `incompatible_state.multiple_labels_unsupported` ; **logique non implémentée** (file d'actions J7b inachevée) | — |
| D-18 | Trois familles de données | schéma v1 en trois sections ; `discard_account_mirror` | `invariant_16_*` |
| D-19 | Entités | `mira-vocabulary/src/domain/` | tests du vocabulaire |
| D-20, D-21, D-22 | Descente par étapes, fenêtre de 90 jours, stratégie par capacités | réglages (`SyncSettings`, 90 j) et briques (`imap/`, `capabilities/`, dépôts) ; **orchestrateur de synchronisation non implémenté** (J5b) | — |
| D-23 | Le serveur gagne, rejeu par-dessus | non implémenté (J5b/J7) | — |
| D-24 | Optimisation « Tous les messages » reportée | — | — |
| D-25 | Écritures optimistes, file d'actions | dépôt `storage/repositories/actions/` (idempotent, ordonné) ; **soumission/rejeu non implémentés** | `invariant_17_*`, `invariant_18_*`, `invariant_19_*` |
| D-26 | Contrat d'API | `mira-core/src/api/` (`Engine::open/execute/subscribe/shutdown`, `Request` par domaine, un handler par opération) — 9 opérations sur ~65 | `api_engine.rs` |
| D-27 | Verrou de profil | `profile/lock.rs` (`File::try_lock`), pris par `Engine::open` | `profile_lock.rs`, `second_engine_on_same_profile_is_refused` |
| D-28 | Découpage v1 | décision de départ 3 ; voir `RAPPORT-FINAL.md` pour ce qui manque | — |
| D-29 | Export du profil | non implémenté | — |
| D-30 | Façade bibliothèque + CLI | `mira_core::api::Engine` (async) ; le CLI n'utilise encore que le registre | — |
| D-31 | Pile du brief | tokio, mail-parser, rusqlite+FTS5, keyring (SecretService via zbus), lettre, reqwest (rustls/ring) ; sqlite-vec reporté | — |
| D-32 | Taille cible | mesure : liste paginée sur 20 000 messages (test de performance de J6) | `reading_*` |
| D-33 | Vocabulaire distinct | crate `mira-vocabulary` (serde + thiserror) | A1 |
| D-34 | Sous-domaines du cœur, bibliothèques encapsulées | modules de `mira-core/src/` | A8 (chaque bibliothèque externe dans son module) |

Modules transverses ajoutés : `events/` (bus numéroté persistant + abonnements reprenables, invariant 32), `operations/` (tâches longues annulables, 12.2.4), `html/` (assainissement `ammonia`, ressources distantes bloquées et listées, 11.6/15.4), `auth/` (OAuth2 PKCE avec écouteur en boucle locale, échange et rafraîchissement des jetons, coffre d'identifiants), `smtp/` (composition MIME, envoi `lettre`, erreurs classées, issue incertaine après DATA).

Décisions prises pendant l'implémentation : `docs/DECISIONS-AGENT.md` (AG-01 à AG-13).
