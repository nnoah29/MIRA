# Plan des jalons

Statuts : À FAIRE · EN COURS · FAIT · BLOQUÉ. Les critères sont écrits avant le code ; chaque case cochée renvoie à un test nommé.
Les jalons suivent le §5 du brief et le chapitre 17 de la référence (phases 1–5, plus l'assainissement HTML de la phase 7).

## Vagues de parallélisme (≤ 3 sous-agents simultanés)

| Vague | Contenu | Pourquoi cet ordre |
|---|---|---|
| V0 | J0 (agent principal) ; installation de nextest / deny / llvm-cov en arrière-plan | Le garde-fou doit exister avant la première ligne |
| V1 | J1 : l'architecte spécifie les interfaces, puis un implémenteur écrit le vocabulaire | Tout dépend du vocabulaire |
| V2 | J2 stockage · J3 providers et registre · J4 testkit IMAP / SMTP / OAuth | Crates et fichiers disjoints |
| V3 | J5a client IMAP encapsulé et MIME · J6 messages, vues, fils (requêtes de stockage) · J4b compléments du testkit | J5 a besoin de J2 et J4 |
| V4 | J5b stratégies de synchro, IDLE, renumérotation · J7 file d'actions, SMTP, brouillons, envoi | J7 a besoin du client IMAP |
| V5 | J8 façade · auteur-de-tests : tests de contrat en boîte noire | La façade assemble tout |
| V6 | J9 CLI et scénarios S1–S14 · J10 durcissement | Le CLI reflète la façade |

## J0 — Squelette — FAIT (jalon/J0)

- [x] Branche `feat/E-01-engine-overnight`, référence commitée telle quelle, `engine_/` remplacé par `engine/`.
- [x] Workspace `engine/` : 9 crates (vocabulary, core, 4 providers, assembly, cli, testkit), `rust-toolchain.toml` (1.99.0), `rustfmt.toml`, lints de workspace (`unsafe_code = "forbid"`, clippy `unwrap_used`, `expect_used`, `panic` = deny hors tests).
- [x] `tools/check-norms` (Python 3, bibliothèque standard) : ≤ 30 lignes par fonction, ≤ 6 fonctions par fichier, aucun commentaire dans un corps, `///` avant chaque `fn`, noms de fichiers interdits. Ses propres tests (fixtures conformes et non conformes).
- [x] `tools/check-architecture` (Python 3 + `cargo metadata`) : le cœur ne dépend d'aucun provider ; un provider ne dépend que de `mira-vocabulary` (ni cœur, ni tokio, ni rusqlite, ni réseau, ni autre provider) ; le vocabulaire ne dépend que de serde et thiserror ; aucun nom de provider dans le code du cœur (invariant 1).
- [x] `./ci.sh` (fmt, clippy `-D warnings`, build, tests, deux garde-fous, grep unwrap/expect/panic hors tests), `Makefile` (`make check`), `deny.toml`.
- [x] `.claude/agents/` : les 8 rôles, outils minimaux ; web seulement pour architecte, relecteur et qualite-securite.
- [x] Fichiers d'état initialisés : HORLOGE, JOURNAL, PLAN, BLOCAGES, TESTS, INVARIANTS ; `docs/DECISIONS-AGENT.md`, `PROPOSITIONS-REFERENCE.md`, `SOURCES.md`.
- **Acceptation** : `./ci.sh` vert ; un fichier de test volontairement non conforme fait échouer `check-norms` ; une dépendance interdite ajoutée à un provider fait échouer `check-architecture`.

## J1 — Vocabulaire et domaine — FAIT

- [x] Identifiants opaques typés (profil, compte, identité, boîte, message, pièce jointe, fil, brouillon, action, opération, événement), UUID v7 sous forme de chaîne (invariant 30).
- [x] Entités D-19 et états (compte, message, action, brouillon) ; rôles de boîte ; drapeaux au niveau du message (D-14).
- [x] Erreur du contrat : catégorie, code stable, message, réessayable, action suggérée, contexte (invariant 31, §12.6).
- [x] Événements numérotés, catégories du §12.5, sans secret ni contenu (invariant 32) ; version du contrat (§12.9).
- [x] Interface provider (§6.3) : gestion d'une adresse, serveurs, méthodes d'authentification, rôle d'un dossier, limites, exceptions, traductions (identifiant stable, libellés, indice de fil, copie automatique dans Envoyés).
- **Acceptation** : tests unitaires de sérialisation JSON stable, codes d'erreur uniques, ordre des événements ; `check-architecture` vert.

## J2 — Stockage — FAIT (jalon/J2)

- [x] Une base SQLite par profil, migrations versionnées ; schéma en trois familles (D-18) ; table de correspondance des identifiants serveur (D-13).
- [x] Source brute compressée (zstd) et cache d'analyse reconstruisible (D-16, invariant 14) ; FTS5 sur objet, expéditeurs, destinataires, texte.
- [x] Verrou de profil (D-27) ; interface du trousseau : `keyring` en production, faux en mémoire en test ; clé = profil + compte + type (D-10).
- [x] Horloge et générateur d'identifiants injectables (annexe H).
- **Acceptation** : invariants 7–12, 14, 15 (aucun secret en base), 16 (jeter le miroir conserve configuration et intentions), 29 (pagination par curseur) testés ; second verrou refusé.

## J3 — Providers et registre — FAIT (jalon/J3)

- [x] 4 crates providers purs, tests par tables (§6.11) : reconnaissance d'adresses (pièges : majuscules, sous-domaines, adresses invalides), serveurs, rôles de dossiers, traductions, limites.
- [x] Registre dans le cœur : enregistrement, recherche par identifiant, détection avec repli sur generic (§6.6) ; provider introuvable → compte en erreur, données conservées (§7.8).
- [x] Capacités effectives = annonce du serveur moins les exceptions du provider (D-08).
- [x] Assemblage : une ligne par provider.
- **Acceptation** : `mira provider detect <adresse>` correct ; invariants 1–3 testés ; le cœur est testé avec un faux provider défini dans ses tests.

## J4 — Testkit protocoles — FAIT

- [x] Faux serveur IMAP : LOGIN / AUTHENTICATE XOAUTH2, CAPABILITY, LIST (SPECIAL-USE), SELECT / EXAMINE, UIDVALIDITY, UID FETCH / SEARCH / STORE / COPY / MOVE / EXPUNGE, APPEND, CONDSTORE / QRESYNC, IDLE, extensions Gmail (X-GM-MSGID, X-GM-LABELS, X-GM-THRID). Extensions activables une à une ; renumérotation et coupure injectables.
- [x] Faux serveur SMTP (EHLO, AUTH, MAIL, RCPT, DATA, refus injectables) et faux serveur d'autorisation OAuth (code + PKCE, rafraîchissement, révocation).
- **Acceptation** : tests propres au testkit ; profils de capacités : complet, sans QRESYNC, sans MOVE, minimal.

## J5 — Synchronisation — PARTIEL (J5a client IMAP + MIME FAIT ; J5b orchestrateur de synchro À FAIRE)

- [ ] Client IMAP encapsulé (annexe H) ; analyse MIME tolérante (§12.7).
- [ ] Descente par étapes (D-20), fenêtre de 90 jours, « remonter plus loin » en tâche longue (D-21).
- [ ] Stratégie choisie d'après les capacités (D-22) : QRESYNC, ou repli par comparaison ; resynchronisation sur changement d'UIDVALIDITY (§10.5) ; identité du message, doublon plutôt que fusion (P7).
- [ ] IDLE sur une connexion dédiée, travail sur une seconde, relevé périodique en repli, backoff ; une tâche de synchro par compte, plafond de connexions.
- [ ] Le serveur gagne ; les actions locales sont rejouées par-dessus (D-23) ; repère avancé après écriture (invariant 20).
- **Acceptation** : S4, S5, S8, S11, S12, S13 et invariants 20, 21, 23, 24, 25 contre le testkit, avec les 4 profils de capacités ; reprise après coupure au milieu d'une synchro.

## J6 — Messages, boîtes, vues, fils — FAIT

- [ ] Lister une boîte ou une vue (curseur, tri, filtres), en-têtes, corps (« indisponible hors ligne » si absent, S6), source brute, pièces jointes.
- [ ] Vues calculées : boîte unifiée, non lus, suivis (§7.10) ; recherche FTS locale avec filtres.
- [ ] Fils calculés par MIRA (en-têtes de réponse, repli sur l'objet normalisé, indice serveur), recalculables (D-15, invariant 13).
- **Acceptation** : S6, S11 (vue unifiée : un seul message), invariants 12, 13, 27 ; liste paginée < 50 ms sur 100 000 messages (test de performance).

## J7 — Écritures et file d'actions — PARTIEL (J7a SMTP + OAuth FAIT ; dépôt de file idempotent FAIT ; soumission, rejeu, envoi À FAIRE)

- [ ] Intentions → identifiant d'action, clé d'idempotence ; écriture en base avant tentative (invariant 17) ; ordre par compte (invariant 19) ; application locale optimiste ; erreur définitive → effet local annulé et événement (invariant 22) ; erreur passagère → backoff ; regroupement ; annulation (undo).
- [ ] Traduction des actions (archiver, corbeille, déplacer, libellés) par le provider ; D-17 refus.
- [ ] Brouillons, identités ; envoi SMTP, délai d'annulation, envoi planifié, copie dans Envoyés selon le provider ; envoi interrompu → état incertain, jamais de renvoi automatique (invariant 26).
- [ ] Jeton expiré : rafraîchissement silencieux ; en cas d'échec, état « authentification requise » et file conservée (S10).
- **Acceptation** : S7, S8, S9, S10 et invariants 17, 18, 19, 22, 26, 28 ; tests de crash entre les étapes de l'envoi.

## J8 — Façade du contrat d'API — PARTIEL (squelette, verrou, version, 9 opérations, bus d'événements, opérations longues)

- [ ] Toutes les opérations v1 du §12.4, un handler par opération ; lectures locales ; tâches longues (identifiant d'opération, progression, annulation) ; événements numérotés avec reprise et filtres ; contrôle de version à l'ouverture ; aucune fuite de secret.
- [ ] Comptes : préparer, tester, ajouter, modifier, pause, supprimer (S14) ; OAuth PKCE avec écouteur sur la boucle locale ; mot de passe d'application ; export JSON (D-29) ; maintenance.
- **Acceptation** : test de contrat versionné (instantané JSON des opérations, événements et erreurs) ; invariants 15, 27–33 ; S1, S2, S3, S14.

## J9 — CLI — PARTIEL (provider list/detect, --json versionné, codes de sortie)

- [ ] `mira <domaine> <verbe>` pour chaque opération ; `--json` = enveloppe versionnée ; codes de sortie 0 / 10–19 / 84 ; erreurs sur stderr ; `--profile`.
- [ ] Scénarios S1–S14 exécutés comme tests via le CLI en processus contre le testkit.
- **Acceptation** : invariant 6 (couverture CLI = façade, testée par énumération) ; S1–S14 verts.

## J10 — Rendu sûr et durcissement — PARTIEL (assainissement HTML fait)

- [ ] HTML assaini dans le moteur (sans script, contenu distant bloqué, liste des ressources retirées) ; autorisation par message ou expéditeur ; pièces jointes à la demande, sans fichier partiel exposé.
- [ ] proptest sur les invariants 7–12 ; fuzz léger des parseurs ; revue des invariants 1–33 un par un ; performance sur 100 000 messages ; `README.md`, `ARCHITECTURE-IMPLEMENTEE.md`, `GUIDE-RUST-POUR-CPP.md`.
