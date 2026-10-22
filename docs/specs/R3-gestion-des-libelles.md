# R3 — Gestion des libellés (onglet dédié, réglages dans le moteur)

Un onglet « Libellés » dans les Paramètres pour gérer tous les libellés, ceux du serveur de chaque compte comme les libellés locaux (R2), avec des réglages par libellé gardés par le moteur.
Référence : §12 (tout ce que l'app fait existe d'abord dans le moteur, testable sans écran), R2 (libellés locaux par conversation), invariant 18.

## 1. Décisions validées

| Décision | Choix |
|---|---|
| Où | Réglages dans le moteur (base SQLite du profil). Le masquage de la Réception y est appliqué : liste, compteurs, notifications. |
| Libellés concernés | Libellés du serveur (boîtes de rôle « autre », par compte) et libellés locaux. |
| Réglages par libellé | masqué de la barre latérale ; masqué de la Réception ; notifications coupées ; compteur (non lus, total, aucun) ; couleur ; ordre. |
| Masquer de la Réception | Libellé **local** : la conversation entière est masquée. Libellé **serveur** : les messages rangés dans ce libellé. On les retrouve en ouvrant le libellé. |
| Notifications | Un mail masqué de la Réception ne notifie pas à son arrivée. |
| Gestion existante | L'onglet remplace la gestion des libellés de l'onglet Comptes et la section des libellés locaux. |

Hors périmètre : libellés imbriqués, synchronisation des réglages entre appareils, règle « marquer lu » liée à un libellé (les règles le font déjà).

## 2. Stockage (migration v010)

`label_settings(kind TEXT, label_id TEXT, hidden INTEGER DEFAULT 0, hide_from_inbox INTEGER DEFAULT 0, notifications INTEGER DEFAULT 1, counter TEXT DEFAULT 'unread', color INTEGER NULL, position INTEGER NULL, PRIMARY KEY (kind, label_id))`.
- `kind` vaut `server` (identifiant de boîte) ou `local` (identifiant de libellé local). Pas de clé étrangère (deux types) : la suppression d'une boîte ou d'un libellé local supprime sa ligne dans la même transaction, et une ligne orpheline est ignorée à la lecture.
- Absence de ligne = réglages par défaut. `counter` ∈ `unread`, `total`, `none`.

## 3. Effet de « masqué de la Réception »

- Un message est masqué de la Réception si, soit il a une apparition non supprimée dans une boîte serveur dont `hide_from_inbox = 1`, soit son fil porte un libellé local dont `hide_from_inbox = 1` (jointure sur la clé du fil, comme R2).
- La condition s'ajoute à la liste de la Réception (vue de la boîte de rôle `inbox` et vue standard « réception »), sans changer les autres vues, la recherche ni les vues de libellés.
- Les compteurs (`total`, `unread`) des boîtes de rôle `inbox` ignorent ces messages. Ils sont recalculés à chaque changement de réglage, de pose ou de retrait d'un libellé local (action, règle, reprise, suppression) et à chaque recalcul habituel ; `MailboxCountersUpdated` est émis pour chaque Réception touchée.
- À l'arrivée d'un mail, `NewMessageReceived.silenced` est vrai s'il est masqué de la Réception (règles appliquées d'abord : un libellé posé par une règle compte).
- Si les notifications d'un libellé sont coupées, `silenced` est vrai pour tout message qui porte ce libellé à l'arrivée.

## 4. API (domaine `label_settings`)

- `ListLabels` : tous les libellés, avec `kind`, `id`, compte (serveur), nom, couleur effective, nombre de messages, `unused` (aucun message) et leurs réglages, dans l'ordre d'affichage (locaux d'abord, puis par compte ; `position` puis nom).
- `SetLabelSettings { label: { kind, id }, hidden?, hide_from_inbox?, notifications?, counter?, color? }` : modifie les champs fournis ; idempotent par nature (état posé). Libellé inconnu : `not_found.label` ; valeur de `counter` inconnue : refus sans écriture.
- `ReorderLabels { order: [ { kind, id } ] }` : fixe `position` selon la liste ; liste incomplète ou avec un libellé inconnu : refus.
- Événement `LabelSettingsChanged` (catégorie `label`) après chaque écriture qui change quelque chose.
- Création, renommage et suppression restent aux domaines existants (`mailbox` pour le serveur, `local_label` pour les locaux) ; à la suppression, les réglages du libellé disparaissent.

## 5. App

- Onglet « Libellés » (Paramètres) : libellés locaux puis ceux de chaque compte ; par ligne : pastille de couleur (changeable), nom, nombre de mails, interrupteurs « Afficher dans la barre latérale », « Masquer de la Réception », « Notifications », choix du compteur (non lus | total | aucun), monter et descendre, renommer, supprimer ; en tête : ajouter un libellé (local ou d'un compte) ; les libellés sans message sont signalés ; raccourci « Créer une règle » qui ouvre l'éditeur de règles prérempli sur ce libellé.
- Barre latérale : libellés masqués absents, ordre et couleur des réglages, compteur selon le choix.
- Retrait de la gestion des libellés de l'onglet Comptes et de la section des libellés locaux.
- Les réglages se relisent sur `label_settings_changed`.

## 6. Tests

- Stockage : valeurs par défaut, modification partielle, suppression en cascade logique (boîte, libellé local), lignes orphelines ignorées.
- Réception : un message d'un libellé serveur masqué n'est plus dans la liste ni dans les compteurs, mais l'est dans la vue du libellé ; une conversation à libellé local masqué disparaît de la Réception ; poser ou retirer le libellé change la liste et les compteurs ; les autres vues et la recherche ne changent pas.
- Notifications : un mail masqué ou à notifications coupées arrive avec `silenced` vrai.
- API : chaque opération, valeurs refusées, ordre, événement, aperçu de contrat (ajouts seuls).
- Pont : réglage et lecture de bout en bout.
- App : modèles, onglet (liste, interrupteurs, ordre), barre latérale (masqué, compteur), retrait des anciennes gestions.

## 7. Livraison en lots

1. Moteur : stockage, API `label_settings`, événement.
2. Moteur : effet sur la Réception (liste, compteurs) et sur les notifications.
3. Pont : test de bout en bout.
4. App : onglet, barre latérale, retrait des anciennes gestions.
