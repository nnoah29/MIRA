# Décisions prises par l'agent

Format : **AG-xx — titre** · contexte · options · choix · raison · retour arrière.

## AG-01 — Garde-fous écrits en Python 3 (bibliothèque standard)
- **Contexte** : `tools/check-norms` et `tools/check-architecture` doivent analyser le code Rust et `cargo metadata`.
- **Options** : crate Rust (avec `syn`) dans le workspace, ou script Python sans dépendance.
- **Choix** : Python 3, bibliothèque standard seulement.
- **Raison** : pas de nouvelle dépendance Rust ; l'outil n'est pas du code du moteur et n'est donc pas soumis à ses normes ; `python3` est présent ; un lexer maison suffit (chaînes, caractères, commentaires, accolades).
- **Retour arrière** : réécrire l'outil en crate Rust ; l'interface (code de sortie, format des messages) reste la même.

## AG-02 — Sous-agents lancés en `general-purpose` avec fichier de rôle
- **Contexte** : les rôles écrits dans `.claude/agents/` pendant la session ne sont pas chargés par l'outil Agent (types introuvables).
- **Choix** : lancer `general-purpose` (modèle sonnet pour implémenteurs et testkit, opus pour relecteur et qualité), en demandant de lire d'abord `.claude/agents/<rôle>.md`, et en interdisant explicitement les outils web dans la consigne des rôles qui n'y ont pas droit.
- **Raison** : seul moyen de déléguer sans redémarrer la session. Les définitions restent valables pour une session future.
- **Retour arrière** : relancer Claude Code ; les types `implementeur-rust`, etc. seront alors disponibles.

## AG-03 — Connexion en clair autorisée seulement vers la boucle locale
- **Contexte** : le testkit écoute en clair sur 127.0.0.1 ; la production exige TLS.
- **Choix** : `Security::Plaintext` existe dans le vocabulaire, mais le cœur refuse (`tls.plaintext_refused`) toute connexion en clair vers un hôte qui n'est pas `127.0.0.1`, `::1` ou `localhost`.
- **Raison** : tester le vrai client IMAP/SMTP contre de faux serveurs sans certificats, sans jamais affaiblir la production.
- **Retour arrière** : ajouter des certificats de test (crate `rcgen`) et supprimer la variante.

## AG-04 — Contrat d'API : énumérations `Request`/`Response` et un handler par opération
- **Contexte** : §11.8 du brief (un handler par opération, routage par `match` exhaustif) et D-30 (même contrat en processus ou séparé).
- **Choix** : `Engine::execute(Request) -> Result<Response, MiraError>` ; chaque variante est routée vers son propre fichier de handler. Les énumérations sont sérialisables (même forme pour CLI `--json`, Flutter, ou un futur transport).
- **Raison** : contrat testable par instantané JSON, invariant 6 vérifiable par énumération.
- **Retour arrière** : ajouter des méthodes typées au-dessus ; les handlers restent.

## AG-05 — Stockage : `Database` + transaction `Tx`, dépôts en blocs `impl Tx`
- **Contexte** : aucune fuite de `rusqlite` hors de `storage/` (annexe H) ; écritures atomiques composées par la synchro.
- **Choix** : `Database { Mutex<Connection> }`, `write(|tx| …)` / `read(|tx| …)` ; un fichier par opération ou groupe d'opérations d'une entité. Accès synchrone depuis le code async (requêtes courtes, un seul écrivain).
- **Raison** : simplicité et lisibilité pour un développeur C++ ; un seul écrivain cohérent avec le verrou de profil (D-27).
- **Retour arrière** : déplacer les appels dans `spawn_blocking` ou un fil dédié derrière la même API.

## AG-06 — Le client IMAP réel est testé contre le faux serveur, sans maquette du client
- **Contexte** : P10 (testable sans réseau) et annexe H (remplaçable par un faux).
- **Choix** : le faux est côté serveur (testkit sur 127.0.0.1). Le client IMAP/SMTP du moteur est un type concret encapsulé, testé de bout en bout contre le testkit.
- **Raison** : on teste le vrai code de protocole ; pas de trait asynchrone dynamique compliqué.
- **Retour arrière** : extraire un trait `MailBackend` (6.10) le jour où un second backend arrive.

## AG-07 — `cargo deny` sans la vérification des avis de sécurité
- **Contexte** : `cargo deny check advisories` télécharge la base d'avis depuis GitHub (réseau hors crates.io, règle 2).
- **Choix** : `./ci.sh` exécute `cargo deny check bans licenses sources` seulement.
- **Retour arrière** : ajouter `advisories` quand le réseau est autorisé.

## AG-08 — Écarts acceptés du vocabulaire (rapport J1)
- `MiraError.context` est un `Box<ErrorContext>` (clippy `result_large_err`) ; JSON inchangé.
- Marqueurs d'identifiants nommés `AccountMarker`… (évite le conflit avec `ActionKind`).
- `Timestamp` et `Id<K>` sérialisés de façon transparente (entier, chaîne).
- `EmailAddress` désérialisée avec validation ; espaces refusés dans la partie locale.
- `EventKind::category()` et `account()` en `#[rustfmt::skip]` pour tenir en 30 lignes (match exhaustif de 44 variantes).
- `EventFilter` : un filtre par compte exclut les événements sans compte ; `after` strict.
- **Temporaire** : `storage::json_columns` porte `#[allow(dead_code)]` jusqu'à son premier usage par les dépôts J2 (à retirer à la fusion de J2).

## AG-09 — Comportements des faux SMTP et OAuth (rapport J4 partie B)
- SMTP : `MAIL FROM` avant `AUTH` → `530` (comme un serveur de soumission réel) ; `fail_next` est une file FIFO ; une panne en fin de DATA n'enregistre pas le message ; `drop_after_data_once` enregistre puis coupe sans répondre.
- OAuth : `approve`/`deny` refusent toute requête invalide et tout `redirect_uri` qui n'est pas en boucle locale ; code à usage unique consommé même si le vérificateur est faux ; le rafraîchissement ne renouvelle pas le jeton de rafraîchissement ; jetons générés par compteur, jamais journalisés.
- Modules `oauth::pkce` et `oauth::form` publics (fonctions pures testées).

## AG-10 — Comportements du faux IMAP (rapport J4 partie A)
- Un seul commit pour tout le serveur (niveaux interdépendants).
- `connection_count()` = total des connexions acceptées ; `active_connections()` = connexions ouvertes (à utiliser pour l'invariant 25).
- QRESYNC exige `ENABLE QRESYNC` avant SELECT (sinon `BAD`), comme un serveur réel ; MODSEQ n'apparaît dans les FETCH qu'une fois CONDSTORE activé.
- `renumber` redémarre les UID à 1 et change UIDVALIDITY ; `deliver_dated` permet des dates internes contrôlées (fenêtre de 90 jours).
- `X-GM-LABELS` : noms des boîtes contenant le message (`INBOX` → `\Inbox`, boîtes `\All` exclues).
- Non supportés : BODYSTRUCTURE, sections numérotées, LSUB, UNSELECT, NAMESPACE, STARTTLS, X-GM-RAW.

## AG-11 — Écarts acceptés de J2a
- Migration `v002` : colonne `accounts.profile_id` (le schéma v1 l'avait oubliée) et index unique partiel « une seule identité par défaut par compte ».
- `keyring` 4.2 : la feature par défaut `v1` fournit déjà SecretService (zbus), aucune dépendance système.
- Clé d'idempotence réutilisée pour une **autre** commande → `invalid_input.idempotency_conflict` ; même commande → même action.
- File d'actions : `server_targets_json` et `undo_json` pas encore remplis (`PendingAction` n'a pas ces champs) → à ajouter en J7.
- Profil : nom vide, `.`, `..` ou contenant `/`, `\`, NUL refusé (traversée de répertoire).
- Dépôt d'événements dans `storage/repositories/events/` ; `src/events/` reste pour le bus (J8).

## AG-12 — Écarts acceptés de J3
- CLI : `run(args, &mut CliIo) -> i32` (au lieu de `CliDeps`/`CliOutcome`) ; l'injection des dépendances du moteur viendra en J9.
- Enveloppe JSON : clés dans l'ordre alphabétique de `serde_json` (`contract_version, data|error, ok`) — sans effet pour un lecteur JSON.
- Stratégie `Qresync` dès que QRESYNC est annoncé (il implique CONDSTORE) ; `COMPRESS` = `COMPRESS=DEFLATE`.
- `X-GM-LABELS` : les noms sont supposés déjà décodés (UTF-7 modifié) par le client IMAP.
- Registre : pas d'élimination des doublons d'identifiant ; `find` renvoie le premier.
