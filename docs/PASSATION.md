# Passation — moteur MIRA (état au 2026-10-14)

Document destiné à l'agent (ou au développeur) qui reprend le travail. Il est **autoporteur** : avec lui, `CLAUDE.md` et les fichiers qu'il cite, on peut reprendre sans autre contexte. En cas de contradiction, l'ordre d'autorité est : `docs/MIRA-architecture-reference.md` (référence, lecture seule) > `docs/ETAT/DECISIONS-DE-DEPART.md` (réponses de l'utilisateur) > `docs/DECISIONS-AGENT.md` (décisions d'implémentation AG-01 à AG-34) > ce document.

Les faits chiffrés ci-dessous ont été vérifiés le 2026-10-14 par lecture du code, `grep` et lecture de `target/rapports/tests.log`.

---

## 1. Le projet en deux paragraphes

MIRA est un client mail Linux natif, multi-comptes, **offline-first**. Ce dépôt contient le **moteur en Rust** (« engine »), utilisable sans interface : d'abord par un CLI (`mira`), plus tard par une interface Flutter. Le porteur du projet (l'utilisateur) est développeur C++ (projet Arcade), peu à l'aise en Rust : le code doit rester simple et lisible, et les explications vont dans la documentation, pas dans le code.

Principes structurants (référence ch. 2) : le cœur ne connaît aucun fournisseur (Gmail, Outlook…) et raisonne en **capacités** du serveur ; les providers sont de pures descriptions ; un message existe **une seule fois** et les boîtes en sont des vues (apparitions) ; le serveur gagne toujours, les actions locales sont rejouées par-dessus ; lire = local, écrire = intention mise en file ; aucun secret hors du trousseau ; tout est testable sans réseau (faux serveurs).

## 2. Où lire quoi

| Fichier | Contenu |
|---|---|
| `CLAUDE.md` | Mémoire de session : règles absolues, normes de codage, boucle de travail, commandes. **À relire en premier.** |
| `docs/MIRA-architecture-reference.md` | La référence (2 115 lignes) : décisions D-01…D-34, invariants 1–33, scénarios S1–S14, contrat d'API (ch. 12). **Ne jamais la modifier.** |
| `docs/ETAT/DECISIONS-DE-DEPART.md` | Réponses de l'utilisateur au questionnaire initial (périmètre, dépendances, CLI, git, règles). |
| `docs/DECISIONS-AGENT.md` | Décisions et limites d'implémentation AG-01 → AG-34, avec raison et retour arrière. **Les AG-27 à AG-34 décrivent l'état réel, limites et « non traité » compris.** |
| `docs/ARCHITECTURE-IMPLEMENTEE.md` | Correspondance D-01…D-34 → code, les 26 modules du cœur, flux principaux, garde-fous, limites connues. |
| `docs/PROPOSITIONS-REFERENCE.md` | Corrections et précisions proposées à la référence (PR-01 → PR-17). |
| `docs/RAPPORT-FINAL.md` | Rapport d'étape au 2026-10-14. |
| `docs/specs/*.md` | Spécifications détaillées par jalon, écrites avant le code : `CORE-ARCHITECTURE.md` (carte historique des modules et schéma SQL ; le schéma a évolué, voir `ARCHITECTURE-IMPLEMENTEE.md`), `J1`…`J8-J9`. |
| `docs/ETAT/PLAN.md` | Statut des jalons J0–J10 et critères d'acceptation. J0–J9 FAIT, J10 EN COURS. |
| `docs/ETAT/JOURNAL.md` | Chronologie (fin du fichier = notes de reprise). |
| `docs/ETAT/INVARIANTS.md` | Table invariants 1–33 et scénarios S1–S14 → tests (régénérée par `tools/update-invariants-table`). |
| `docs/ETAT/REVUE-RELECTEUR.md`, `AUDIT-SECURITE.md` | Revue indépendante (31 remarques) et audit qualité/sécurité (remarques numérotées jusqu'à 20, 18 présentes dans le fichier : les numéros 7 et 19 n'y figurent pas). Leur traitement est dans AG-27 à AG-30. |
| `docs/ETAT/BLOCAGES.md`, `TESTS.md`, `HORLOGE.md`, `ENVIRONNEMENT.md` | **`TESTS.md` et `HORLOGE.md` sont périmés** (relevés du 2026-10-11, 478 tests) ; `ENVIRONNEMENT.md` décrit la machine locale d'origine. À remettre à jour. |
| `docs/GIT_WORKFLOW.md` | Workflow git de l'utilisateur (main / dev / feat/E-XX). Ne pas modifier. |

## 3. Règles non négociables (résumé de `CLAUDE.md`)

1. Pas de navigateur ni de computer use ; réseau : crates.io via cargo, git, WebSearch/WebFetch **pour se documenter seulement** (sources à consigner dans `docs/SOURCES.md`, qui est **vide** à ce jour).
2. **Aucun compte mail réel** : tout est testé contre le testkit sur `127.0.0.1`.
3. **Git** : `git push` autorisé **vers les branches `feat/E-01-*` seulement** (instruction explicite de l'utilisateur du 2026-10-14, AG-18) ; jamais vers `dev`/`main`, jamais de fusion dans `dev`/`main`, pas de changement de remote, pas de `sudo`. **La suppression de branches distantes est refusée par le système** : ne pas essayer (les branches `worktree-agent-*` et `feat/E-01-*` d'agents restent sur `origin`).
4. Pas d'`unsafe` ; pas d'`unwrap`/`expect`/`panic!` hors tests ; **aucun secret** dans journaux, erreurs, événements, réponses, base.
5. Frontière : `mira-core` ne nomme aucun fournisseur (même dans ses tests : faux providers `fake_labels`, `fake_folders`, `fake_fallback`, `fake_oauth`, noms d'identifiants « stable ») ; les providers ne dépendent que de `mira-vocabulary` ; seul `mira-assembly` connaît tout.
6. Normes (vérifiées mécaniquement) : fonction ≤ 30 lignes ; ≤ 6 fonctions avec corps par fichier (tests compris) ; **aucun commentaire dans un corps de fonction** ; exactement une ligne `///` en français avant chaque `fn` ; pas de `utils.rs`/`helpers.rs`/`common.rs`/`misc.rs` ; tests dans des fichiers séparés ; le cœur n'emploie pas le mot « fenêtre » (A9 : écrire « plage », identifiants `window_*`).
7. Jamais supprimer ni affaiblir un test pour passer au vert ; `#[ignore]` seulement après 3 cycles de blocage, avec `BLOQUÉ-<id>` (un seul test ignoré à ce jour : le vrai trousseau, manuel).
8. **Un commit par fonctionnalité**, dès que `./ci.sh` est vert ; fusions `--no-ff` sans squash. Trailer `Co-Authored-By` de `CLAUDE.md` plus la ligne `Claude-Session:` que fournit la session courante.
9. **Consigne de l'utilisateur (2026-10-11)** : continuer jusqu'à ce que J5–J10 soient FAITS, sans raccourci, sauf arrêt demandé. **Ne poser aucune question** : trancher par l'option la plus simple et réversible, la consigner dans `DECISIONS-AGENT.md`.

## 4. État actuel

**Branche de travail** : `feat/E-01-engine-overnight` (depuis `dev`), dernier commit `34ef005`, 208 commits depuis `origin/dev` dont 32 fusions. **`./ci.sh` vert : 1 080 tests réussis, 0 échec, 1 ignoré** ; `check-norms` : 1 433 fichiers, 0 violation ; `check-architecture` : 9 crates, 0 violation. Tags locaux : `jalon/J0` à `J9` et `jalon/J10-partiel` (**`J5`, `J7` et `J8` pointent le même commit `5cf2b8a`** ; ce ne sont pas des instantanés distincts). Au moment d'écrire, seuls les quatre documents de fin de ce lot (`ARCHITECTURE-IMPLEMENTEE.md`, `PROPOSITIONS-REFERENCE.md`, `PASSATION.md`, `RAPPORT-FINAL.md`) sont modifiés et non commités ; `README.md` et `GUIDE-RUST-POUR-CPP.md` sont confiés à un autre agent.

Jalons : **J0–J9 FAITS** ; **J10 partiel** (voir §7).

### Workspace `engine/` (toolchain `1.99.0`, édition 2024)

| Crate | Rôle | État |
|---|---|---|
| `mira-vocabulary` | Identifiants opaques `Id<K>`, entités D-19, `MiraError` (catégorie + code stable), 42 `EventKind`, `ContractVersion` (1.0), trait `Provider` | Fait |
| `mira-core` | Le moteur, 26 modules (tableau ci-dessous) | Fait, voir limites §7 |
| `mira-provider-{gmail,outlook,yahoo,generic}` | Descriptions pures (données du §6.7, **non vérifiées en ligne**) | Fait |
| `mira-assembly` | `build_registry()`, `production_config(profil)` (synchro de fond active, `OAuthClientConfig` **vide**), réexports | Fait |
| `mira-cli` | Binaire `mira` : 72 commandes, `--json` versionné, `--profile`, codes de sortie 0/10–19/84 | Fait |
| `mira-testkit` | Faux IMAP (CONDSTORE/QRESYNC, IDLE, MOVE, UIDPLUS, libellés, pannes, renumérotation), faux SMTP (pannes, coupure après DATA), faux OAuth2 PKCE ; **en clair seulement** | Fait |

### Modules de `mira-core/src/` (tous fusionnés)

`accounts` (cycle de vie, suppression S14) · `actions` (file locale, rebase, rejeu serveur) · `api` (façade : `Engine`, 16 domaines, 71 variantes de `Request`, un handler par opération) · `auth` (OAuth2 PKCE, coffre, rafraîchissement) · `body_storage` · `capabilities` · `drafts` · `events` (bus numéroté persistant) · `fetching` (téléchargements explicites) · `html` (`ammonia`, liste blanche) · `imap` (client maison) · `mailboxes` · `maintenance` (purge, diagnostic, export de profil D-29) · `mime` · `operations` (tâches longues) · `ports` (horloge, ids, trousseau) · `profile` (chemins, verrou D-27) · `reading` (lectures locales) · `registry` · `replayer` (rejeu de fond par compte) · `send` (envoi, reprise, décision sur envoi incertain) · `smtp` · `storage` (SQLite, migrations v001–v005, dépôts) · `sync` (descente, stratégies, identité, renumérotation, plage, temps réel, superviseur) · `sync_service` · `threads`. Détail et fichiers : `docs/ARCHITECTURE-IMPLEMENTEE.md` §2.

### Couverture des invariants et scénarios

Les 33 invariants et les 14 scénarios ont chacun au moins un test nommé ou un garde-fou (`docs/ETAT/INVARIANTS.md`) ; les scénarios S1–S14 sont exécutés **via le CLI en processus** contre le testkit. **La revue de chaque invariant « un par un » (au-delà de l'existence d'un test) n'est pas faite** (J10).

## 5. Méthode de travail qui a fonctionné

- **Orchestrateur + sous-agents** : l'agent principal écrit les specs, intègre, révise ; les implémenteurs travaillent en **worktree** (`.claude/worktrees/`), ≤ 3 en parallèle. Les rôles sont dans `.claude/agents/` (`implementeur-rust`, `auteur-de-tests`, `relecteur`, `qualite-securite`, `chasseur-de-bugs`, `testkit-protocole`, `architecte`, `documentaliste`). Quand les types personnalisés ne sont pas chargés, lancer `general-purpose` en demandant de lire d'abord le fichier de rôle (AG-02) ; seuls `architecte`, `relecteur` et `qualite-securite` ont le web.
- **Consigne type d'un implémenteur** : chemin de la spec, briques existantes à réutiliser (noms exacts), fichiers autorisés et interdits (éviter les conflits), rappel de la frontière (aucun nom de fournisseur dans le cœur, tests compris), normes, variables d'environnement de build (§6), sorties dans `target/rapports/`, `./ci.sh` vert avant chaque commit, **commits tôt et souvent**, rapport final court.
- **Intégration** : `git merge --no-ff --no-edit <branche>`, résoudre les conflits (presque toujours `lib.rs`, `migrations/mod.rs`, `repositories/mod.rs`, `Cargo.toml`/`Cargo.lock`), `./ci.sh`, commit, supprimer le worktree fusionné, consigner les décisions de l'agent en AG-xx, `tools/update-invariants-table`, mettre à jour PLAN et JOURNAL.
- **Vérifier après chaque fusion** : aucun marqueur de conflit commité (`tools/check-forbidden-patterns` le détecte) et tout module ajouté est bien déclaré dans son `mod.rs`.
- **Revue** : un relecteur indépendant de l'auteur. La revue du 2026-10-14 (31 remarques) et l'audit (18 entrées) ont été traités en cinq lots A à E (branches `feat/E-01-fix-{a-send-mime,b-sync-budget,c-html-auth,d-send-uncertain,e-erase-wait}`) : cause racine, test de non-régression d'abord.
- **Perte de travail** : tout ce qui n'est pas commité **et poussé** est perdu si le conteneur cloud disparaît (c'est arrivé : AG-17, J5b/J7c/J9a refaits depuis les specs). Commiter souvent ; pousser les branches `feat/E-01-*` (AG-18).

## 6. Outils, garde-fous et environnement

| Outil | Vérifie |
|---|---|
| `./ci.sh` (= `make check`) | tout, dans l'ordre ci-dessous |
| `tools/test-check-norms`, `tools/test-check-architecture` | auto-tests des garde-fous (fixtures `tools/fixtures/{ok,bad}`, workspace saboté) |
| `tools/check-norms` | N1 ≤ 30 lignes, N2 ≤ 6 fonctions/fichier, N3 noms fourre-tout, N4 aucun commentaire dans un corps, N5 exactement une ligne `///` |
| `tools/check-architecture` | A0 crates présents, A1 vocabulaire, A2 providers, A3 cœur, A4 CLI, A5 testkit, A6 aucun nom de fournisseur dans le cœur, A7 pas de chargement dynamique, A8 chaque bibliothèque externe dans son module, A9 pas de notion d'écran |
| `tools/check-forbidden-patterns` | `unwrap`/`expect`/`panic!`/`todo!`/`unsafe` hors tests, TODO non référencé (citer `AG-n` ou `BLOQUÉ-id`), marqueurs de conflit, `libloading` |
| `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, `cargo deny check bans licenses sources` | standard (avis de sécurité exclus : AG-07) |
| `tools/update-invariants-table` | régénère `docs/ETAT/INVARIANTS.md` depuis les tests `invariant_<n>_…` / `scenario_s<n>_…` |

Conventions de tests : intégration dans `engine/crates/<crate>/tests/<thème>.rs`, première ligne `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`, ≤ 6 tests par fichier, `tokio::time::timeout` autour de tout ce qui touche le réseau local. Supports dans `mira-core/tests/` (`support*/`, `provider_fakes/`) et `mira-cli/tests/cli_support/`.

**Pièges connus de l'environnement**
- **Disque** : le conteneur cloud partage ≈ 250 Go entre sessions ; un `target/` de workspace complet pèse plusieurs Go (jusqu'à 16–30 Go avec les symboles de débogage, d'où AG-15). Construire avec `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0` (et `CARGO_BUILD_JOBS` borné), **supprimer le worktree d'un agent dès qu'il est fusionné** (`git worktree remove --force`), surveiller `df -h`. Sur l'ancienne machine locale, `/home` avait atteint 99 %.
- **Mémoire** : pas plus de 3 compilations simultanées (14 Go sur la machine d'origine).
- **Limites d'usage de l'API** (HTTP 429) et redémarrages de session ont interrompu les agents à plusieurs reprises. À la reprise : `git worktree list`, `git -C <worktree> log <branche-de-travail>..HEAD`, `git status`, puis relancer l'agent depuis son historique s'il existe, sinon un nouvel agent avec la même consigne et la branche à reprendre.
- **Toolchain** : la toolchain par défaut de la machine d'origine était une vieille nightly ; `engine/rust-toolchain.toml` épingle `1.99.0` (installée par rustup au premier appel). `cargo-nextest` ne s'installe pas (`aws-lc-sys`, BLOQUÉ-01) : `ci.sh` utilise `cargo test`. `cargo-deny` et `cargo-llvm-cov` étaient installés sur la machine d'origine (couverture jamais mesurée, composant `llvm-tools` absent).
- Ne pas mettre `| head` derrière une compilation cargo (SIGHUP a interrompu un build) : rediriger vers un fichier.
- Hooks du plugin `ecc` sur la machine d'origine (`GateGuard`, détecteur de boucle) : session lancée avec `ECC_GATEGUARD=off` ; sans objet dans le conteneur cloud si le plugin n'y est pas.
- Le dossier `engine/target/` et `target/rapports/` (journaux de CI : `ci.log`, `tests.log`, `contract.log`) ne sont pas versionnés.

## 7. Ce qui reste à faire (liste exacte)

### 7.1 J10 — durcissement (ordre conseillé)
1. **Défaut `html::sanitize_html` quadratique en profondeur d'imbrication** (AG-34) : 1 000 niveaux 68 ms, 10 000 niveaux 5,3 s, 100 000 niveaux > 10 min ; aucune borne dans `C/html/`. Correctif : plafonner taille et profondeur **avant** `ammonia`, puis un test de non-régression chronométré (écrire le test d'abord). Appelants : `C/api/handlers/message/safe_html.rs` (hors verrou de base, `spawn_blocking`).
2. **UTF-7 modifié** (RFC 3501 §5.1.3) : absent (aucune occurrence dans le code). À faire : encodage dans `imap/protocol/quoting.rs`, décodage dans `imap/parse/list.rs` et pour `X-GM-LABELS` (AG-12 le suppose déjà fait), `SEARCH CHARSET UTF-8` avec littéral (`imap/protocol/literal_command.rs`) pour la recherche serveur (revue #11, jamais traitée). Le testkit devra parler UTF-7 pour les tests.
3. **Fuzz léger** des parseurs MIME (proptest « ne panique jamais » existe déjà), HTML, réponses IMAP (`imap/parse`, `imap/protocol`, tests internes `#[cfg(test)]`).
4. **Performance à 100 000 messages** : `mira-core/tests/reading_performance.rs` mesure 20 000 (`MESSAGES`) ; son en-tête promet une mesure à 100 000 « consignée dans `docs/ETAT/TESTS.md` » qui n'existe pas. Cible : page < 50 ms (décision de départ 6, D-32).
5. **Revue des invariants 1–33 et S1–S14 un par un** (le test nommé couvre-t-il vraiment l'invariant ?).
6. **Remettre à jour** `docs/ETAT/TESTS.md` (1 080 tests), `HORLOGE.md`, `INVARIANTS.md` (`tools/update-invariants-table`), `PLAN.md` (le J10 passera à FAIT) après chaque étape ; vérifier que `README.md` et `docs/GUIDE-RUST-POUR-CPP.md` (écrits par un autre agent) sont commités.

### 7.2 « Non traité » des AG-27 à AG-34 (repris tels quels, avec leur statut au 2026-10-14)
- **AG-27** : fournisseur d'identifiants unique pour `ImapAccess` (rafraîchissement automatique du jeton : aujourd'hui `fetching/access.rs` utilise le jeton rangé sans rafraîchir, AG-22). *(La purge de la copie du brouillon dans `recover_interrupted_send` et l'audit #4 ont été réglés par AG-30.)*
- **AG-28** : boîte de destination traitée avant la source (désarchivage vers la réception) crée encore un nouveau message — fusion en fin de passe non faite ; deux copies identiques à ±60 s tenues pour la même — persister « tentative commencée » ; `extend_window` s'exécute encore en parallèle de la tâche du compte ; l'attente du rejeu à la suppression d'un compte n'a pas de test déterministe ; le repli VANISHED (> 1 M d'UID ou `*`) n'a pas de test d'intégration.
- **AG-29** : revue #28 (`X-GM-EXT-1` dans `imap/operations/labels.rs` et `\Inbox` dans `actions/replay/execute/labels.rs` codés en dur dans le cœur : à tirer des particularités du provider) ; audit #17 (`zeroize` : secrets non effacés de la mémoire) ; mutex `std` de la base tenu sur des threads tokio hors trousseau/`GetSafeHtml`/`CheckIntegrity`. Défense complète de `UpdateAccount` avec d'autres serveurs = changement de contrat.
- **AG-30** : rien de non traité ; critère d'« issue inconnue » = catégorie `Network` hors `network.unreachable` ; une erreur `Tls` en cours d'envoi est classée passagère (à auditer) ; la distinction « injoignable » / « session perdue » repose sur le libellé `"Connection error"` de `lettre` (`smtp/errors.rs`), fragile à une montée de version.
- **AG-31** : écoute IDLE continue inexistante côté CLI (une commande `watch` de longue durée serait un ajout) ; comportements « corps préchargé » et ordre du préchargement conservés (PR-14, PR-15).
- **AG-32** : limites des tests en processus (annulation pendant `extend-window`, suppression pendant synchro, `reauthenticate` + `oauth-complete` en deux processus) ; `sync extend-window --days` s'ajoute à la plage courante.
- **AG-33** : pas de `VACUUM` ; les bases créées avant le correctif ne sont pas purgées rétroactivement.
- **AG-34** : voir 7.1.
- **Plus anciens, toujours ouverts** : AG-21/AG-25 (message arrivé entre la fin de la première passe et l'établissement d'IDLE vu seulement au relevé suivant ; annulation de `extend_window` testée seulement avant le premier lot) ; AG-26 (le `warn!` « première synchro non lancée » d'`AddAccount` est masqué par une directive du CLI : à supprimer côté moteur quand la synchro est `Off` ; il manque une API moteur pour attendre la première synchro lancée par `AddAccount`) ; AG-14 (le test de connexion à l'ajout ne teste pas SMTP).

### 7.3 Écarts constatés à la relecture de ce lot de documentation (aucun n'est consigné dans un AG)
- `maintenance::discard_mirror_and_cache` (`C/maintenance/account_cache.rs`) supprime les fichiers de cache puis la base **sans** `Database::erase_traces()`, contrairement à `erase_account` ; `Database::discard_mirror_of` n'a pas d'appelant en production ; aucune requête de la façade n'expose « jeter le miroir ». AG-33 laisse entendre le contraire.
- Sept `EventKind` sur 42 ne sont jamais émis : `EngineFatalError`, `MailboxUpdated`, `AttachmentProgress`, `ThreadUpdated`, `QuotaFull`, `CertificateNeedsApproval`, `RateLimited`.
- `production_config` n'a aucun `client_id` OAuth (`OAuthClientConfig::default()`) : `mira account add-oauth` échoue hors configuration injectée. Aucune lecture de configuration (fichier ou variable d'environnement) n'existe pour ces identifiants.
- Les données des providers (serveurs, adresses OAuth, plafonds de connexions, copie dans Envoyés) viennent du §6.7 de la référence et n'ont pas été vérifiées en ligne ; `docs/SOURCES.md` est vide.
- Aucun test ne réussit une poignée de main TLS ni STARTTLS (le testkit est en clair, AG-03/AG-10) ; seuls le refus du clair et l'échec de poignée de main sont testés.
- `imap/mod.rs` porte `#![allow(dead_code)]` ; A8 (`tools/check-architecture`) liste encore `async_imap`, retiré du projet (AG-29).
- Les documents d'état `TESTS.md` et `HORLOGE.md` n'ont pas été mis à jour depuis le 2026-10-11.

### 7.4 Au-delà de la v1 (reporté par les décisions de départ)
Import de profil (D-29), processus moteur séparé (D-30), réglages d'application D-11, optimisation « Tous les messages » D-24, DNS MX, relais d'images, `sqlite-vec`, vues enregistrées, contacts, règles, PGP/S-MIME, Flutter (phase 6) et IA (phase 8).

## 8. Commandes utiles

```bash
cd /home/user/MIRA                                  # clone cloud (origin = https://github.com/nnoah29/MIRA)
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=4
./ci.sh > target/rapports/ci.log 2>&1; echo $?      # tout vérifier ; puis : grep -E "TOUT EST VERT|error|FAILED" target/rapports/ci.log
grep -E "test result" target/rapports/tests.log      # décompte des tests
tools/update-invariants-table                        # régénère docs/ETAT/INVARIANTS.md
git worktree list
git -C .claude/worktrees/<agent> log --oneline feat/E-01-engine-overnight..HEAD
git merge --no-ff --no-edit <branche> -m "merge: <jalon> (<branche>)"
git push origin feat/E-01-engine-overnight           # autorisé pour feat/E-01-* seulement (AG-18)
cd engine && cargo test -p mira-core --test reading_performance
cd engine && cargo run -q -p mira-cli -- --json provider detect alice@example.org
cd engine && cargo run -q -p mira-cli -- --help      # 16 domaines, 72 commandes
```
