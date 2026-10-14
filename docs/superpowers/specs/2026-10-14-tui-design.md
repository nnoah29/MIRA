# TUI de MIRA — spécification de conception

Date : 2026-10-14 · Statut : **en attente de relecture** · Branche prévue : `feat/E-01-tui` (depuis `feat/E-01-engine-overnight`)

## 1. Objectif

Un client en terminal complet pour MIRA, posé sur le moteur existant. Il ne remplace pas le CLI `mira-cli`, qui reste inchangé. Le TUI est un client de plus du contrat d'API (D-04, D-26, principe P1) : il n'embarque aucune logique métier, seulement de la présentation.

**Cible** : tout ce que fait `mira` aujourd'hui (lecture, tri, rédaction, envoi, comptes, paramètres, maintenance), livré en trois étapes, chacune utilisable et testée avant la suivante :

1. **Lecture et tri** : comptes, boîtes, liste de messages, lecteur (texte), fils, recherche, drapeaux, archiver, supprimer, déplacer, état de la synchro.
2. **Rédaction** : éditeur, répondre, répondre à tous, transférer, brouillons, file d'actions, suivi de l'envoi.
3. **Comptes et paramètres** : ajout de compte IMAP et OAuth, identités, maintenance, journal d'événements.

**Hors périmètre** : interprétation de HTML (le moteur fournit le HTML nettoyé, le TUI l'affiche en texte), plugins, thèmes utilisateur, modification du moteur ou de `mira-cli`.

## 2. Contraintes

- Règles absolues de `CLAUDE.md` : pas d'`unsafe`, pas d'`unwrap`/`expect`/`panic!` hors tests, aucun secret dans l'affichage, les journaux ou les erreurs, tests uniquement contre le testkit sur `127.0.0.1`, jamais de compte réel.
- Normes de `tools/check-norms` : fonction de 30 lignes au plus, 6 fonctions avec corps par fichier (tests compris, dans des fichiers séparés), aucun commentaire dans un corps de fonction, une ligne `///` en français avant chaque `fn`, pas de `utils.rs`/`helpers.rs`/`common.rs`/`misc.rs`, `match` exhaustif, un handler par intention.
- Frontière : le TUI ne dépend que de `mira-assembly` (façade, `production_config`, types du contrat). Il ne nomme aucun fournisseur.
- Nouvelles dépendances : `ratatui` et `crossterm` (à vérifier avec `cargo deny`). Toolchain 1.99.0, édition 2024.
- Le terminal est toujours restauré à la sortie, y compris sur erreur ou signal d'interruption.

## 3. Approche retenue

**Client direct sur la façade `Engine`** : un seul `Engine` ouvert pour la durée de la session, synchro de fond activée, abonnement au bus d'événements. Chaque requête passe par `Engine::execute(Request)`.

Écartée : piloter `mira --json` en sous-processus. Le verrou de profil impose un moteur par commande, d'où ni événements, ni synchro continue, ni latence acceptable.

## 4. Architecture

Flux à sens unique en trois couches :

- **`state`** : état de l'application en données pures, sans I/O.
- **`update`** : `(état, message) → (état, commandes)`. Un handler par intention, routage par `match` exhaustif. Les messages viennent du clavier, d'une réponse du moteur ou d'un événement du moteur.
- **`view`** : `état → widgets ratatui`, sans effet de bord.

**Pont asynchrone** : une boucle unique lit le clavier et les événements du moteur, applique `update`, redessine, puis lance les commandes. Les appels `Engine::execute` tournent dans des tâches `tokio`, et leur réponse revient dans la boucle sous forme de message. L'interface ne se fige donc jamais pendant une synchro ou un envoi.

**Isolation** : `update` et `view` ne touchent ni au terminal ni au moteur. Seuls `app/` et `commands/` font des entrées-sorties.

### 4.1 Organisation des fichiers

Crate `engine/crates/mira-tui` (binaire `mira-tui`, bibliothèque `mira_tui`), membre du workspace via `crates/*`.

```
src/
  main.rs  lib.rs
  app/        boucle, pont asynchrone, restauration du terminal
  state/      accounts mailboxes messages reader search compose queue settings focus status
  update/     keyboard/  engine_response/  engine_event/
  commands/   description des appels moteur (Request à lancer)
  view/       layout  sidebar/  message_list/  reader/  compose/  queue/  settings/  statusbar/  popups/
  input/      touches → messages, par contexte
  theme.rs    couleurs et styles
tests/        un fichier par comportement
```

### 4.2 Mise en page

Trois colonnes (comptes et boîtes, liste de messages, lecture), plus une barre d'état en bas (synchro en cours, taille de la file d'actions, dernière erreur, rappel de l'aide).

- **Terminal de moins de 100 colonnes** : un seul panneau à la fois, `Entrée` pour descendre, `Échap` pour remonter.
- **Fenêtres modales** : rédaction, ajout de compte, confirmation, aide.
- **Redimensionnement** : pris en compte à chaque tour de boucle, sans état caché.

### 4.3 Clavier

Raccourcis proches de mutt et vim, flèches en secours, pied de page contextuel qui affiche les touches valables.

| Contexte | Touches |
|---|---|
| Global | `Tab` / `Maj+Tab` changer de panneau · `?` aide · `/` recherche · `:` palette de commandes · `q` quitter · `Ctrl+r` forcer la synchro |
| Navigation | `j`/`k` ou ↓/↑ · `g`/`G` début/fin · `Entrée` ouvrir · `Échap` retour |
| Message | `r` répondre · `a` répondre à tous · `f` transférer · `e` archiver · `d` supprimer · `s` étoile · `u` lu/non lu · `m` déplacer · `z` défaire |
| Rédaction | `Ctrl+s` brouillon · `Ctrl+Entrée` envoyer · `Échap` fermer |
| Comptes | `n` ajouter · `E` modifier · `X` supprimer (avec confirmation) |

Les suppressions et les envois demandent confirmation. Les actions réversibles passent par la file d'actions du moteur et se défont avec `z`. La palette `:` donne accès aux opérations rares (maintenance, identités, événements).

## 5. Gestion des données et des erreurs

- **Listes longues** : défilement par fenêtre sur le contrat paginé du moteur, pour tenir D-32 (liste de 100 000 messages sans charger tout en mémoire).
- **Erreurs du moteur** : `MiraError` s'affiche dans la barre d'état, détaillée dans une fenêtre sur demande. Le message est déjà expurgé de tout secret par le moteur, et le TUI ne l'enrichit pas.
- **Saisie de secrets** : mot de passe masqué, jamais conservé dans l'état au-delà de l'envoi de la requête, jamais écrit dans les journaux.
- **Hors ligne** : l'écran reste utilisable (offline-first). La barre d'état indique l'état de la connexion et les actions en attente dans la file.
- **Fermeture** : arrêt propre du moteur (`shutdown_within`) avant de rendre le terminal.

## 6. Tests

- `update` : tests purs sur les transitions d'état, sans terminal.
- `view` : `ratatui::backend::TestBackend`, comparaison du contenu de l'écran à des attendus.
- Bout en bout : scénarios contre le testkit sur `127.0.0.1` (synchro, lecture, archivage, envoi, ajout de compte), sans vrai compte.
- Un test d'absence de secret dans l'écran, les journaux et les erreurs.
- Restauration du terminal sur sortie normale et sur erreur.
- `./ci.sh` doit rester vert à chaque étape (fmt, clippy `-D warnings`, `check-norms`, `check-architecture`, `cargo deny`).

## 7. Étapes de livraison

Chaque étape donne un TUI utilisable, un commit par fonctionnalité (`feat(tui): …`) et une revue indépendante avant clôture.

0. **Squelette** : crate, boucle, restauration du terminal, ouverture du moteur, écran vide testé.
1. **Lecture et tri** (voir §1).
2. **Rédaction** (voir §1).
3. **Comptes et paramètres** (voir §1).

## 8. Questions ouvertes (à trancher avant ou pendant le plan)

- Le nom du binaire (`mira-tui` retenu par défaut).
- Besoin d'un fichier de configuration pour les raccourcis (non prévu en v1).
- Si `ratatui` ou `crossterm` posent problème à `cargo deny`, choix d'une autre version ou d'une alternative.
