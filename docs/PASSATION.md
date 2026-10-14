# Passation — moteur MIRA (état au 2026-10-14)

Document destiné à l'agent (ou au développeur) qui reprend le travail. Il est **autoporteur** : avec lui, `CLAUDE.md` et les fichiers qu'il cite, on peut reprendre sans autre contexte. En cas de contradiction, l'ordre d'autorité est : `docs/MIRA-architecture-reference.md` (référence, lecture seule) > `docs/ETAT/DECISIONS-DE-DEPART.md` (réponses de l'utilisateur) > `docs/DECISIONS-AGENT.md` (décisions d'implémentation AG-xx) > ce document.

---

## 1. Le projet en deux paragraphes

MIRA est un client mail Linux natif, multi-comptes, **offline-first**. Ce dépôt contient le **moteur en Rust** (« engine »), utilisable sans interface : d'abord par un CLI, plus tard par une interface Flutter. Le porteur du projet (l'utilisateur) est développeur C++ (projet Arcade), peu à l'aise en Rust : le code doit rester simple et lisible, et les explications vont dans la documentation, pas dans le code.

Principes structurants (référence ch. 2) : le cœur ne connaît aucun fournisseur (Gmail, Outlook…) et raisonne en **capacités** du serveur ; les providers sont de pures descriptions ; un message existe **une seule fois** et les boîtes en sont des vues (apparitions) ; le serveur gagne toujours, les actions locales sont rejouées par-dessus ; lire = local, écrire = intention mise en file ; aucun secret hors du trousseau ; tout est testable sans réseau (faux serveurs).

## 2. Où lire quoi

| Fichier | Contenu |
|---|---|
| `CLAUDE.md` | Mémoire de session : règles absolues, normes de codage, boucle de travail, commandes. **À relire en premier.** |
| `docs/MIRA-architecture-reference.md` | La référence (2 115 lignes) : décisions D-01…D-34, invariants 1–33, scénarios S1–S14, contrat d'API (ch. 12). **Ne jamais la modifier.** |
| `docs/ETAT/DECISIONS-DE-DEPART.md` | Réponses de l'utilisateur au questionnaire initial (périmètre, dépendances, CLI, git, règles). |
| `docs/DECISIONS-AGENT.md` | Décisions prises pendant l'implémentation (AG-01 → AG-16), avec raison et retour arrière. |
| `docs/specs/*.md` | Spécifications détaillées par jalon, écrites avant le code : `CORE-ARCHITECTURE.md` (carte des modules, ports, schéma SQL), `J1`…`J8-J9`. **Les jalons restants ont déjà leur spec.** |
| `docs/ETAT/PLAN.md` | Statut des jalons J0–J10 et critères d'acceptation. |
| `docs/ETAT/JOURNAL.md` | Chronologie complète (fin du fichier = notes de reprise). |
| `docs/ETAT/INVARIANTS.md` | Table invariants/scénarios → tests (régénérée par `tools/update-invariants-table`). |
| `docs/ETAT/BLOCAGES.md`, `TESTS.md`, `HORLOGE.md`, `ENVIRONNEMENT.md` | Blocages, dernier résultat de CI, horloge de la session, environnement machine. |
| `docs/ARCHITECTURE-IMPLEMENTEE.md` | Correspondance D-xx → code (**à mettre à jour** : elle date d'avant J7b/J8a). |
| `docs/PROPOSITIONS-REFERENCE.md` | Propositions de corrections de la référence (PR-01 → PR-06). |
| `docs/RAPPORT-FINAL.md` | Rapport de la première nuit (**obsolète** sur les chiffres : voir §4 ici). |
| `docs/GIT_WORKFLOW.md` | Workflow git de l'utilisateur (main / dev / feat/E-XX). Ne pas modifier. |

## 3. Règles non négociables (résumé de `CLAUDE.md`)

1. Pas de navigateur ni de computer use ; réseau : crates.io via cargo, git local, WebSearch/WebFetch **pour se documenter seulement** (sources dans `docs/SOURCES.md`).
2. **Aucun compte mail réel** : tout est testé contre le testkit sur `127.0.0.1`.
3. **Pas de `git push`**, pas de fusion dans `dev`/`main`, pas de `sudo`, rien supprimé hors du dépôt.
4. Pas d'`unsafe` ; pas d'`unwrap`/`expect`/`panic!` hors tests ; **aucun secret** dans journaux, erreurs, événements, réponses, base.
5. Frontière : `mira-core` ne nomme aucun fournisseur (même dans ses tests : faux providers `fake_labels`, `fake_folders`, `fake_fallback`, `fake_oauth`) ; les providers ne dépendent que de `mira-vocabulary` ; seul `mira-assembly` connaît tout.
6. Normes (vérifiées mécaniquement) : fonction ≤ 30 lignes ; ≤ 6 fonctions avec corps par fichier (tests compris) ; **aucun commentaire dans un corps de fonction** ; exactement une ligne `///` en français avant chaque `fn` ; pas de `utils.rs`/`helpers.rs`/`common.rs`/`misc.rs` ; tests dans des fichiers séparés.
7. Jamais supprimer ni affaiblir un test pour passer au vert.
8. **Un commit par fonctionnalité**, dès que `./ci.sh` est vert ; fusions `--no-ff` sans squash. Trailers de commit :
   ```
   Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
   Claude-Session: https://claude.ai/code/session_01Mabg36RAZoqZyQR3eALweY
   ```
9. **Consigne de l'utilisateur (2026-10-11)** : plus de limite horaire ; continuer jusqu'à ce que J5–J10 soient FAITS, rigoureusement, sans raccourci, sauf arrêt demandé. **Ne poser aucune question** : trancher l'ambiguïté par l'option la plus simple et réversible, la consigner dans `DECISIONS-AGENT.md`.

## 4. État actuel du code

**Branche de travail** : `feat/E-01-engine-overnight` (créée depuis `dev`), dernier commit `517c6f1`, arbre propre, **`./ci.sh` vert : 606 tests réussis, 0 échec, 1 ignoré** (test manuel du vrai trousseau, autorisé). Tags locaux : `jalon/J0`, `J1`, `J2`, `J3`, `J4`, `J6`.

### Workspace `engine/` (toolchain épinglée 1.99.0, édition 2024)

| Crate | Rôle | État |
|---|---|---|
| `mira-vocabulary` | Identifiants opaques `Id<K>`, entités D-19, `MiraError` (catégorie + code stable + réessayable + action suggérée), catalogue de codes, `EventKind` (44 variantes), `EventFilter`, `ContractVersion`, trait `Provider` | Fait |
| `mira-core` | Le moteur (modules ci-dessous) | Partiel |
| `mira-provider-{gmail,outlook,yahoo,generic}` | Descriptions pures (données du §6.7, **non vérifiées en ligne**) | Fait |
| `mira-assembly` | Composition root : `build_registry()`, `production_config(profile)`, réexports | Fait |
| `mira-cli` | Binaire `mira` : seulement `provider list/detect` sur la branche de travail | Partiel |
| `mira-testkit` | Faux IMAP (CONDSTORE/QRESYNC, IDLE, MOVE, UIDPLUS, libellés, SEARCH HEADER, pannes, renumérotation), faux SMTP (pannes, coupure après DATA), faux OAuth2 PKCE | Fait |

### Modules de `mira-core/src/` (fusionnés)

| Module | Contenu | Jalon |
|---|---|---|
| `storage/` | SQLite par profil, migrations v001–v005, `Database::write/read` + `Tx`, dépôts par entité (`impl Tx`), FTS5 échappé, source brute zstd, pagination par curseur opaque, introspection | J2 |
| `ports/` | `Clock`/`ManualClock`, `IdGenerator`/`SequentialIds`/`UuidV7Ids`, `SecretStore`/`KeyringSecretStore`/`MemorySecretStore`, `SecretValue` masqué | J2 |
| `profile/` | Chemins (`MIRA_HOME` > XDG), verrou exclusif D-27 | J2 |
| `registry/`, `capabilities/`, `mailboxes/` | Registre + repli, capacités effectives et choix de stratégie, rôles de dossiers (SPECIAL-USE d'abord) | J3 |
| `imap/` | Client IMAP **maison** de haut niveau (connexion, liste, select/QRESYNC, fetch, flags, move/copy/expunge/append, libellés, recherche, IDLE) ; clair seulement vers la boucle locale (AG-03) | J5a |
| `mime/` | Analyse tolérante (`mail-parser`), proptest « ne panique jamais » | J5a |
| `reading/`, `threads/` | Listes par boîte et vues standard (unifiée, non lus, suivis), corps hors ligne (S6), recherche filtrée, fils D-15, perf 20 000 messages | J6 |
| `smtp/`, `auth/` | Composition MIME, envoi `lettre` avec erreurs classées, OAuth2 PKCE + écouteur en boucle locale, échange/rafraîchissement, `CredentialVault` | J7a |
| `actions/`, `drafts/` | File d'actions **locale** : `submit` (idempotent, effet local + inverse + cibles serveur), `cancel`, `retry`, `revert_local_effect`, `reapply_pending_local_effects` (rebase) ; brouillons (réponse, transfert, pièces jointes ≤ 25 Mo), `schedule_send`, `cancel_send` | J7b |
| `events/`, `operations/` | Bus d'événements numérotés persistés + abonnements reprenables ; tâches longues annulables | transverses |
| `html/` | Assainissement `ammonia`, ressources distantes bloquées et listées | J10 |
| `accounts/`, `api/` | Façade `Engine::open/execute/subscribe/shutdown`, `Request` par domaine (Engine, Provider, Account, Identity, Auth, Operation, Event), comptes (Add/Test/Update/Pause/Resume/Delete), identités, OAuth (Start/Complete/Cancel/Reauthenticate), `EngineConfig` avec `NetworkPolicy` et `OAuthClientConfig` | J8a |

### Couverture des invariants et scénarios

30 / 47 ont un test nommé. **Manquants** : invariants 6, 10, 20, 21, 23, 24, 25, 26, 30 ; scénarios S4, S5, S7, S8, S9, S10, S12, S13. Ils dépendent de J5b, J7c, J8 (reste), J9.

## 5. Travail en cours, non fusionné (dans des worktrees)

Les worktrees vivent sous `.claude/worktrees/` et partagent le dépôt (les branches sont visibles depuis la branche de travail).

| Jalon | Branche | Worktree | État |
|---|---|---|---|
| **J5b — synchronisation** | `feat/E-01-j5b-sync` | `agent-a1884904a70cc440a` | **6 commits** (contexte + rebase + identifiants, session + budget de connexions + liste des boîtes, première synchro par étapes + identité + préchargement, incrémental QRESYNC/CONDSTORE/comparaison, renumérotation, fenêtre élargie). **Non commité** : `sync/download/account.rs`, `sync/download/mod.rs` modifiés, dossier `sync/supervisor/` nouveau (superviseur en cours). Reste : superviseur (une tâche par compte, inv. 24), temps réel IDLE/relevé + backoff + états du compte (S5), tests des 4 jeux de capacités. Base : `aef564f` (avant J7b/J8a → fusionner la branche de travail dedans avant de continuer ; conflits attendus sur `lib.rs`, `migrations/mod.rs`, `repositories/mod.rs`). Branchement à faire : `sync::LocalIntentReplayer` doit appeler `actions::reapply_pending_local_effects`. |
| **J9a — CLI sur la façade** | `feat/E-01-j9a-cli` | `agent-ac3a8e43602d93a7d` | **0 commit, ~32 fichiers non commités** (commandes engine/account/identity/auth/operation/events, `cli_deps`, `engine_session`, `request_parsing`, rendu). État de compilation inconnu : lancer `./ci.sh` dans ce worktree, corriger, commiter. |
| **J7c — rejeu, envoi, reprise** | `feat/E-01-j7c-replay-send` | `agent-a70d10c1eb5a98f25` | **Rien** (arrêté au démarrage). À relancer avec la même consigne (voir §7). |

## 6. Ce qu'il reste à faire, dans l'ordre

1. **J5b** : terminer dans son worktree (ou relancer un agent en lui disant de reprendre la branche) ; spec `docs/specs/J5b-synchronisation.md` §7–§8. Fusionner.
2. **J7c** : rejeu des actions sur le serveur (traduction par particularités/capacités, vérification avant rejeu non idempotent), erreurs (passagère / auth / définitive + revert), **envoi** (`InProgress` persisté avant SMTP, Message-ID mémorisé, copie dans Envoyés selon `server_saves_sent_copy`), **reprise** (`Send` `InProgress` → recherche du Message-ID dans Envoyés → `Succeeded` ou `Uncertain`, jamais de renvoi automatique, décision 9), `resolve_uncertain_send`. Spec : `docs/specs/J6-J7-messages-actions.md` J7 §2, §3, §7, §8. Tests S7, S8, S9, S10, invariants 17, 18, 19, 22, 26.
3. **J9a** : terminer le CLI pour les domaines déjà exposés, avec `invariant_6_every_request_has_a_cli_command` (énumération exhaustive des variantes de `Request`).
4. **J8 restant** (façade) : domaines Boîtes, Lecture (`ListMessages`, `GetMessage`, `GetMessageBody`, `FetchMessageBody` en tâche longue, `GetRawSource`, pièces jointes avec `SaveAttachment` atomique, `GetSafeHtml`, `AllowRemoteContent`), Fils, Recherche (+ `SearchServer`), Actions (`ApplyAction` en lot, `UndoAction`), Rédaction (brouillons, `SendDraft`, `CancelSend`, `ListOutbox`, `ResolveUncertainSend`), File d'actions, Synchro (`StartSync`, `PauseSync`, `GetSyncStatus`, `ExtendWindow`, `ResyncMailbox`), Maintenance (stats, purge du cache, reconstruction de l'index, intégrité, export de diagnostic, **export du profil D-29**). Brancher la première synchro sur `AddAccount`. Achever au démarrage une suppression de compte interrompue. Test de contrat : instantané JSON versionné de chaque variante. Spec : `docs/specs/J8-J9-facade-cli.md`.
5. **J9 complet** : une sous-commande par variante, puis **S1–S14 exécutés via le CLI** contre le testkit.
6. **Revue indépendante** (agents `relecteur` puis `qualite-securite`) sur tout le code fusionné, en priorité `auth/`, `smtp/`, secrets ; corriger chaque remarque (test de non-régression d'abord).
7. **J10** : suppression des fichiers de cache de pièces jointes avec le miroir et à la suppression d'un compte (AG-13), proptest sur les invariants 7–12, fuzz léger des parseurs, perf à 100 000 messages, `docs/GUIDE-RUST-POUR-CPP.md`, mise à jour de `ARCHITECTURE-IMPLEMENTEE.md`, `README.md` (commandes CLI), nouveau `RAPPORT-FINAL.md`.

Critères de « terminé » : voir `CLAUDE.md` (§ Définition de « terminé »).

## 7. Méthode de travail qui a fonctionné

- **Orchestrateur + sous-agents** : l'agent principal écrit les specs, intègre, révise ; les implémenteurs travaillent en **worktree** (`isolation: worktree`), ≤ 3 en parallèle. Les types d'agents personnalisés sont dans `.claude/agents/` (`implementeur-rust`, `auteur-de-tests`, `relecteur`, `qualite-securite`, `chasseur-de-bugs`, `testkit-protocole`, `architecte`, `documentaliste`) ; ils sont disponibles quand la session a démarré avec ces fichiers présents.
- **Consigne type d'un implémenteur** : chemin de la spec, briques existantes à réutiliser (avec leurs noms exacts), fichiers autorisés et interdits (pour éviter les conflits entre agents), rappel A6 (aucun nom de fournisseur dans le cœur, tests compris), normes, `CARGO_BUILD_JOBS=3`, `CARGO_INCREMENTAL=0`, sorties dans `target/rapports/`, `./ci.sh` vert avant chaque commit, **commits tôt et souvent**, rapport final court.
- **Intégration** : `git merge --no-ff <branche>`, résoudre les conflits (presque toujours `lib.rs`, `migrations/mod.rs`, `repositories/mod.rs`, `Cargo.toml`/`Cargo.lock`), `./ci.sh`, commit, `git worktree remove --force <worktree>`, consigner les décisions de l'agent en AG-xx, `tools/update-invariants-table`, journal.
- **Vérifier après chaque fusion** qu'aucun marqueur de conflit n'a été commité (c'est arrivé une fois ; `tools/check-forbidden-patterns` le détecte désormais) et que les modules ajoutés sont bien déclarés (un agent interrompu avait laissé `labels.rs` et `idle.rs` non déclarés, donc jamais compilés).

## 8. Outils et garde-fous (`./ci.sh` = `make check`)

| Outil | Vérifie |
|---|---|
| `tools/test-check-norms`, `tools/test-check-architecture` | Auto-tests des garde-fous (fixtures et workspace saboté) |
| `tools/check-norms` | Normes §11 : N1 ≤ 30 lignes, N2 ≤ 6 fonctions, N3 noms fourre-tout, N4 aucun commentaire dans un corps, N5 exactement une ligne `///` |
| `tools/check-architecture` | A1 vocabulaire, A2 providers, A3 cœur, A4 CLI, A5 testkit, A6 aucun nom de fournisseur dans le cœur, A7 pas de chargement dynamique, A8 chaque bibliothèque externe dans son module, A9 pas de notion d'écran |
| `tools/check-forbidden-patterns` | `unwrap`/`expect`/`panic!`/`todo!`/`unsafe` hors tests, TODO non référencé, marqueurs de conflit |
| `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo deny check bans licenses sources` | Standard |
| `tools/update-invariants-table` | Régénère `docs/ETAT/INVARIANTS.md` à partir des tests nommés `invariant_<n>_…` / `scenario_s<n>_…` |

Conventions de tests : intégration dans `engine/crates/<crate>/tests/<thème>.rs`, première ligne `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`, ≤ 6 tests par fichier, `tokio::time::timeout` autour de tout ce qui touche le réseau local. Supports existants dans `mira-core/tests/` : `support/`, `support_mirror/`, `support_imap/`, `support_api/`, `support_actions/`, `provider_fakes/`.

## 9. Pièges connus de l'environnement

- **Limites d'usage de l'API** (HTTP 429) et **redémarrages de session** ont interrompu les agents à plusieurs reprises (environ 7 h perdues en tout). Parade : commits fréquents ; à la reprise, inspecter chaque worktree (`git log feat/E-01-engine-overnight..HEAD`, `git status`) puis relancer l'agent depuis son historique (`SendMessage`) s'il existe encore, sinon un nouvel agent avec la même consigne en lui indiquant la branche à reprendre.
- **Disque** `/home` (460 Go, ~88 Go libres au 2026-10-14) : il a atteint 99 % à cause des `target/` des worktrees (16–30 Go chacun). Les profils dev/test du workspace sont allégés (AG-15). Supprimer le worktree d'un agent dès sa fusion ; surveiller `df -h /home`.
- **RAM** limitée (14 Go, ~4 Go libres) : pas plus de 3 compilations simultanées, `CARGO_BUILD_JOBS=3`.
- **Hooks du plugin `ecc`** : `GateGuard` exige de « présenter des faits » avant la première commande ou écriture (lancer la session avec `ECC_GATEGUARD=off`) ; un hook refuse `git commit --no-verify` ; un hook de « boucle » signale à tort les lectures par tranches.
- **cargo-nextest** ne s'installe pas (`aws-lc-sys`) : `./ci.sh` utilise `cargo test` (BLOQUÉ-01). `cargo-deny` et `cargo-llvm-cov` sont installés (llvm-cov sans le composant `llvm-tools`).
- La toolchain par défaut de la machine est une vieille nightly : le workspace épingle `1.99.0` via `engine/rust-toolchain.toml`.
- Ne pas utiliser `| head` derrière une compilation cargo (SIGHUP a déjà interrompu un build) : rediriger vers un fichier.

## 10. Commandes utiles

```bash
cd /home/nnoah/_WORKS/DeV/MIRA
./ci.sh > target/rapports/ci.log 2>&1; echo $?; grep -E "VERT|error|FAILED" target/rapports/ci.log
git worktree list
git -C .claude/worktrees/<agent> log --oneline feat/E-01-engine-overnight..HEAD
git merge --no-ff --no-edit <branche> -m "merge: <jalon> (<branche>)"
tools/update-invariants-table
cd engine && cargo run -q -p mira-cli -- --json provider detect alice@example.org
```
