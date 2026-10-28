# Propositions de modification de la référence

La référence est en lecture seule. Les incohérences et idées relevées pendant l'implémentation sont listées ici.

## PR-01 — Un crate par provider
L'annexe E présente « un crate par provider » comme un écart avec l'architecture, qui « retient un bloc providers regroupant les modules ». L'implémentation garde un crate par provider (décision de départ 34) : c'est le seul moyen de vérifier **mécaniquement** qu'un provider ne dépend ni du cœur ni d'un autre provider (invariant 2). Proposition : préciser en 5.3 que le bloc providers est composé d'un crate par provider.

## PR-02 — Colonne de profil des comptes
Le chapitre 8 met le profil dans le compte, mais un espace de données par profil (D-10) rend la colonne redondante en base. Proposition : préciser que l'identifiant de profil d'un compte est implicite (celui de la base qui le contient), ou l'assumer comme garde-fou de cohérence (choix actuel : colonne `profile_id`, migration v002).

## PR-03 — Envoi interrompu
La décision de départ 9 (jamais de renvoi automatique, état « incertain », vérification dans Envoyés par `Message-ID`) mérite d'entrer dans la référence en remplacement du point ouvert de 10.8, avec l'état `Uncertain` ajouté au cycle de vie d'une action (8.6) et l'événement `SendUncertain` (12.5).

## PR-04 — Clair vers la boucle locale
Ajouter à 15.2/annexe H la règle AG-03 : connexion en clair autorisée uniquement vers `127.0.0.1`/`::1`/`localhost` (bancs d'essai), refusée ailleurs (`tls.plaintext_refused`).

## PR-05 — Page de confirmation OAuth
L'écouteur de redirection (15.2) renvoie une petite page HTML au navigateur : c'est la seule « interface » produite par le moteur. Préciser qu'elle est statique, sans script, et ne relève pas de l'invariant 4.

## PR-06 — Revalidation après renumérotation
10.5 ne dit pas comment garder les anciennes apparitions pendant qu'on recrée les nouvelles avec des UID qui peuvent se chevaucher. Implémentation : UID temporairement négatifs pendant la revalidation (AG-13). Proposition : mentionner cette contrainte d'unicité (boîte, UID).

---

Les propositions PR-07 à PR-17 ont été ajoutées le 2026-10-14, après J9 : elles viennent de ce que l'implémentation et les relectures (`docs/ETAT/REVUE-RELECTEUR.md`, `docs/ETAT/AUDIT-SECURITE.md`) ont révélé. Chacune dit ce que la référence écrit, ce que le code fait, ce qu'il faudrait changer dans la référence, et renvoie aux décisions AG.

## PR-07 — Effacement sûr des données d'un compte (S14, 15.1)
**Contexte.** S14 demande d'effacer les données locales d'un compte et ses secrets. Supprimer des lignes SQLite ne suffit pas : avec les réglages par défaut, des fragments de sujets et d'adresses restent dans les pages libres, dans les segments de l'index FTS5 et dans le journal WAL. Un test de S14 via le CLI l'a montré (AG-32).
**Proposition.** Ajouter à S14 et à 15.1 : (a) la base est ouverte avec `PRAGMA secure_delete=ON` ; (b) à la suppression d'un compte et à l'abandon de son miroir, l'index de recherche est fusionné (`optimize`) puis le journal est tronqué (`wal_checkpoint(TRUNCATE)`) ; (c) les fichiers de cache des pièces jointes du compte sont supprimés avant les lignes de la base ; (d) une suppression interrompue (compte marqué « en suppression ») est achevée au prochain démarrage ; (e) le moteur n'exécute pas `VACUUM` (réécriture complète, espace disque doublé) : les traces d'une base créée avant ces réglages ne sont pas purgées après coup.
**Raison.** La vie privée de l'utilisateur est le but de S14 ; sans (a)–(b) l'« effacement » est cosmétique.
**Renvoi.** AG-32, AG-33, AG-34 ; `C/storage/erasure.rs`, `C/accounts/deletion.rs`. À noter : `maintenance::discard_mirror_and_cache` n'appelle pas encore l'effacement des traces (voir `docs/ARCHITECTURE-IMPLEMENTEE.md` §11).

## PR-08 — Sémantique de `--wait` et clients de courte durée (12.2, 12.3)
**Contexte.** La référence décrit des commandes d'intention qui « rendent la main avec un identifiant d'action » et dont le résultat arrive par événement (12.2.2). Un CLI vit le temps d'une commande : sans attente, le rejeu de fond est tué avec le processus.
**Proposition.** Préciser en 12.8 (CLI) : les opérations longues (corps, pièces jointes, recherche serveur) sont toujours attendues par le CLI (délai `--timeout`, 120 s) ; les commandes d'intention sont attendues seulement avec `--wait` (60 s). Issue de l'attente : action `Failed` → catégorie serveur (16) ; `Uncertain` ou délai dépassé → réseau (13), l'action **reste en file** ; compte à réauthentifier → authentification (15) immédiate, action suggérée `reauthenticate`, l'action reste en file. L'attente ne modifie jamais l'état de l'action.
**Raison.** Sans cette règle, un script ne peut pas distinguer « en file » de « échoué » ; l'implémentation initiale attendait jusqu'au délai (code 13) sur un compte à réauthentifier, ce qui masquait la cause (corrigé en AG-33).
**Renvoi.** AG-24, AG-33 ; `mira-cli/src/commands/waiting/`.

## PR-09 — Envoi à issue inconnue : toute erreur réseau après la connexion est incertaine (10.8, décision 9, invariant 26)
**Contexte.** La référence laisse le cas d'un plantage « entre l'acceptation par le serveur SMTP et l'enregistrement local » OUVERT (10.8) ; PR-03 propose d'y mettre la décision 9. Le rejeu a révélé un cas voisin, plus fréquent : une erreur de transport **pendant** la session SMTP (délai, coupure, `ETIMEDOUT` après `DATA`) ne dit pas si le serveur a accepté le message.
**Proposition.** Ajouter à 10.8 la règle : pour une action d'envoi, seule une erreur prouvant que rien n'est parti (hôte injoignable, connexion TCP impossible) est passagère et rejouable. **Toute autre erreur réseau survenue après l'établissement de la connexion est « issue inconnue »** : le moteur cherche le `Message-ID` de l'envoi dans Envoyés (réussi s'il y est), sinon passe l'action à `Uncertain` (événement `SendUncertain`) et laisse l'utilisateur décider. Le moteur ne renvoie jamais seul. Le `Message-ID` est fixé et mémorisé dans la même transaction que le passage à « en cours », avant tout appel SMTP ; un renvoi décidé par l'utilisateur reçoit un **nouveau** `Message-ID`. Un délai global (120 s) borne la session ; son dépassement après connexion est une perte de connexion (incertaine), jamais un `network.timeout` passager.
**Raison.** Un renvoi sur délai aurait produit des doublons visibles par les destinataires (audit #4, revue #3).
**Renvoi.** AG-20, AG-27, AG-30 ; `C/actions/replay/classify.rs`, `C/send/settle.rs`. Limite : une erreur de catégorie `Tls` en cours d'envoi est classée passagère (à auditer).

## PR-10 — Le plafond de connexions s'applique à toutes les lectures explicites (10.12, invariant 25)
**Contexte.** Le plafond « par compte, fourni par le provider » (10.12) était respecté par la synchro seule ; le rejeu, les téléchargements de corps et de pièces jointes, la recherche sur le serveur et les tests de connexion ouvraient leurs propres connexions (revue #6, audit #6).
**Proposition.** Préciser l'invariant 25 : un **budget de connexions par compte**, tenu par le moteur, est pris avant toute connexion IMAP, quel qu'en soit l'usage ; un test de connexion avant l'ajout du compte utilise un budget propre à l'adresse candidate. Les demandes explicites en attente d'un permis sont bornées (16 par compte) ; au-delà, `server.rate_limited`. L'écoute IDLE n'est établie que si un permis de réserve reste (limite du provider ≥ 3).
**Raison.** Sans partage du budget, une rafale de lectures explicites peut dépasser le plafond du provider et faire bloquer le compte.
**Renvoi.** AG-21, AG-28 ; `C/sync/session/{budget,budgets,lease}.rs`.

## PR-11 — Contenu distant : liste blanche plutôt que liste noire (11.6, 15.4, décision 10)
**Contexte.** 15.4 demande de bloquer les ressources distantes ; une première implémentation retirait les URL « qui ressemblent à du distant ». Elle était contournable (`ht&#9;tps://…`, `\\hôte`, URL relatives, attribut `background`).
**Proposition.** Écrire en 15.4 : l'assainissement du moteur fonctionne en **liste blanche**. Seuls sont conservés, dans les attributs chargés automatiquement, les schémas `cid:` et `data:image/*` (hors SVG) ; l'URL est normalisée avant test (retrait des tabulations, sauts de ligne et caractères de contrôle, espaces d'extrémité, `\` vers `/`) ; toute URL relative est retirée ; `data:` est interdit dans `href`. Les ressources retirées sont listées dans la réponse (les URL relatives retirées ne sont pas comptées). L'autorisation par message ou par expéditeur (décision 10) n'active le chargement qu'à la demande de `GetSafeHtml`, jamais par défaut. Pas de relais d'images en v1.
**Raison.** Une liste noire d'URL est contournable par nature ; une liste blanche échoue du côté sûr.
**Renvoi.** AG-29 ; `C/html/remote.rs`, `C/html/sanitize.rs`. Reste : `sanitize_html` est quadratique en profondeur d'imbrication (voir PR-17).

## PR-12 — Rétention des événements (12.5, 15.1)
**Contexte.** La référence numérote et persiste les événements pour la reprise (invariant 32) mais ne dit pas combien de temps. La table `events` n'était jamais purgée (audit #9) ; l'ancien document d'architecture disait « les 10 000 derniers ».
**Proposition.** Fixer la rétention en 12.5 : **30 jours et 100 000 lignes au plus**, appliquée à l'ouverture du moteur et après chaque passe de synchronisation. La table des opérations terminées est bornée de la même façon (256 conservées).
**Raison.** Sans borne, la base d'un profil grossit sans limite au rythme des événements de progression.
**Renvoi.** AG-28 ; `C/storage/repositories/events/prune.rs`, `C/operations/retention.rs`. Ouvert : le comportement exact d'un abonné dont le numéro `after` a été purgé n'est pas spécifié.

## PR-13 — UTF-7 modifié (noms de boîtes IMAP) : absent de l'implémentation (6.8, 10.2)
**Contexte.** La RFC 3501 §5.1.3 impose l'UTF-7 modifié pour les noms de boîtes non ASCII (`&AOk-` pour « é »). Vérification par recherche dans les sources (`utf7`, `utf-7`, « modified utf ») : **aucune occurrence**. Le client IMAP cite les noms tels quels (`C/imap/protocol/quoting.rs`) ; AG-12 suppose les libellés `X-GM-LABELS` « déjà décodés par le client IMAP », ce qui est faux aujourd'hui. La recherche serveur n'utilise pas non plus `CHARSET UTF-8` (revue #11, jamais traitée).
**Proposition.** Ajouter en 6.8/10.2 que le client IMAP décode et encode l'UTF-7 modifié pour les noms de boîtes et de libellés (et utilise `UTF8=ACCEPT` ou `SEARCH CHARSET UTF-8` avec littéral quand le serveur l'annonce) ; l'identifiant et le nom affiché d'une boîte sont le nom décodé. Tant que ce n'est pas fait, c'est une lacune connue de J10.
**Raison.** Sans cela, les dossiers accentués apparaissent avec leur forme encodée et ne peuvent être créés, renommés ni retrouvés ; les recherches non ASCII sont à la merci du serveur.
**Renvoi.** AG-12, AG-34 ; revue #11 ; `docs/ETAT/PLAN.md` (J10).

## PR-14 — Ordre du préchargement des corps (10.2, D-21)
**Contexte.** 10.2 dit « préchargement des plus récents » sans fixer l'ordre entre boîtes. Pour S4 (« l'utilisateur voit sa boîte de réception le plus tôt possible »), il faut l'ordre.
**Proposition.** Préciser : les boîtes sont descendues dans l'ordre réception, Envoyés, Brouillons, Archive/Tous/Suivis, autres, Corbeille, Spam ; les corps de la **réception** sont préchargés (les 50 plus récents encore « en-tête seul », 10 Mo au plus chacun) **juste après ses en-têtes**, avant ceux des autres boîtes. Seule la réception est préchargée ; un échec serveur sur un message est journalisé et ignoré.
**Raison.** C'est l'ordre implémenté et testé (S4) ; il évite d'attendre l'envoi des en-têtes de toutes les boîtes avant de pouvoir lire le dernier message reçu.
**Renvoi.** AG-28, AG-31 ; `C/sync/download/{order,prefetch}.rs`.

## PR-15 — Hors ligne et pièces jointes (S6, 12.7)
**Contexte.** S6 dit : « Les pièces jointes non téléchargées ne sont pas disponibles » et « le téléchargement [du corps] sera tenté quand la connexion reviendra ».
**Proposition.** Préciser deux points. (1) Un message dont le corps est préchargé conserve sa **source brute complète** en base : ses pièces jointes y sont donc disponibles hors ligne même si l'état de la pièce indique « non téléchargée » ; `SaveAttachment` essaie d'abord le cache local et la source, et ne contacte le serveur que si besoin. La règle « non disponible » de S6 vise les messages en **en-tête seul**. (2) Il n'existe pas de file de téléchargement différé : un corps absent est obtenu par la commande explicite `FetchMessageBody` ; la seule récupération automatique au retour de la connexion est le préchargement des 50 plus récents de la réception à la passe suivante.
**Raison.** Aligner la référence sur le comportement testé et signaler l'écart avec « le téléchargement sera tenté ».
**Renvoi.** AG-31 ; `C/fetching/attachment.rs`, `C/reading/attachments.rs`, `mira-cli/tests/scenario_s06_*`.

## PR-16 — CLI de courte durée : IDLE et OAuth en deux processus (10.6, 12.8, 15.2)
**Contexte.** La référence suppose un moteur durable (écoute IDLE, flux OAuth en cours). Le CLI crée un moteur par commande ; la table des flux OAuth et les résultats de recherche serveur sont en mémoire.
**Proposition.** Préciser en 12.8 : (a) un CLI de courte durée n'entretient aucune écoute IDLE ; S5 est testé au plus près (relevé incrémental, arrivées concurrentes à `sync start`), et une commande `watch` de longue durée est une extension possible ; (b) l'autorisation OAuth se fait en **un seul processus** (`account add-oauth` : URL sur stderr, attente bornée, puis ajout) ; les commandes `oauth-start` puis `oauth-complete` dans deux processus ne sont pas supportées, de même que `auth reauthenticate` suivi de `oauth-complete` ; (c) `search server-result` ne retrouve rien d'un autre processus (code 11) ; (d) le moteur est exclusif par profil (D-27), donc `operation cancel` pendant `extend-window` ou la suppression d'un compte pendant une synchro ne se testent pas en processus séparés ; (e) la synchro de fond n'est activée par le CLI que pour `sync …` et `account add|add-oauth --sync-wait`.
**Raison.** Évite qu'un second client suppose un état partagé que seul un moteur long possède (D-30 : processus séparé reporté).
**Renvoi.** AG-19, AG-26, AG-31, AG-32.

## PR-17 — Bornes de ressources face à un contenu hostile (11, 15.3, 15.4)
**Contexte.** La référence parle de tolérance du MIME (12.7) et de sécurité du rendu, pas de limites de taille ou de temps. La revue et l'audit ont montré qu'un message ou un serveur hostile peut geler le moteur ou saturer la mémoire.
**Proposition.** Ajouter un paragraphe « bornes » en 15.3 : ligne IMAP 1 Mo, littéral et réponse 64 Mo, délai d'inactivité réarmé par bloc ; message analysé ≤ 50 Mo ; préchargement ≤ 10 Mo par message ; HTML converti en texte ≤ 2 Mo ; pièce jointe de brouillon ≤ 25 Mo ; réponse de jetons OAuth ≤ 1 Mo ; VANISHED au-delà d'un million d'UID, ou avec `*`, remplacé par la comparaison complète ; dates serveur bornées 1900–9999 ; recherche serveur limitée à 200 résultats ; « message poison » (échec serveur du préchargement d'un message) journalisé et ignoré. **Lacune à combler** : l'assainissement HTML n'a aucune borne de taille ni de profondeur et est quadratique en profondeur d'imbrication (1 000 niveaux : 68 ms ; 10 000 : 5,3 s ; 100 000 : plus de 10 min) ; il faut plafonner taille et profondeur **avant** `ammonia`.
**Raison.** Chaque borne ferme un déni de service par un seul message ou un serveur malveillant.
**Renvoi.** AG-27, AG-28, AG-29, AG-34 ; `C/imap/protocol/reader.rs`, `C/mime/message.rs`, `C/html/`.

## PR-18 — Dernier numéro d'événement dans l'état du moteur (12.4, 12.6)
**Contexte.** Un client de longue durée (le TUI) veut réagir aux seuls événements postérieurs à son ouverture. Un abonnement sans `after` rejoue tout le journal conservé (jusqu'à 100 000 événements sur 30 jours) ; le contrat ne donne pas le dernier numéro, si bien que le TUI le cherche en lisant `GetEvents` page par page (1 000 par page) au démarrage, sans réagir.
**Proposition.** Ajouter `last_event_seq: Option<EventSeq>` à `Response::Status` (`EngineStatus`), ou une option `from_now` au filtre d'abonnement, pour s'abonner « à partir de maintenant » en une requête.
**Raison.** Évite une lecture linéaire du journal à chaque ouverture d'un client et le risque de rejouer d'anciens événements comme s'ils étaient récents.
**Renvoi.** AG-41 ; `mira-tui/src/app/event_cursor.rs`, `mira-tui/src/app/event_feed.rs`.

## PR-19 — Plus de client en ligne de commande (objectif 6, D-04, 12.8)
**Contexte.** La référence fait du CLI le premier client (D-04) et le moyen de « tout tester sans écran ». Le projet n'a plus qu'un client, l'application Flutter par le pont FFI ; le CLI et le TUI sont retirés (AG-63).
**Proposition.** Remplacer, dans l'objectif 6 et D-04, « utilisable en ligne de commande » par « utilisable sans interface par sa façade (`Engine`) et son contrat sérialisé, testé de bout en bout contre le testkit » ; rendre sans objet l'invariant 6 (correspondance `Request` ↔ commande CLI) et PR-16 ; garder S1–S14 comme scénarios de la façade.
**Raison.** Le contrat reste la seule frontière publique ; un CLI n'apporte plus rien qu'un test de façade ne vérifie, et coûte en maintenance et en ressources de CI.
**Renvoi.** AG-63 ; `mira-core/tests/scenario_s*`.

## PR-20 — Ne plus présenter MIRA comme « offline-first » (1, glossaire)
**Contexte.** L'introduction (ch. 1) et le glossaire présentent MIRA comme un client « offline-first ». Ce n'est pas la raison d'être du projet : MIRA existe parce que les clients mail Linux disponibles étaient trop austères, payants ou enfermés dans un navigateur.
**Proposition.** Au ch. 1, remplacer « Il fonctionne en mode **offline-first** : on peut lire, trier et rédiger sans connexion, et tout se rattrape quand le réseau revient » par une phrase de positionnement (client Linux moderne, soigné, libre et gratuit, natif) ; retirer l'entrée « Offline-first » du glossaire ou la rebaptiser « Miroir local ». Les mécanismes (lecture locale, file d'actions, écritures optimistes, S6–S8) restent inchangés : ce sont des choix d'architecture, pas un argument produit.
**Raison.** Aligner la référence sur l'objectif réel du produit sans toucher aux invariants.
**Renvoi.** `docs/MIRA-architecture-reference.md` ch. 1 et glossaire ; l'ensemble des fichiers hors référence a été aligné.
