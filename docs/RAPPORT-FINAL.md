# Rapport d'étape — moteur MIRA, au 2026-10-14

Ce rapport remplace celui de la première nuit (2026-10-11, périmé). Ce n'est **pas** un rapport de fin de projet : J0 à J9 sont faits, **J10 est partiel** et un défaut connu n'est pas corrigé. Les chiffres ont été vérifiés le jour de l'écriture (lecture des sources, `grep`, `target/rapports/tests.log` du dernier `./ci.sh`).

## 1. En bref

| Élément | Valeur |
|---|---|
| Branche | `feat/E-01-engine-overnight`, dernier commit `34ef005` (2026-10-14 12:53 UTC) |
| Historique | 208 commits depuis `origin/dev`, dont 32 fusions `--no-ff` sans squash ; première nuit du 2026-10-11 (≈ 3 h 35 de travail effectif sur 7 h 16 d'horloge, le reste perdu en limites d'usage et redémarrages), reprise du 2026-10-14 (07:48 → 12:53 UTC, environ 5 h) |
| `./ci.sh` | vert : **1 080 tests réussis, 0 échec, 1 ignoré** (le vrai trousseau, test manuel prévu) ; `check-norms` 1 433 fichiers, 0 violation ; `check-architecture` 9 crates, 0 violation ; motifs interdits OK ; fmt, clippy `-D warnings`, `cargo deny` (bans, licences, sources) OK |
| Code | 9 crates ; ≈ 38 600 lignes de Rust hors dossiers `tests/` ; 1 433 fichiers `.rs` ; 1 081 attributs de test (mira-core 716, mira-cli 191, mira-testkit 105, mira-vocabulary 32, providers 33, mira-assembly 4) |
| Contrat | 16 domaines, **71 variantes** de `Request`, un handler par opération ; version du contrat 1.0 ; 42 `EventKind` ; 11 catégories d'erreur ; instantané JSON versionné |
| CLI | **72 commandes** (`mira <domaine> <verbe>`), `--json` versionné, codes de sortie 0 / 10–19 / 84 |
| Stockage | SQLite par profil, migrations v001 à v005, FTS5, source brute zstd |
| Jalons | J0–J9 FAITS (tags locaux `jalon/J0` … `jalon/J9`, `jalon/J10-partiel`) ; J10 en cours |

Non mesuré : la couverture de lignes (décision de départ 27, indicative) ; aucun outil de couverture n'a été exécuté.

## 2. Livré par jalon

| Jalon | Statut | Ce qui existe |
|---|---|---|
| J0 Squelette | FAIT | workspace de 9 crates, toolchain `1.99.0`, `ci.sh`/`make check`, `deny.toml`, garde-fous `check-norms` / `check-architecture` / `check-forbidden-patterns` avec auto-tests, 8 rôles d'agents |
| J1 Vocabulaire | FAIT | identifiants opaques, entités D-19, erreurs à code stable, événements numérotés, trait `Provider` |
| J2 Stockage | FAIT | base par profil, trois familles de données, correspondance D-13, source zstd, FTS5, file d'actions idempotente, événements, ports (horloge, ids, trousseau), verrou de profil D-27 |
| J3 Providers | FAIT | gmail, outlook, yahoo, generic purs ; registre avec repli ; capacités effectives ; rôles de dossiers |
| J4 Testkit | FAIT | faux IMAP (CONDSTORE/QRESYNC, IDLE, MOVE, libellés, pannes, renumérotation), faux SMTP, faux OAuth2 PKCE |
| J5 Synchronisation | FAIT | client IMAP maison, MIME tolérant, descente par étapes, stratégies QRESYNC / CONDSTORE / comparaison complète, renumérotation, identité et recoupement (P7), plage et « remonter plus loin », IDLE + relevé + reconnexion, superviseur (une tâche par compte), budget de connexions |
| J6 Lecture | FAIT | listes par boîte et vues standard, corps hors ligne, recherche, fils D-15, perf sur 20 000 messages |
| J7 Écritures | FAIT | file locale (effet optimiste, inverse, rebase D-23), rejeu serveur ordonné, erreurs classées, brouillons, envoi avec reprise sur plantage et issue inconnue, `ResolveUncertainSend` |
| J8 Façade | FAIT | `Engine::open/execute/subscribe/shutdown(_within)`, comptes, identités, OAuth, lecture, écriture, synchro, maintenance (dont export de profil D-29), suppression de compte avec effacement des traces |
| J9 CLI | FAIT | 72 commandes ; S1–S14 exécutés via le CLI en processus contre le testkit ; invariant 6 par énumération des variantes |
| J10 Durcissement | **PARTIEL** | fait : assainissement HTML en liste blanche, effacement sûr (S14), suppression du cache de pièces jointes avec le miroir ou le compte, proptest des invariants 7–12 (12 propriétés). **Pas fait** : voir §6 |

Les tags `jalon/J5`, `jalon/J7` et `jalon/J8` désignent le même commit (`5cf2b8a`).

## 3. Revue indépendante et audit

Un relecteur indépendant a rendu **31 remarques** (`docs/ETAT/REVUE-RELECTEUR.md` : 1 bloquante, 12 importantes, 18 mineures) et un audit qualité/sécurité a rendu des remarques **numérotées jusqu'à 20, dont 18 figurent dans le fichier** (`docs/ETAT/AUDIT-SECURITE.md`, les numéros 7 et 19 manquent ; ses points « sains » sont listés). Ils ont été traités en lots A à E, cause racine d'abord, test de non-régression d'abord (AG-27 à AG-30, AG-33). Le rapprochement ci-dessous a été fait remarque par remarque d'après le texte des AG et les tests présents ; il est à contrôler avant de s'y fier.

| Remarques de la revue | Statut |
|---|---|
| 24 traitées : #1, 2, 3, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25, 26, 30 | avec limites pour #7 et #9 (voir §5) |
| 5 partielles : #4 (synchro de fond désactivée par défaut dans le CLI, mais le rejeu de fond démarre à chaque ouverture), #22 (rangement des corps unifié, pas le fournisseur d'identifiants), #27 (validité de la boîte comparée, mais exécution encore parallèle à la tâche du compte), #29 (test de connexion avec le secret rangé, pas de nouvelle saisie), #31 (trousseau et `GetSafeHtml`/`CheckIntegrity` hors verrou, pas le mutex `std` de la base) | — |
| 2 **non traitées** : #11 (UTF-7 modifié et `SEARCH CHARSET UTF-8`), #28 (`X-GM-EXT-1` et `\Inbox` codés en dur dans le cœur) | — |

| Remarques de l'audit (18 présentes) | Statut |
|---|---|
| 17 traitées : #1–6, 8–16, 18, 20 | #1 : l'assainissement HTML est sorti du verrou de base, mais **reste quadratique** en profondeur (voir §6) |
| 1 **non traitée** : #17 (`zeroize`, secrets non effacés de la mémoire) | — |

Ce que les relectures ont corrigé de plus grave : gel du moteur par un HTML→texte quadratique appelé sous verrou de base ; envoi annulé pouvant partir quand même (course annulation/rejeu) ; **doublon d'envoi possible sur délai SMTP** (désormais « issue inconnue », jamais renvoyé) ; connexions IMAP hors budget du compte ; contenu distant contournable par `ht&#9;tps://…` ; mémoire non bornée par un serveur IMAP hostile ; message « poison » rejouant indéfiniment ; suppression de compte laissant des fichiers de cache, des sujets et des adresses dans l'index et le WAL.

## 4. Invariants et scénarios

Les 33 invariants et les 14 scénarios ont **au moins un test nommé ou un garde-fou** ; la table est dans `docs/ETAT/INVARIANTS.md`. Invariants 1–5 : garde-fous mécaniques ; invariant 6 : énumération des variantes de `Request` ; 7–33 : tests (dont proptest pour 7–12). S1–S14 : exécutés via le CLI et, pour la plupart, aussi au niveau du moteur. Les critères de « terminé » de `CLAUDE.md` liés aux tests sont couverts : reprise sur plantage de la synchro (`sync_markers.rs`, 4 jeux de capacités) et de l'envoi (`send_interrupted.rs`, `actions_replay_crash.rs`), idempotence (`invariant_18_*`, `actions_submit_idempotency.rs`), absence de secrets (`invariant_15_*`, `api_secrecy*.rs`, `storage_no_secret.rs`).

**Ce que cela ne garantit pas** : l'existence d'un test nommé n'est pas une revue de l'invariant. Cette revue « un par un » n'a pas été faite (J10).

## 5. Limites et dette connues

Détail et références dans `docs/ARCHITECTURE-IMPLEMENTEE.md` §11 et `docs/PASSATION.md` §7.

- **Défaut non corrigé, mail hostile** : `html::sanitize_html` est quadratique en profondeur d'imbrication (1 000 niveaux 68 ms ; 10 000 niveaux 5,3 s ; 100 000 niveaux plus de 10 min). Aucune borne de taille ni de profondeur. Une lecture sûre d'un message malveillant peut bloquer le moteur (AG-34).
- **UTF-7 modifié absent** : les boîtes à nom non ASCII ne sont ni lues ni écrites correctement ; la recherche serveur n'envoie pas de `CHARSET UTF-8`.
- **Jamais exercé hors du banc d'essai** : aucun test n'effectue de poignée de main TLS ni de STARTTLS (le testkit parle en clair) ; aucune connexion réelle à un service ; les valeurs des quatre providers (serveurs, adresses OAuth, plafonds, copie dans Envoyés) viennent du §6.7 de la référence et n'ont pas été vérifiées en ligne (`docs/SOURCES.md` est vide) ; `production_config` n'a aucun `client_id` OAuth, donc `account add-oauth` est refusé sans configuration injectée.
- **Synchro** (AG-21, 25, 28) : désarchivage vers la réception traité avant la source = nouveau message ; deux copies identiques à ±60 s prises pour la même ; message arrivé entre la fin de la première passe et l'établissement d'IDLE vu seulement au relevé suivant ; `extend_window` parallèle à la tâche du compte ; repli VANISHED sans test d'intégration.
- **Envoi** : le critère d'issue inconnue est la catégorie `Network` hors `network.unreachable` ; une erreur `Tls` en cours d'envoi est classée passagère ; la distinction « injoignable » / « session perdue » lit le libellé d'erreur de `lettre`.
- **Effacement** : `discard_mirror_and_cache` (abandon du miroir) n'efface pas les traces FTS/WAL contrairement à la suppression de compte, et n'est exposé par aucune requête de la façade ; pas de `VACUUM` ; les bases créées avant AG-33 ne sont pas purgées après coup.
- **Événements** : 7 des 42 `EventKind` ne sont jamais émis (`EngineFatalError`, `MailboxUpdated`, `AttachmentProgress`, `ThreadUpdated`, `QuotaFull`, `CertificateNeedsApproval`, `RateLimited`).
- **CLI de courte durée** : pas d'écoute IDLE continue ; OAuth en deux processus impossible (la commande combinée `account add-oauth` le remplace) ; `search server-result` ne retrouve rien d'un autre processus.
- **Frontière** : deux littéraux propres à un fournisseur restent dans le cœur (`X-GM-EXT-1`, `\Inbox`), non détectés par A6 car ils ne portent pas le nom du fournisseur.
- **Réglages D-11 et optimisation D-24** non implémentés ; import de profil, processus moteur séparé : reportés par décision.
- **Documents d'état périmés** : `docs/ETAT/TESTS.md` (478 tests) et `HORLOGE.md` datent du 2026-10-11 ; `reading_performance.rs` renvoie à une mesure à 100 000 messages qui n'a jamais été consignée.
- **Organisation** : la première nuit a perdu environ 7 h en limites d'usage de l'API et redémarrages ; le 2026-10-14, des branches d'agents jamais poussées ont été perdues et J5b, J7c, J9a refaits depuis les specs (AG-17) ; des branches `worktree-agent-*` encombrent `origin` (leur suppression est refusée par le système).

## 6. Ce qui reste à faire

1. **Corriger `sanitize_html`** (plafonner taille et profondeur avant `ammonia`) avec test de non-régression chronométré.
2. **UTF-7 modifié** et `SEARCH CHARSET UTF-8` (avec évolution du testkit).
3. **Fuzz léger** MIME / HTML / réponses IMAP.
4. **Performance à 100 000 messages** (cible : page < 50 ms) et consignation des chiffres.
5. **Revue des invariants 1–33 et des scénarios S1–S14 un par un.**
6. Dette des AG-27 à AG-30 (fournisseur d'identifiants unique, particularités du provider pour les deux littéraux, `zeroize`, mutex de la base, fusion en fin de passe, tentative persistée pour les déplacements ambigus).
7. Brancher la configuration OAuth de production et vérifier les valeurs des providers (réseau autorisé pour se documenter seulement).
8. Tests TLS/STARTTLS contre un faux serveur à certificat de test.
9. Mettre à jour `TESTS.md`, `HORLOGE.md`, `PLAN.md` ; nettoyer les branches d'agents locales ; commiter et pousser les documents de fin.

Tant que 1 à 5 ne sont pas faits, **J10 n'est pas terminé** et les critères de « terminé » de `CLAUDE.md` ne sont pas tous atteints (échec explicite : défaut de sécurité connu, UTF-7, perf à 100 000, revue des invariants).

## 7. Décisions et propositions

Décisions d'implémentation : `docs/DECISIONS-AGENT.md` (AG-01 à AG-34). Propositions pour la référence : `docs/PROPOSITIONS-REFERENCE.md` (PR-01 à PR-17). Reprise : `docs/PASSATION.md`. Architecture réelle : `docs/ARCHITECTURE-IMPLEMENTEE.md`.
