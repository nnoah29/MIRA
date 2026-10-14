# Journal

- **01:08 (0 h 00)** — GO reçu. Branche `feat/E-01-engine-overnight` créée depuis `dev`. Dossier vide `engine_/` supprimé (décision 35). Installation de cargo-nextest et cargo-deny lancée en arrière-plan. Début de J0.
- **01:14 (0 h 06)** — J0 FAIT et tagué `jalon/J0` (5 commits) : garde-fous `check-norms` et `check-architecture` avec auto-tests, `check-forbidden-patterns`, `ci.sh`, `Makefile`, workspace de 9 crates, 8 rôles d'agents. cargo-deny installé ; cargo-nextest en échec (BLOQUÉ-01, contourné par `cargo test`).
- **01:16 (0 h 07)** — Spécifications J1 (vocabulaire) et J4 (testkit) écrites. Vague V1 lancée : 3 sous-agents (J1 vocabulaire, J4 faux IMAP, J4 faux SMTP+OAuth) en worktrees. Types d'agents personnalisés non chargés → `general-purpose` (AG-02).
- **01:22 (0 h 13)** — Pendant V1 : `docs/specs/CORE-ARCHITECTURE.md` (modules, ports, schéma), `J2-stockage.md`, `J3-providers.md`, migration SQL v1. Décisions AG-03 à AG-07.
- **01:31 (0 h 22)** — J1 fusionné (5 commits, 32 tests ; écarts AG-08). Socle stockage commité (Database, Tx, migration v1, introspection). J4 partie B fusionnée (faux SMTP + faux OAuth PKCE, 49 tests) ; correctif : `Debug` expurgé de `Credentials` (fuite potentielle signalée par l'agent). Lancés : J2a (ports, profil, dépôts config/intentions/événements) et J3 (providers, registre, capacités, assemblage, CLI provider). Faux IMAP toujours en cours.
- **01:42 (0 h 33)** — Les 3 sous-agents (faux IMAP, J2a, J3) ont été interrompus par une limite d'usage de l'API (HTTP 429), réinitialisée à 01:40. État : J3 = 4 providers commités ; J2a = ports, profil, comptes/identités commités ; faux IMAP = travail non commité. Les trois agents sont relancés depuis leur historique (SendMessage), consigne de commiter au plus tôt.
- **01:44 (0 h 35)** — Faux IMAP fusionné (1 commit, 54 tests, tout le périmètre des priorités 1 à 4 + SEARCH HEADER). CI vert (296 fichiers conformes). J4 FAIT. Lancé : J2b (miroir et recherche), basé sur les commits de J2a pour les comptes.
- **01:46 (0 h 37)** — J2a fusionné (6 commits : ports, profil + verrou D-27, comptes/identités/autorisations, brouillons, file d'actions idempotente, événements numérotés ; invariants 11, 15, 17, 18, 19 testés). CI vert (343 fichiers). Lancé : J5a (MIME + client IMAP). En cours : J2b, J3.
- **01:48 (0 h 39)** — J3 fusionné et tagué (9 commits : 4 providers, registre, rôles, capacités, assemblage, CLI `provider list/detect` avec `--json` versionné). Vérifié : `mira provider detect alice@gmail.com` → gmail. CI vert (408 fichiers). Spec J7a (SMTP + OAuth) écrite.
- **01:58 (0 h 49)** — J2b fusionné (6 commits, 41 tests : boîtes, messages, apparitions, correspondance D-13, source brute zstd, pièces jointes, fils, FTS5 échappé, miroir jetable). J2 FAIT, tagué. CI vert (459 fichiers). Lancement de J6.
- **02:07 (0 h 59)** — Travail direct de l'agent principal pendant J5a/J6/J7a : garde-fous A8 (encapsulation, invariant 5) et A9 (pas d'écran, invariant 4) ; `tools/update-invariants-table` ; bus d'événements numérotés avec abonnements reprenables (invariant 32) ; registre des opérations longues (annulation coopérative) ; module `html/` (assainissement, ressources distantes bloquées et listées) ; squelette de la façade `Engine` (ouverture, verrou D-27, version du contrat, routage par domaine, 7 opérations). 5 commits, CI vert.
- **02:53 (1 h 44)** — Reprise après arrêt de la session principale : fusion J7a terminée (SMTP, composition MIME, OAuth PKCE, coffre d'identifiants, 62 tests) ; faux positif A9 (« fenêtre » dans la page de confirmation OAuth) corrigé par reformulation. Relecteur lancé, J5a et J6 relancés.
- **02:55 → 06:40** — Limite d'usage : tous les agents arrêtés (BLOQUÉ-03).
- **06:42 (5 h 33)** — Reprise : J5a et J6 relancés avec budget temps ; J7b (actions locales, brouillons) lancé. Périmètre réduit pour tenir l'arrêt doux de 09:08.
- **08:20 (7 h 11)** — Nouvelle reprise après redémarrage de session. Les agents J5a, J6 et J7b s'étaient arrêtés ; J7b n'avait rien commité (abandonné). Handlers comptes/identités/auth de la façade inachevés mis en `git stash` (ne compilaient pas).
- **08:22** — J5a fusionné (client IMAP de haut niveau, MIME tolérant). J6 fusionné (lectures, vues standard, recherche filtrée, fils, perf 20 000 messages) ; marqueurs de conflit commités par la branche J6 corrigés ; garde-fou ajouté. CI vert : 478 tests.
- **08:25 (7 h 16)** — Documents de fin : README, ARCHITECTURE-IMPLEMENTEE, PROPOSITIONS (PR-02 à PR-06), PLAN, TESTS, RAPPORT-FINAL. Arrêt doux.
- **08:44** — Nouvelle consigne de l'utilisateur : continuer tous les jalons jusqu'à la fin, rigoureusement, sans limite horaire (CLAUDE.md mis à jour). Restes intégrés : déclarations `labels`/`idle` du client IMAP (non compilées jusque-là) + tests IDLE. Worktrees nettoyés. Stash des handlers comptes → branche `feat/E-01-j8-accounts` (commit WIP).
- **08:47** — Vague de reprise lancée : J5b (orchestrateur de synchro), J7b (actions locales + brouillons), J8a (comptes, identités, authentification de la façade, S1/S2/S3/S14).
- **~09:00 → 11:40** — Troisième limite d'usage de l'API : les agents J5b, J7b, J8a se sont arrêtés (J5b : 2 commits, J8a : 1 commit, J7b : non commité).
- **12:03** — Reprise des trois agents depuis leur historique.
- **12:25** — Disque /home à 99 % : worktree J8a supprimé, `target/` nettoyé, profils allégés (AG-15) → 69 Go libres. J8a fusionné (comptes, identités, OAuth dans la façade ; S1, S2, S3, S14, invariants 11 et 15). CI vert : 518 tests.
- **12:32** — J7b fusionné (file d'actions locale : soumission, effets locaux, annulation, relance, rebase ; brouillons et envoi planifié ; ~60 tests). CI vert : 606 tests. Lancement de J7c (rejeu serveur, envoi, reprise sur plantage).
- **12:37** — **Arrêt demandé par l'utilisateur** (« stop »), reprise prévue la semaine prochaine. Agents J5b, J7c, J9a stoppés. Branche de travail propre, CI verte (606 tests), dernier commit avant ce journal : 47d3b4a.

## Pour reprendre (semaine prochaine)

1. Relire CLAUDE.md, PLAN.md, la fin de ce journal.
2. Travail non fusionné, conservé dans des worktrees (à reprendre ou relancer) :
   - .claude/worktrees/agent-a1884904a70cc440a feat/E-01-j5b-sync commits=6 non_commités=3
   - .claude/worktrees/agent-a70d10c1eb5a98f25 feat/E-01-j7c-replay-send commits=0 non_commités=0
   - .claude/worktrees/agent-ac3a8e43602d93a7d feat/E-01-j9a-cli commits=0 non_commités=32
3. Ordre conseillé : J5b (synchronisation, spec docs/specs/J5b-synchronisation.md) → J7c (rejeu, envoi, reprise sur plantage) → J9a (CLI sur la façade, invariant 6) → J8 restant (lectures, actions, synchro, maintenance dans la façade, test de contrat JSON) → S1–S14 de bout en bout → revue indépendante (relecteur, qualite-securite) → J10.
4. Disque : surveiller /home (profils allégés AG-15 ; supprimer les `target/` des worktrees terminés).

## Reprise (2026-10-14)

- **07:48 UTC** — Reprise dans un conteneur distant neuf (`/home/user/MIRA`, clone de `origin/feat/E-01-engine-overnight`, dernier commit 363f184). Disque 31 Go libres. Toolchain 1.99.0 installée par rustup.
- **Constat** : les trois worktrees de la machine locale (J5b 6 commits, J9a 32 fichiers, J7c vide) ne sont **pas** dans ce clone (jamais poussés) ; seules la branche de travail et ses 606 tests existent. J5b, J9a et J7c sont donc **à refaire depuis les specs**, dans l'ordre du §6 de PASSATION.md. Consigné en AG-17.
