# Design — refus `busy` à la poignée de main du démon (SEC-04, point c)

**Statut : accepté par l'utilisateur et implémenté le 2026-11-26 (branche `feat/SEC-06-busy`).** Constat d'origine : `RESTE-A-FAIRE.md` §3.5, « Poignée de main du démon ».

## Problème
`session::run` répond `hello_ok` (`greeting::greet` puis `hello::handshake`), *puis* prend une place dans `client_slots` (`MAX_CLIENTS = 3`, `server/context.rs`). Un 4ᵉ client authentifié reçoit donc `hello_ok`, se croit connecté, puis la connexion est fermée sans explication : côté Dart, `ConnectionLost` au premier appel, message trompeur.

## Proposition
1. **Prendre la place avant de répondre.** Dans `hello::answer`, après le verdict positif (jeton d'abord, donc un inconnu n'apprend jamais que le démon est plein), `try_acquire_owned` sur `client_slots`. Succès : `hello_ok`, la place est portée par la session (comme aujourd'hui). Échec : `hello_err` puis fermeture.
2. **Nouvelle raison fixe `busy`** dans `HelloRefusal` (`mira-protocol/src/handshake.rs`), sérialisée `"busy"`. Toujours aucun texte libre.
3. **Versionnement** : variante additive → `PROTOCOL_VERSION` 1.1 → **1.2** (mineure). `CONTRACT_VERSION` (contrat d'API) **inchangé**.
4. **Client Dart** (`host_errors.dart`, `refusalError`) : `'busy'` → `EngineException.local('host.busy', …)` « Trop d'applications MIRA sont déjà connectées au moteur en arrière-plan : fermez-en une puis relancez ». Pas de relance automatique (un nouvel essai retomberait sur le même refus). Chaîne ajoutée dans les 5 langues (`tools/i18n-audit`).
5. **Compatibilité** : un client 1.1 qui reçoit `busy` tombe dans la branche `_ => unavailableError()` déjà existante (message générique, pas de plantage) ; un démon 1.1 ne l'émet jamais. Seul client existant : l'app Flutter, mise à jour avec le démon.

## Tests à écrire d'abord (vus rouges)
- `session_hello` (Rust) : 3 clients identifiés tiennent, le 4ᵉ reçoit `hello_err busy` et est fermé ; une fois un client parti, un nouveau est accepté ; un 4ᵉ client avec mauvais jeton reçoit `auth`, **pas** `busy`.
- Vecteur doré `mira-protocol/tests/golden/frames.json` : `hello_err` `busy`, vérifié côté Rust et Dart.
- Dart : `refusalError(HandshakeRefused('busy'))` → code `host.busy` ; connexion contre le faux démon (`fake_daemon.dart`) qui répond `busy`.
- Docs : `PROTOCOLE-DEMON.md` (table des trames, règle 3 de la poignée de main), `CONSTATS.md`.

## Alternatives écartées
- **Statu quo documenté** : simple, mais l'utilisateur voit une déconnexion inexpliquée.
- **Évincer le plus ancien client identifié** (comme pour les connexions en attente) : un processus local muni du jeton pourrait chasser l'app réelle ; une connexion identifiée a une valeur que n'a pas une connexion muette.

## Retour arrière
Retirer la variante et revenir à la prise de place après `hello_ok` ; aucune donnée persistée.
