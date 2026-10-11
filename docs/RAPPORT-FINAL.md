# Rapport final — session de nuit du moteur MIRA

## Durée

- Début (GO) : 2026-10-11 01:08:54. Fin : 08:25 (arrêt doux, avant 09:08 et la limite de 09:30). Écoulé : 7 h 16.
- **Temps de travail effectif : ~3 h 35.** Environ 3 h 50 ont été perdues : arrêt de la session principale vers 02:10, puis limite d'usage de l'API (HTTP 429) jusqu'à 06:40, puis un second redémarrage de session vers 08:20 (BLOQUÉ-02 et BLOQUÉ-03).
- 83 commits sur `feat/E-01-engine-overnight` (dont 12 fusions de branches d'agents, sans squash), 13 sous-agents lancés (dont 3 interrompus avant la fin). Tags locaux : `jalon/J0`, `J1`, `J2`, `J3`, `J4`, `J6`. Aucun push, rien fusionné dans `dev` ni `main`.

## Résultat de `./ci.sh` (dernier commit)

Vert : 478 tests réussis, 0 échec, 1 ignoré (test manuel du vrai trousseau, prévu) ; clippy `-D warnings` ; fmt ; cargo-deny ; `check-norms` (688 fichiers, 0 violation) ; `check-architecture` (A0–A9, 0 violation) ; motifs interdits.

## Jalons

| Jalon | Statut | Contenu |
|---|---|---|
| J0 Squelette | FAIT | workspace 9 crates, toolchain épinglée, garde-fous auto-testés, `ci.sh`, `make check`, 8 rôles d'agents |
| J1 Vocabulaire | FAIT | identifiants opaques, entités D-19, erreurs à code stable, 44 événements numérotés, interface Provider |
| J2 Stockage | FAIT | SQLite par profil (4 migrations), trois familles, correspondance D-13, source zstd, FTS5 échappé, file d'actions idempotente, événements, ports (horloge, ids, trousseau keyring/mémoire), profil + verrou D-27 |
| J3 Providers | FAIT | 4 providers purs, registre avec repli, rôles (SPECIAL-USE d'abord), capacités effectives et choix de stratégie |
| J4 Testkit | FAIT | faux IMAP (CONDSTORE/QRESYNC, IDLE, MOVE, UIDPLUS, libellés, pannes), faux SMTP (pannes, coupure après DATA), faux OAuth2 PKCE |
| J5 Synchro | **PARTIEL** | J5a fait (client IMAP de haut niveau, MIME tolérant avec proptest) ; **J5b (orchestrateur : descente, stratégies, renumérotation, IDLE, superviseur) non fait** |
| J6 Lectures, vues, fils | FAIT | listes par boîte et vues standard, corps hors ligne (S6), recherche filtrée, fils D-15, performance 20 000 messages |
| J7 Écritures | **PARTIEL** | SMTP + composition MIME + OAuth PKCE + coffre d'identifiants faits ; **soumission/effets locaux, rejeu, envoi et reprise d'envoi non faits** |
| J8 Façade | **PARTIEL** | `Engine::open` (version du contrat, verrou), routage par domaine, 9 opérations, abonnements, opérations longues ; ~55 opérations restantes |
| J9 CLI | **PARTIEL** | `mira provider list/detect`, `--json` versionné, codes de sortie |
| J10 Durcissement | **PARTIEL** | assainissement HTML (ressources distantes bloquées et listées) |

Invariants et scénarios avec test nommé : 24 sur 47 (`docs/ETAT/INVARIANTS.md`). Manquants : invariants 6, 10, 20–26, 28, 30 et scénarios S1–S5, S7–S10, S12–S14, qui dépendent tous de J5b, J7 et J8.

**Les critères de « terminé » du brief ne sont donc pas atteints** : J5, J7, J8 et J9 sont partiels.

## Travail laissé en cours (rien n'est perdu)

- `git stash list` : handlers de la façade pour comptes, identités et mot de passe d'application (types du contrat écrits, handlers non écrits).
- Worktrees d'agents non nettoyés, avec du travail non commité : `.claude/worktrees/agent-ad41150dde994ad67` (J5a : tests des libellés), `agent-abd29737e1914ae45` (J6 : petites modifications).

## Décisions prises seul

`docs/DECISIONS-AGENT.md` (AG-01 à AG-13), notamment AG-03 (clair seulement vers la boucle locale), AG-04 (contrat en énumérations, un handler par opération), AG-05 (stockage `Tx`), AG-06 (vrais clients testés contre de faux serveurs). Propositions pour la référence : `docs/PROPOSITIONS-REFERENCE.md` (PR-01 à PR-06).

## Comment lancer

```bash
./ci.sh                                                     # tout vérifier
cd engine && cargo run -q -p mira-cli -- provider detect alice@gmail.com
cd engine && cargo run -q -p mira-cli -- --json provider list
cd engine && cargo test -p mira-core --test reading_performance   # mesure sur 20 000 messages
```

## Risques résiduels

- Les valeurs des providers (serveurs, adresses OAuth, plafonds, copie dans Envoyés) viennent du §6.7, **non vérifiées** en ligne.
- Les chemins TLS réels (993/465/587 avec certificats) ne sont pas testés : seul le clair vers 127.0.0.1 l'est.
- La distinction SMTP « injoignable » / « coupure » repose sur les libellés d'erreur de `lettre`.
- Les fichiers de cache des pièces jointes ne sont pas supprimés quand on jette le miroir (AG-13).
- Le code n'a pas encore eu de revue indépendante complète : le relecteur a été interrompu par la limite d'usage.

## Les 5 prochaines actions recommandées

1. **J5b — orchestrateur de synchronisation** (`docs/specs/J5b-synchronisation.md`) : toutes les briques existent (client IMAP, capacités, dépôts, fils, bus).
2. **J7 — file d'actions et envoi** (`docs/specs/J6-J7-messages-actions.md`, J7) : soumission avec effet local, rejeu, envoi avec état « incertain » (invariant 26).
3. **J8 — compléter la façade** : reprendre `git stash pop` (comptes, identités), puis `AddAccount` avec test de connexion, OAuth, actions, synchro, maintenance ; test de contrat par instantané JSON.
4. **J9 — CLI complet** et scénarios S1–S14 de bout en bout contre le testkit.
5. **Revue indépendante** (agent `relecteur` puis `qualite-securite`) du code fusionné, en priorité `auth/`, `smtp/` et les secrets ; puis nettoyage des worktrees restants (`git worktree remove`).
