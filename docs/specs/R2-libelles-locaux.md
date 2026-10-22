# R2 — Libellés locaux dans le moteur

Déplace les libellés locaux (aujourd'hui dans les préférences de l'app) dans le moteur, pour que les règles (R1) puissent les poser, les retirer et les tester.
Référence : §12 (tout ce que l'app fait existe d'abord dans le moteur, testable sans écran), D-14 (les drapeaux visibles sont au niveau du message ; les libellés locaux sont au niveau de la conversation, voir §1), invariant 13 (les fils sont dérivés et peuvent être recalculés), invariant 18 (idempotence).

## 1. Décisions validées

| Décision | Choix |
|---|---|
| Où | Dans le moteur (base SQLite du profil). L'app n'est plus que l'affichage. |
| Granularité | **Par conversation (fil)** : un libellé posé sur une conversation est porté par tous ses mails, y compris ceux qui arriveront. |
| Portée | Un libellé est commun à tous les comptes. Chaque conversation (de n'importe quel compte) peut le porter. |
| Règles | Actions « Mettre un libellé local » et « Retirer un libellé local », condition « A le libellé local ». Utilisables avec « Tous les comptes ». |
| Reprise | Au premier lancement après la mise à jour, l'app envoie au moteur ses libellés et leurs poses, puis efface sa copie. |
| Hors moteur | Sans moteur (mode démonstration), l'app garde son fonctionnement actuel par préférences. |

Hors périmètre : synchronisation des libellés locaux entre appareils, libellés sur les boîtes du serveur, héritage entre conversations fusionnées (voir §4).

## 2. Modèle (`mira-vocabulary`)

- `LocalLabelId` (identifiant opaque, comme les autres).
- `LocalLabel { id: LocalLabelId, name: String, color: u32 }` ; `color` est une valeur ARGB (comme `localLabelColors` de l'app).
- `LocalLabelAssignment { thread: ThreadId, labels: Vec<LocalLabelId> }`.
- Plafonds : nom 64 octets, 200 libellés, 1000 conversations par appel de pose.

## 3. Stockage (migration v008, par profil)

- `local_labels(id PRIMARY KEY, name, color, created_at)`, nom unique sans tenir compte de la casse.
- `thread_local_labels(label_id REFERENCES local_labels ON DELETE CASCADE, account_id REFERENCES accounts ON DELETE CASCADE, root_key, PRIMARY KEY (label_id, account_id, root_key))`.
- **La pose est rattachée à `(compte, clé du fil)`, pas à l'identifiant du fil.** Raison : `recompute_threads` efface puis recrée les fils d'un compte, et le rejet du miroir fait de même ; la clé du fil, calculée à partir des en-têtes du message, redonne le même fil. Aucune clé étrangère vers `threads`.
- Résolution : un fil est identifié par son `ThreadId` côté API, et la table `threads(account_id, root_key)` donne la clé. Une pose vers un fil inconnu est refusée (`not_found.thread`).
- Les libellés sont de la configuration : migrés sans perte, conservés si le miroir est jeté.
- Supprimer un libellé supprime ses poses ; supprimer un compte supprime ses poses.

## 4. Conversations recalculées, fusionnées

- Recalcul d'un compte (`recompute_threads`) et resynchronisation : les poses subsistent, car elles suivent la clé du fil.
- Fusion de deux conversations : la conversation résultante garde le fil, donc les libellés de sa clé. Les poses de l'autre clé restent en base sans effet visible et réapparaîtraient si la conversation se scindait de nouveau. Limite documentée (AG), non bloquante.

## 5. API (domaine `local_label`)

- `ListLocalLabels`, `CreateLocalLabel { name, color?, idempotency_key }`, `RenameLocalLabel { label, name }`, `RecolorLocalLabel { label, color }`, `DeleteLocalLabel { label }`.
- `SetLocalLabel { label, threads: Vec<ThreadId>, on: bool, idempotency_key }` : pose ou retire sur des conversations (idempotent par nature : pose d'ensemble).
- `ListLocalLabelAssignments` : toutes les poses, rendues par conversation (`Vec<LocalLabelAssignment>`), limitées aux fils existants.
- `ImportLocalLabels { labels: Vec<{ name, color, conversations: Vec<String> }>, idempotency_key }` : reprise unique ; chaque clé de conversation est résolue comme un identifiant de fil, sinon comme un identifiant de message (rattaché à son fil), sinon ignorée ; la réponse compte les poses reprises et les clés ignorées.
- Validation : nom vide ou trop long, doublon (sans casse), plafonds dépassés ⇒ refus explicite sans écriture. Erreurs statiques, sans le contenu saisi.
- Événement : `LocalLabelsChanged` (catégorie `local_label`), émis après chaque écriture qui change quelque chose ; un rejeu idempotent n'émet rien.

## 6. Règles (extension de R1)

- Conditions : `HasLocalLabel { label }` (vrai si la conversation du mail porte le libellé).
- Actions : `AddLocalLabel { label }`, `RemoveLocalLabel { label }`. Sans compte unique exigé, sans boîte.
- `MessageContext` gagne `thread_labels: Vec<LocalLabelId>` (libellés de la conversation à l'arrivée). `evaluate` tient un ensemble courant mis à jour par chaque action de libellé : les règles suivantes voient les effets des précédentes.
- `RuleOutcome` gagne `label_changes: Vec<LabelChange { label, on }>` (ordre des règles, la dernière gagne).
- Application (`rules/apply`) : les changements sont écrits directement en base, sans file d'actions (aucun effet sur le serveur), puis `LocalLabelsChanged` est émis. La pose est un ensemble : un rejeu après crash ne change rien.
- Validation : le libellé doit exister (`RuleWorld::local_label_exists`) ; si le libellé est supprimé ensuite, la règle devient invalide (statut calculé à la lecture) et n'est plus exécutée.
- `PreviewRule` utilise les libellés réels des conversations.
- Catalogue (`ListRuleCatalog`) : les trois variantes, avec un paramètre `label` dont les choix sont les libellés locaux.

## 7. App

- `engine/local_label_admin.dart` (sur le modèle de `rule_admin.dart`) : liste, création, renommage, couleur, suppression, pose, poses de toutes les conversations, reprise.
- `MailStore` : en mode moteur, les libellés locaux et leurs poses viennent du moteur (relus sur `local_labels_changed` et après chaque écriture) ; `mail.labels` est complété par les poses de la conversation (clé = identifiant de fil, `m['thread']`). Sans moteur, comportement actuel inchangé.
- Reprise : si les préférences contiennent `localLabels` et pas `localLabelsMigrated`, l'app appelle `ImportLocalLabels` ; en cas de succès elle pose `localLabelsMigrated`, efface `localLabels` ; en cas d'échec elle garde sa copie et réessaie au lancement suivant.
- Éditeur de règles : choix de libellé local pour `HasLocalLabel`, `AddLocalLabel`, `RemoveLocalLabel`, depuis la liste des libellés ; ces variantes restent proposées avec « Tous les comptes ».

## 8. Tests

- Stockage : création, doublon sans casse, renommage, couleur, suppression en cascade, pose et retrait, pose sur fil inconnu refusée, suppression de compte, rejet du miroir (poses conservées), recalcul des fils (poses conservées), reprise sans doublon (même clé).
- API : chaque opération, plafonds, erreurs sans contenu saisi, idempotence, événements (émis, non émis au rejeu), snapshot de contrat (ajouts seuls).
- Règles : `HasLocalLabel` vrai/faux/négation ; ajout et retrait ; une règle voit l'effet de la précédente ; libellé supprimé ⇒ règle invalide ; reprise sur crash sans doublon ; aperçu ; un nouveau mail d'une conversation étiquetée est vu comme porteur du libellé.
- Pont : création, pose, liste, reprise de bout en bout.
- App : modèles, admin, bascule moteur/préférences, reprise (réussie, échouée), éditeur, affichage des libellés sur les conversations.

## 9. Livraison en lots

1. Moteur : types, migration v008, dépôt, API `local_label`, événement, reprise (`ImportLocalLabels`).
2. Moteur : extension des règles (types, évaluation, validation, application, catalogue, aperçu).
3. Pont : test de bout en bout.
4. App : admin, bascule `MailStore`, reprise, éditeur de règles.

Les lots 1 à 3 sont testables sans écran.
