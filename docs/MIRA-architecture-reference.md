# MIRA — Document de référence d'architecture du moteur

**Mail Interface & Retrieval Application — le moteur (backend) en Rust**

Version 0.1 — 10 octobre 2026 — Document vivant

---

## Table des matières

0. Mode d'emploi du document
1. Vision et périmètre
2. Principes directeurs
3. Registre des décisions
4. Historique du raisonnement : pourquoi cette architecture
5. Architecture générale
6. Les providers
7. Profils, comptes et identités
8. Le modèle de domaine
9. Messages, boîtes et vues
10. La synchronisation
11. Les fonctionnalités d'un client mail : étude et couverture
12. Le contrat d'API du moteur
13. Scénarios de bout en bout
14. Invariants et règles de cohérence
15. Sujets restant à traiter
16. Risques et questions ouvertes
17. Feuille de route
18. Annexes

---

## 0. Mode d'emploi du document

### 0.1 Objet

Ce document est la **référence d'architecture du moteur de MIRA**. Il rassemble, au même endroit, tout ce qui a été discuté, décidé, proposé et laissé ouvert pendant la phase de conception : le raisonnement qui a mené aux choix, les alternatives écartées, le découpage du code, le modèle de données, le fonctionnement de la synchronisation et le contrat que le moteur offre à ses clients (le CLI d'abord, l'interface Flutter ensuite).

Il remplit trois fonctions. D'abord, il sert de **mémoire** : dans six mois, personne ne se souviendra pourquoi les plugins dynamiques ont été abandonnés ou pourquoi un message n'existe qu'une fois dans la base, et ce document le dit. Ensuite, il sert de **référence de travail** : quand on écrira le code, c'est ici qu'on vérifiera qu'une idée respecte l'architecture. Enfin, il sert de **base de discussion** : chaque point encore ouvert y est signalé comme tel, avec les options connues, pour que les décisions restantes se prennent en connaissance de cause.

### 0.2 À qui il s'adresse

Le lecteur principal est le porteur du projet, qui connaît bien le C++ (il a réalisé le projet Arcade, une plateforme de jeux rétro à base de bibliothèques dynamiques) mais ne maîtrise pas encore Rust. Le document explique donc les notions propres à Rust en les rattachant au C++ chaque fois que c'est utile (voir l'annexe B). Il s'adresse aussi à d'éventuels contributeurs futurs, qui y trouveront les règles du jeu avant d'écrire une ligne.

### 0.3 Portée

Le document traite **uniquement du moteur**, c'est-à-dire de la partie Rust qui gère les comptes, parle aux serveurs, analyse les messages, stocke et synchronise. L'interface Flutter n'y apparaît qu'à travers le contrat que le moteur lui offre. Les choix d'interface, de design et d'expérience utilisateur sont hors périmètre.

### 0.4 Statuts des éléments

Chaque décision, proposition ou question est marquée par un statut. Il est essentiel de les distinguer, car ce document mélange des choix fermes et des idées encore en discussion.

- **[DÉCIDÉ]** : choix explicitement validé par le porteur du projet pendant la conception. On ne le remet en cause que par une discussion délibérée.
- **[PROPOSÉ]** : proposition formulée pendant la conception, qui n'a pas été explicitement validée ou rejetée, ou qui a été rédigée à la demande du porteur sans relecture point par point. Elle sert de base de travail mais peut changer.
- **[OUVERT]** : question identifiée, pas encore tranchée. Les options connues sont décrites.
- **[BRIEF]** : élément issu du brief initial du projet. Il a valeur d'intention de départ, pas de décision d'architecture détaillée.

### 0.5 Identifiants de décision

Les décisions et les points ouverts importants portent un identifiant de la forme **D-xx**. Le registre du chapitre 3 les liste tous avec leur statut et le chapitre qui les développe. Dans le texte, on renvoie à ces identifiants entre parenthèses, par exemple « (D-12) ». Un identifiant n'est jamais réutilisé : si une décision est remplacée, l'ancienne est marquée comme abandonnée et la nouvelle reçoit un nouvel identifiant.

### 0.6 Vocabulaire

Quelques termes reviennent partout et ont un sens précis dans ce document. Le glossaire complet est en annexe A, mais voici les plus importants.

- **Moteur** : la bibliothèque Rust qui contient toute la logique de MIRA.
- **Cœur** : la partie du moteur qui ne connaît aucun fournisseur de messagerie en particulier.
- **Provider** : la description d'un fournisseur de messagerie (Gmail, Outlook, Yahoo, « Autre »), c'est-à-dire ses serveurs, ses méthodes d'authentification et ses particularités.
- **Compte** : une connexion d'un utilisateur à un serveur de messagerie, via un provider.
- **Boîte** : une vue d'appartenance réelle sur les messages d'un compte, correspondant à un dossier ou à un libellé côté serveur. L'interface pourra l'appeler « dossier ».
- **Message** : un courrier, qui existe **une seule fois** dans la base du moteur, même s'il apparaît dans plusieurs boîtes.
- **Apparition** : la présence d'un message dans une boîte précise, avec son numéro côté serveur.
- **Action en attente** : une opération voulue par l'utilisateur, déjà appliquée en local mais pas encore confirmée par le serveur.

### 0.7 Ce que ce document n'est pas

Ce n'est pas un document de code : il ne contient ni structures, ni signatures de fonctions, ni exemples en Rust. Le choix est volontaire. La conception se fait en termes d'interfaces, de dépendances et de responsabilités, comme pour Arcade, et la traduction en code viendra après. Un premier brouillon de code existe (voir l'annexe E), mais il a été produit avant que l'architecture soit fixée et ne fait pas référence.

Ce n'est pas non plus un document d'interface : aucune maquette, aucune décision de présentation.

### 0.8 Faire évoluer le document

Le document est vivant. Toute décision nouvelle s'ajoute au registre avec son identifiant et son statut, et le chapitre concerné est mis à jour dans la foulée. Une décision qui change n'est pas effacée : l'ancienne reste visible, marquée comme remplacée, avec la raison du changement. C'est ce qui fait la valeur d'un registre.

---

## 1. Vision et périmètre

### 1.1 Le projet MIRA

MIRA, pour *Mail Interface & Retrieval Application*, est un client de messagerie **moderne, performant, sécurisé et multi-comptes**, conçu spécifiquement pour Linux. Il associe un moteur très rapide écrit en Rust, chargé du réseau, de l'analyse des messages et de la base locale, à une interface fluide en Flutter Desktop. Il fonctionne en mode **offline-first** : on peut lire, trier et rédiger sans connexion, et tout se rattrape quand le réseau revient. À terme, il accueillera un **agent d'intelligence artificielle** pour la recherche sémantique et l'assistance.

Ce que MIRA veut éviter est aussi important que ce qu'il veut offrir. Il ne cherche pas à devenir une suite bureautique complète, ni un service hébergé : c'est un client local, qui parle directement aux serveurs de messagerie de l'utilisateur, conserve ses données sur sa machine et protège ses secrets avec les outils du système.

### 1.2 Ce que couvre ce document

Le moteur de MIRA est la partie la plus structurante du projet : tout le reste (interface, agent IA, intégration bureau) s'appuie sur lui. Une erreur de modèle de données ou de découpage y coûte cher, parce que les données de l'utilisateur y vivent et qu'une migration est toujours pénible. C'est pourquoi la phase de conception se concentre sur lui, avant d'écrire du code ou de toucher à l'interface.

### 1.3 Objectifs du moteur

Le moteur doit permettre, de façon fiable :

1. de connecter plusieurs comptes de plusieurs providers, avec une authentification adaptée (OAuth2 ou mot de passe d'application) ;
2. de synchroniser les messages dans les deux sens, y compris après une longue période hors ligne ;
3. de présenter une base locale rapide à interroger, avec recherche en texte intégral ;
4. d'envoyer des messages sans jamais les perdre ni les dupliquer ;
5. de réagir en temps réel à l'arrivée de nouveaux messages ;
6. d'être utilisé **sans interface**, en ligne de commande, de façon à pouvoir tout tester sans écran ;
7. de rester extensible : ajouter un provider doit être une opération localisée et peu coûteuse ;
8. de préparer l'arrivée d'un agent d'IA sans le coupler au reste.

### 1.4 Non-objectifs

Pour que le périmètre reste tenable, plusieurs choses sont explicitement **hors du projet** à ce stade :

- un service multi-utilisateurs hébergé (on travaille sur un client local) ;
- des plugins tiers chargés à l'exécution (voir le chapitre 4, décision D-02) ;
- la gestion d'un agenda ou d'un carnet d'adresses complet (des contacts minimaux pourront venir plus tard) ;
- la collaboration d'équipe (boîtes partagées, délégations, administration) ;
- les fonctions de suivi de lecture destinées à l'envoi en masse ;
- le support des protocoles propriétaires non IMAP/SMTP en version 1 (Exchange natif, API Gmail, Microsoft Graph), même si l'architecture doit les laisser possibles.

### 1.5 Contexte d'usage

MIRA est un **client Linux natif**. Cela a plusieurs conséquences qui reviendront dans tout le document. Les secrets (mots de passe, jetons) se rangent dans le trousseau du système (SecretService de Freedesktop, par exemple gnome-keyring ou KWallet). Les données de l'utilisateur vivent dans les dossiers standards de sa session. Les notifications passent par le bus de session. Et la séparation entre utilisateurs est déjà assurée par le système : deux personnes sur la même machine ont chacune leur session, donc leur propre exemplaire de MIRA.

### 1.6 La pile technique prévue par le brief

Le brief d'origine fixe une pile technique, que ce document reprend comme point de départ **[BRIEF]** :

- **Concurrence et réseau** : asynchrone, avec `tokio`.
- **Protocoles** : IMAP avec l'extension IDLE pour la réception en temps réel, SMTP pour l'envoi, authentification OAuth2 (Gmail, Outlook) ou TLS avec mot de passe d'application.
- **Analyse des messages** : la caisse `mail-parser`, pour un traitement robuste et sûr du texte brut, du HTML et des pièces jointes.
- **Stockage** : une base SQLite locale (comptes, dossiers, messages), le moteur de recherche en texte intégral FTS5 intégré, et l'extension vectorielle `sqlite-vec` pour préparer l'indexation sémantique.
- **Briques existantes** : possibilité, facultative, de s'appuyer sur les caisses du projet Himalaya pour accélérer le moteur IMAP/SMTP.
- **Interface** : Flutter Desktop avec un lanceur Linux natif (GTK, Libadwaita), notifications via `org.freedesktop.Notifications`, liaison avec le moteur par `flutter_rust_bridge`.
- **Sécurité** : trousseau SecretService, rendu HTML sans JavaScript et avec blocage par défaut des ressources distantes et des pixels espions.
- **Agent d'IA** : bus d'événements interne (par exemple `NewEmailReceived`), recherche sémantique sur les représentations vectorielles stockées dans SQLite, résumés de fils, tri et classification, aide à la rédaction, modèles locaux (Ollama, llama.cpp) ou API distantes.

Ces choix de bibliothèques n'ont pas été rediscutés pendant la phase de conception : ils restent valables jusqu'à preuve du contraire. Deux points du brief ont toutefois été nuancés par les décisions prises ici : le langage du moteur a été revalidé (D-01) et la liaison avec Flutter reste à trancher entre appel direct et processus séparé (D-30).

### 1.7 La feuille de route du brief

Le brief prévoit cinq étapes **[BRIEF]** :

1. **Moteur de base** : validation de la connexion IMAP/SMTP, authentification OAuth2 et basique, analyse MIME.
2. **Persistance** : SQLite et FTS5, schéma local multi-comptes, indexation du texte, cache des pièces jointes.
3. **Liaison Flutter** : connexion du moteur à l'interface, création de l'interface multi-comptes.
4. **Intégration Linux et sécurité** : trousseau, notifications, rendu HTML sécurisé.
5. **Agent d'IA** : recherche sémantique locale, résumés, assistance.

Le chapitre 17 reprend cette feuille de route en tenant compte des décisions d'architecture, notamment l'arrivée du CLI comme premier client.

---

## 2. Principes directeurs

Ces principes ne sont pas des décisions isolées : ce sont les règles de lecture de toutes les décisions. Quand un choix de détail semble hésiter entre deux solutions, on revient ici.

### P1 — Le moteur existe sans interface

Le moteur est une bibliothèque complète, utilisable en ligne de commande avant d'avoir la moindre interface graphique. L'interface Flutter n'est qu'un client parmi d'autres du même contrat. **Conséquence** : tout ce que l'interface saura faire devra d'abord exister comme commande du moteur, donc être testable sans écran. **Anti-modèle** : une logique « cachée » dans l'interface (une règle de tri, une résolution de conflit) qui ne serait pas reproductible en ligne de commande. C'est la même discipline que celle d'Arcade, où les bibliothèques graphiques ne devaient en aucun cas influencer la logique de jeu.

### P2 — Une frontière interne plutôt que des plugins dynamiques

La modularité vient de **frontières nettes dans le code**, appliquées à la compilation, et pas du chargement de code à l'exécution. Le cœur ne connaît aucun provider concret ; les providers sont fournis au cœur par un point d'assemblage. **Conséquence** : un seul exécutable, pas de bibliothèque dynamique à charger, pas de problème de compatibilité binaire. **Anti-modèle** : un `si provider == Gmail` dans le cœur.

### P3 — La généricité, héritée d'Arcade

Dans Arcade, le cœur devait traiter toutes les bibliothèques de façon **uniforme** : aucune ne devait être traitée différemment. Ici, le cœur traite tous les providers de la même manière, y compris le provider « Autre ». Toute différence passe par des **données** (le profil) ou par des **hooks** déclarés (les particularités), jamais par une identité nommée dans le cœur.

### P4 — Le serveur est la vérité, le local est un miroir plus des intentions

Les données se répartissent en trois familles : la configuration, le **miroir du serveur** (qu'on peut jeter et reconstruire) et les **intentions locales** (brouillons, actions en attente, qui ne sont nulle part ailleurs). En cas de désaccord entre le miroir et le serveur, le serveur gagne toujours (D-23). Les intentions locales, elles, sont rejouées par-dessus.

### P5 — Lire en local, écrire en intention

Toute lecture interroge la base locale : elle est rapide et fonctionne hors ligne. Toute modification s'applique immédiatement en local, puis est enregistrée comme action en attente, que la synchronisation rejouera sur le serveur. Le réseau n'est jamais sur le chemin d'une lecture ordinaire.

### P6 — Se dégrader selon les capacités, pas selon l'identité

Le cœur ne demande jamais « est-ce Outlook ? » mais « ce serveur sait-il faire X ? ». Il choisit sa stratégie d'après ce que le serveur annonce (extensions IMAP, par exemple) et d'après les exceptions connues du provider. Un serveur inconnu, un provider « Autre », se comporte donc correctement, avec les mêmes règles.

### P7 — Préférer le doublon à la perte

Quand le moteur hésite sur l'identité de deux messages, il préfère en afficher deux plutôt que d'en fusionner à tort deux qui sont distincts. Un doublon est gênant mais récupérable ; une perte de message est grave et silencieuse.

### P8 — L'isolation par construction

Ce qui doit être séparé l'est par la structure, pas par la discipline : un espace de données par profil, des secrets rangés par profil et par compte, une tâche de synchronisation par compte. On évite ainsi qu'un oubli de filtre dans une requête mélange les données de deux comptes ou de deux profils.

### P9 — Aucun secret en dehors du trousseau

Mots de passe et jetons n'existent que dans le trousseau du système. Ils ne sont ni stockés dans la base, ni écrits dans les journaux, ni renvoyés par l'API du moteur. Le compte ne conserve qu'une **référence** vers son secret.

### P10 — Tout doit être testable sans réseau

La plupart de la logique (choix de stratégie, résolution de conflits, application d'actions, traduction par les providers) doit pouvoir être exercée avec de faux serveurs et de faux providers, sans connexion. Cette exigence est une raison de plus de garder les providers purs, sans entrées-sorties (D-07).

### P11 — La simplicité d'abord, l'extensibilité par des frontières

On ne construit pas d'abstraction pour un besoin hypothétique. En revanche, on place des **frontières** aux endroits où un changement futur est probable et coûteux : entre le cœur et les providers, entre le modèle de domaine et le protocole, entre le moteur et ses clients. La frontière est peu coûteuse à poser maintenant et très coûteuse à poser plus tard.

### P12 — Les décisions s'écrivent avec leur raison

Une décision sans raison est un caprice, et une raison oubliée rend la décision indéfendable. Chaque choix important est donc consigné avec ses alternatives et ses motifs, pour qu'on puisse le réviser sans repartir de zéro quand le contexte change.

---

## 3. Registre des décisions

Ce registre liste, dans l'ordre où elles sont apparues, toutes les décisions et tous les points ouverts. Le détail et la justification se trouvent dans le chapitre indiqué.

| ID | Sujet | Décision ou question | Statut | Chap. |
|---|---|---|---|---|
| D-01 | Langage du moteur | Rust, après examen d'une alternative C++ | DÉCIDÉ | 4 |
| D-02 | Plugins | Pas de plugins chargés à l'exécution (ni `.so`, ni WASM, ni processus externe) | DÉCIDÉ | 4 |
| D-03 | Découpage | Frontière interne entre cœur et providers, en trois blocs : cœur, providers, assemblage | DÉCIDÉ | 5 |
| D-04 | Clients du moteur | Moteur utilisable en CLI d'abord ; Flutter n'est qu'une interface | DÉCIDÉ | 4, 5 |
| D-05 | Définition d'un provider | Un profil (données) plus des particularités (code optionnel) | DÉCIDÉ | 6 |
| D-06 | Règle du cœur | Le cœur ne nomme jamais un provider ; il raisonne en capacités | DÉCIDÉ | 5, 6 |
| D-07 | Providers purs | Les providers n'effectuent aucune entrée-sortie ; ils décrivent et traduisent | PROPOSÉ | 6 |
| D-08 | Capacités | La vérité est ce que le serveur annonce ; le provider ne fournit que des exceptions | PROPOSÉ | 6 |
| D-09 | Utilisateurs | Pas de notion d'utilisateur applicatif : un profil par session système | DÉCIDÉ | 7 |
| D-10 | Profils | Porte ouverte aux profils multiples : moteur ouvert sur un profil, données par profil, secrets par profil et compte | DÉCIDÉ | 7 |
| D-11 | Réglages | Distinguer réglages d'application et réglages de profil | PROPOSÉ | 7 |
| D-12 | Modèle des messages | Un message unique ; les boîtes en sont des vues d'appartenance ; vues calculées en plus ; vocabulaire « dossier » en interface | DÉCIDÉ | 9 |
| D-13 | Identité | Identifiant interne MIRA, table de correspondance vers les identifiants serveur, pas de fusion en cas de doute | DÉCIDÉ | 9 |
| D-14 | Drapeaux | Drapeaux utilisateur au niveau du message ; « supprimé » interne à l'apparition | DÉCIDÉ | 9 |
| D-15 | Fils | Calculés par MIRA, même algorithme partout, indice du serveur utilisé s'il existe | DÉCIDÉ | 9 |
| D-16 | Stockage du message | Source brute compressée, analyse en cache reconstruisible, pièces jointes à la demande | DÉCIDÉ | 9 |
| D-17 | Libellés multiples | Sur un provider à dossiers, ajouter un second libellé est refusé ou devient un déplacement | PROPOSÉ | 9 |
| D-18 | Familles de données | Configuration, miroir du serveur, intentions locales | DÉCIDÉ | 8 |
| D-19 | Entités | Profil, compte, identité, boîte, message, apparition, pièce jointe, fil, brouillon, action en attente, vue calculée | DÉCIDÉ | 8 |
| D-20 | Descente | Téléchargement par étapes : boîtes, en-têtes, corps, pièces jointes | DÉCIDÉ | 10 |
| D-21 | Fenêtre locale | En-têtes des 90 derniers jours, corps préchargés pour les plus récents, bouton pour remonter | DÉCIDÉ | 10 |
| D-22 | Détection des changements | Stratégie choisie d'après les capacités du serveur | DÉCIDÉ | 10 |
| D-23 | Conflits | Le serveur gagne toujours ; les actions locales sont rejouées par-dessus | DÉCIDÉ | 10 |
| D-24 | Gmail « Tous les messages » | Optimisation de téléchargement notée, reportée | DÉCIDÉ | 10 |
| D-25 | Écritures | Application locale immédiate, file d'actions, résultat par événement | PROPOSÉ | 10, 12 |
| D-26 | Contrat d'API | Contrat des commandes, événements et erreurs, tel que décrit au chapitre 12 | PROPOSÉ | 12 |
| D-27 | Verrou de profil | Un seul moteur actif par profil en version 1 | OUVERT | 12 |
| D-28 | Découpage v1 | Quelles commandes en version 1 et lesquelles plus tard | OUVERT | 12 |
| D-29 | Sauvegarde | Commandes d'export et de sauvegarde en version 1 ou non | OUVERT | 12 |
| D-30 | Liaison avec Flutter | Appel direct en processus ou processus moteur séparé | OUVERT | 15 |
| D-31 | Pile du brief | tokio, mail-parser, SQLite/FTS5/sqlite-vec, trousseau, flutter_rust_bridge | BRIEF | 1 |
| D-32 | Taille cible | Nombre de comptes et de messages visés | OUVERT | 16 |
| D-33 | Vocabulaire commun | Types du domaine et interface provider forment une petite bibliothèque distincte dans le bloc cœur, seule dépendance des providers | PROPOSÉ | 5 |
| D-34 | Découpage interne du cœur | Sous-domaines (API, comptes, authentification, trousseau, clients IMAP/SMTP, MIME, stockage, synchro, événements, registre), chaque bibliothèque externe encapsulée | PROPOSÉ | 5 |

### 3.1 Notes sur certains statuts

Quelques décisions méritent une précision sur la façon dont elles ont été prises.

**D-07 et D-08** sont des conséquences directes de D-05 et D-06, présentées pendant la discussion sur la frontière. Elles n'ont pas été contestées, mais elles n'ont pas fait l'objet d'une validation explicite distincte : elles restent donc **proposées**.

**D-11** est une recommandation formulée en complément de D-10, qui n'a pas non plus été validée séparément.

**D-17** est un cas à part : la question a été posée (refuser ou transformer en déplacement, ou conserver un libellé purement local), l'option « refus ou déplacement » a été recommandée, mais la réponse du porteur a porté sur le modèle général (D-12) sans trancher cette sous-question. Elle reste donc proposée.

**D-25 et D-26** découlent du contrat d'API rédigé à la demande du porteur. Le contrat est complet dans ses principes, mais il n'a pas été relu commande par commande ; trois points restent ouverts (D-27, D-28, D-29).

### 3.2 Comment lire les dépendances entre décisions

Les décisions ne sont pas indépendantes. La figure suivante en résume l'enchaînement logique.

```
D-04 (moteur sans interface) ───► D-26 (contrat d'API)
D-02 (pas de plugins) ──► D-03 (frontière interne) ──► D-05 (provider = profil + particularités)
                                   │                              │
                                   ▼                              ▼
                              D-06 (pas de nom       D-08 (capacités : le serveur d'abord)
                              de provider dans le cœur)          │
                                                                  ▼
D-12 (message unique, boîtes = vues) ──► D-13 (identité) ──► D-22 (stratégie de synchro)
        │                                      │
        ▼                                      ▼
D-14 (drapeaux) , D-15 (fils)          D-16 (stockage brut)
```

Lecture : abandonner les plugins (D-02) a conduit à poser une frontière interne (D-03), qui impose de définir ce qu'est un provider (D-05) et de garder le cœur indépendant (D-06). Le choix d'un message unique (D-12) impose une identité propre à MIRA (D-13), qui conditionne la détection des changements (D-22).

---
## 4. Historique du raisonnement : pourquoi cette architecture

Ce chapitre retrace le chemin qui a conduit à l'architecture actuelle. Il est précieux pour une raison simple : les décisions les plus importantes du projet sont des **abandons** (pas de plugins dynamiques, pas de C++), et un abandon ne se comprend qu'en connaissant ce qu'il remplace.

### 4.1 Le point de départ : le projet Arcade

Le porteur du projet a réalisé à l'école un projet nommé **Arcade** : une plateforme de jeux rétro en C++ dont les jeux et les bibliothèques graphiques étaient des bibliothèques dynamiques (des fichiers `.so`) chargées à l'exécution. L'idée de départ pour MIRA était de transposer cette approche aux fournisseurs de messagerie : un `.so` par provider (Google, Yahoo, Outlook, « Autre »), chargé dynamiquement par le moteur.

Les contraintes d'Arcade, qui ont servi de modèle, étaient les suivantes :

- les bibliothèques graphiques et les jeux devaient être des **bibliothèques dynamiques chargées à l'exécution**, avec les fonctions `dlopen`, `dlclose`, `dlsym` et `dlerror`, et une **encapsulation** de cette interface en C dans une classe C++ ;
- l'exécutable principal ne devait montrer aucune dépendance vers ces bibliothèques lorsqu'on lançait `ldd` ;
- il était **strictement interdit** de mentionner explicitement une bibliothèque graphique dans le programme principal ;
- les bibliothèques graphiques ne devaient en aucun cas influencer la logique des jeux, et les jeux ne devaient contenir aucune information de rendu ni d'évènement bas niveau ;
- toutes les bibliothèques devaient être traitées de façon **générique et uniforme** (« keep genericity ») ;
- le build devait offrir des règles séparées pour le cœur, les jeux et les bibliothèques graphiques ;
- les interfaces devaient être partagées avec un autre groupe pour que les jeux d'un groupe tournent avec la plateforme de l'autre ;
- les erreurs allaient sur la sortie d'erreur, avec le code de sortie 84 ;
- une documentation expliquant comment ajouter une bibliothèque était exigée.

Cet ensemble de contraintes est une excellente discipline d'architecture. Ce que MIRA en retient, et ce qu'il laisse de côté, est détaillé en 4.7.

### 4.2 Premier obstacle : Rust n'a pas d'interface binaire stable

En C++, le schéma d'Arcade fonctionne parce que le noyau et les bibliothèques sont compilés avec le même compilateur et la même bibliothèque standard : une interface abstraite pure (comme `IGame` ou `IGfx`), une fonction de fabrique exportée en C, et un chargement par `dlopen`. Les tables de méthodes virtuelles ont la même forme des deux côtés.

En Rust, la situation est différente. Les objets de trait (`dyn Trait`), les collections (`Vec`), les chaînes (`String`) et les fonctions asynchrones n'ont **aucune disposition mémoire garantie** d'une compilation à l'autre. Une bibliothèque dynamique écrite en Rust et chargée par un exécutable Rust peut planter dès que la version du compilateur change, voire dès que les options de compilation diffèrent. Ce n'est pas un défaut de jeunesse : le langage ne promet pas cette stabilité.

Quatre voies existaient pour obtenir malgré tout de la modularité :

| Option | Principe | Avantages | Inconvénients |
|---|---|---|---|
| **A. Modules statiques** | Chaque provider est un module compilé avec le reste, relié par une interface et un registre | Simple, sûr, compatible avec la liaison Flutter et le packaging Flatpak | Pas de provider tiers sans recompiler |
| **B. `.so` avec outils d'ABI stable** | Bibliothèques dynamiques via des caisses qui fournissent une ABI stable | Proche d'Arcade | Lourd, programmation asynchrone pénible, peu de gain réel ici |
| **C. WebAssembly** | Providers compilés en modules isolés | Isolation, multi-langage | Réseau et TLS à exposer par l'hôte, complexité |
| **D. Processus externes** | Providers en processus séparés parlant JSON-RPC | Isolation totale, n'importe quel langage | Latence, gestion de processus |

### 4.3 Deuxième constat : un provider n'est pas un protocole

Dans Arcade, chaque `.so` apportait **un comportement complet** : un jeu entier, un moteur de rendu entier. Dans MIRA, la situation est très différente. Gmail, Outlook, Yahoo et la plupart des autres fournisseurs parlent tous **IMAP** pour la réception et **SMTP** pour l'envoi. Ce qui change d'un fournisseur à l'autre, c'est :

- les **hôtes, ports et modes de chiffrement** des serveurs ;
- la **méthode d'authentification** (OAuth2 avec ses adresses et ses périmètres d'autorisation, ou mot de passe d'application) ;
- les **particularités** : les libellés propres à Gmail et les identifiants de fil qui vont avec, les noms de dossiers spéciaux (par exemple un dossier d'envoyés qui porte un nom particulier), la fiabilité de la réception en temps réel, les limites de débit ;
- éventuellement, à terme, une **API native** (l'API de Gmail, Microsoft Graph) à la place d'IMAP.

Conclusion : réécrire un moteur IMAP par provider serait absurde. Le **moteur IMAP/SMTP est unique et générique**, et le provider est une **description** (données, plus quelques particularités). C'est plus léger, plus testable, et cela évite de dupliquer la partie la plus difficile du projet.

Cette observation affaiblit déjà l'intérêt des plugins dynamiques : ils résolvent le problème d'Arcade (charger du comportement), pas celui de MIRA (décrire des différences).

### 4.4 La question du C++

Face à l'obstacle de l'ABI, une question légitime a été posée : et si le moteur était écrit en C++, comme Arcade ? L'analyse a pesé les deux côtés.

**Ce que C++ apporterait.** Les plugins `.so` deviennent naturels : interface pure virtuelle, fonction de fabrique en `extern "C"`, `dlopen`. Cela marche en pratique tant que le cœur et les plugins sont compilés avec le même compilateur et la même bibliothèque standard, ce qui était le cas à l'école. Le porteur réutiliserait en outre son expérience et son code d'Arcade (encapsulation de `dlopen`, registre).

**Ce que C++ ferait perdre.**

- **La sûreté mémoire, là où elle compte le plus.** Un client de messagerie analyse du MIME et du HTML provenant d'inconnus : c'est sa surface d'attaque principale. Rust écarte des classes entières de bogues à cet endroit précis ; en C++, cela demande une discipline permanente, du fuzzing et des outils d'analyse (sanitizers).
- **La liaison avec Flutter.** Le brief s'appuie sur `flutter_rust_bridge`, qui génère automatiquement une communication transparente entre Dart et Rust. En C++, il faudrait passer par `dart:ffi` avec une interface C écrite à la main, avec une gestion manuelle de la mémoire aux frontières, des rappels (callbacks) et des flux plus pénibles.
- **L'écosystème.** `tokio`, `mail-parser`, les caisses de Himalaya, les bibliothèques de trousseau et d'OAuth2 n'ont pas d'équivalent aussi mûr en C++. Il faudrait assembler libcurl ou une bibliothèque de messagerie, une couche asynchrone, un client OAuth2 maison, libsecret, avec à construire soi-même la réception en temps réel (IDLE), les reconnexions et l'asynchronisme.
- Un **build plus lourd** (CMake, gestionnaire de dépendances) et un **packaging** plus fastidieux.

**Le point décisif.** Ce qui est stable, ce n'est pas le C++, c'est le **C**. Si l'on tenait absolument à des bibliothèques dynamiques, Rust sait exposer une interface en C (`#[repr(C)]`, `extern "C"`), et les modules pourraient alors être écrits en Rust, en C++ ou en C. Autrement dit, le gain apparent du C++ (la stabilité du chargement dynamique) n'est pas réservé à C++.

**Décision D-01 : le moteur reste en Rust.** Le porteur a confirmé : « on garde le Rust ». Les raisons principales sont la sûreté mémoire sur l'analyse de contenu hostile, la liaison avec Flutter, la maturité de l'écosystème asynchrone, et le fait que le gain du C++ n'est de toute façon pas exclusif.

Un point d'honnêteté : le porteur a déclaré ne pas maîtriser Rust. C'est un coût réel d'apprentissage, assumé. L'architecture est donc conçue pour être comprise en termes de **responsabilités et d'interfaces** plutôt qu'en termes de subtilités du langage, et l'annexe B traduit les notions de Rust en équivalents C++.

### 4.5 L'abandon des plugins dynamiques

Quand le porteur a demandé ce que l'abandon des plugins apporterait et enlèverait, l'analyse a distingué trois niveaux, car « laisser tomber le plugin » peut vouloir dire trois choses.

| Niveau | Principe | Exemple |
|---|---|---|
| **A. Plugins chargés à l'exécution** | Fichiers `.so` ou WebAssembly chargés au démarrage | Arcade |
| **B. Providers compilés derrière une interface** | Modularité du code, sans chargement dynamique | Ce qui est retenu |
| **C. Aucune abstraction** | Un provider est une valeur d'énumération ou une ligne de table | Tout en dur, avec des `match` partout |

**Abandonner le niveau A (chargement dynamique)**

*Ce que cela apporte :*

- plus de problème d'interface binaire entre versions du compilateur, plus de code non sûr (`unsafe`) ni de bibliothèque de chargement ;
- un seul exécutable, donc un packaging plus simple (Flatpak, AppImage) et une intégration plus simple avec la liaison Flutter ;
- **aucun code tiers chargé dans le processus qui détient les jetons OAuth** : pour un client de messagerie, ce point n'est pas un détail ;
- des erreurs plus faciles à diagnostiquer.

*Ce que cela enlève :*

- la possibilité d'ajouter un provider sans recompiler ni redistribuer MIRA ;
- une communauté qui écrirait ses propres providers de son côté ;
- l'élégance démonstrative d'une architecture « à la Arcade ».

Ce qui est perdu est faible dans MIRA, justement parce qu'un provider y est surtout de la donnée (voir 4.3).

**Abandonner aussi le niveau B (tout en dur, niveau C).** Cela donnerait le code le plus court possible, mais éparpillerait chaque particularité dans des `match` à travers tout le moteur. Ajouter Yahoo ou iCloud obligerait à modifier le cœur. Le jour où un provider exigerait un **autre protocole** (API de Gmail, Microsoft Graph), il n'y aurait plus d'endroit propre pour le brancher. Et l'on ne pourrait plus tester le moteur avec un faux provider. C'est pour cela que le niveau C a été écarté.

**Une voie intermédiaire, à trois couches,** a été décrite : des **profils déclaratifs** (données) pour tout ce qui est configuration, un petit nombre de **hooks de code compilés** pour les vraies particularités, et un **trait de backend** pour le jour où un provider ne parlerait plus IMAP. Elle donne l'effet « plugin » (on peut ajouter un fournisseur par la donnée) sans exécuter de code tiers. Cette structure a été retenue dans son esprit : c'est la définition d'un provider comme **profil plus particularités** (D-05).

**Décisions D-02 et D-03.** Le porteur a tranché : « il est nécessaire ou bien adéquat de laisser tomber les plugins. Ce qu'on va faire, on va essayer de délimiter une frontière en interne dans le code de l'engine entre le corps et les providers. » Les plugins dynamiques sont abandonnés (D-02), une frontière interne est posée entre le cœur et les providers (D-03), et la porte reste ouverte pour qu'un contributeur futur ajoute un provider par la donnée.

### 4.6 Le moteur d'abord, en ligne de commande

Une autre décision de fond est venue du porteur : le moteur doit pouvoir être utilisé **sans interface**, en ligne de commande d'abord, et l'interface Flutter n'est qu'une interface posée dessus (D-04).

Cette décision a des conséquences structurantes.

- Le moteur devient une **bibliothèque** avec une API publique propre, composée de commandes d'un côté et d'un flux d'événements de l'autre. Le CLI en est le premier client, Flutter le second.
- Tout ce que Flutter pourra faire devra **d'abord exister en commande CLI**, donc être testable sans écran. C'est la meilleure garantie que l'interface n'infiltrera pas de logique.
- Le CLI sert aussi de **banc d'essai** : scénarios de synchronisation, cas d'erreur, hors ligne, scriptables.
- Cela relance la question de savoir si Flutter appellera la bibliothèque directement (liaison en processus, comme le brief le prévoit) ou parlera à un processus moteur qui tourne en arrière-plan. L'API étant la même dans les deux cas, la décision peut être repoussée : elle est notée D-30 (ouverte).

### 4.7 Ce que MIRA retient d'Arcade

Le tableau suivant met en regard les contraintes d'Arcade et leur transposition.

| Contrainte d'Arcade | Transposition dans MIRA |
|---|---|
| Bibliothèques dynamiques chargées à l'exécution | **Abandonné** (D-02) : tout est lié dans un seul exécutable |
| Encapsulation de la bibliothèque C `libdl` | Le **principe** subsiste : toute bibliothèque externe (client IMAP, trousseau, SQLite, analyseur MIME) est encapsulée derrière une interface propre au moteur (voir 5.6) |
| `ldd` ne doit montrer aucune dépendance vers les bibliothèques | Équivalent : **le cœur n'a aucune dépendance vers les providers**, vérifiable en lisant l'arbre de dépendances du projet |
| Interdit de nommer une bibliothèque graphique dans le programme principal | Équivalent : **le cœur ne nomme jamais un provider** (D-06) |
| Les bibliothèques graphiques n'influencent pas la logique de jeu | Équivalent : **l'interface n'influence pas la logique du moteur** (P1) |
| Traitement générique et uniforme des bibliothèques | Conservé tel quel (P3) |
| Règles de build séparées (cœur, jeux, graphiques) | Les trois blocs (cœur, providers, assemblage) se compilent séparément |
| Interfaces partagées avec un autre groupe | Non exigé ici, mais le **contrat d'API** (chapitre 12) joue ce rôle de document d'interface |
| Erreurs sur la sortie d'erreur, code de sortie 84 | Repris pour le CLI : erreurs sur la sortie d'erreur, codes de sortie par catégorie d'erreur (12.8) |
| Documentation « comment ajouter une bibliothèque » | Équivalent : la procédure d'ajout d'un provider (6.9) et ce document |

### 4.8 Chronologie résumée du raisonnement

Pour mémoire, l'enchaînement des idées a été le suivant :

1. Idée de départ : des plugins `.so` par provider, à la manière d'Arcade.
2. Constat de l'absence d'interface binaire stable en Rust, et examen de quatre options.
3. Constat qu'un provider est surtout une différence de données, pas de protocole.
4. Examen du C++, et confirmation de Rust (D-01).
5. Examen des trois niveaux d'abstraction, et abandon des plugins dynamiques (D-02).
6. Décision d'une frontière interne en trois blocs (D-03) et définition d'un provider comme profil plus particularités (D-05).
7. Décision de construire le moteur pour qu'il fonctionne en CLI d'abord (D-04).
8. Réflexion sur les utilisateurs et profils (chapitre 7), puis sur le modèle de données (chapitres 8 et 9), la synchronisation (chapitre 10) et le contrat d'API (chapitre 12).

---

## 5. Architecture générale

### 5.1 Vue d'ensemble

Le moteur se compose de trois blocs logiques, que seul un quatrième élément (l'exécutable) relie. La figure suivante en donne la forme générale.

```
          ┌────────────────┐        ┌────────────────────┐
          │      CLI       │        │  Flutter (plus tard)│
          └───────┬────────┘        └──────────┬─────────┘
                  │  commandes / requêtes / événements  │
                  └───────────────┬─────────────────────┘
                                  ▼
 ┌──────────────────────── BLOC CŒUR ──────────────────────────┐
 │  Contrat d'API                                               │
 │  Comptes · Synchro · File d'actions · Stockage · Événements  │
 │  Clients IMAP/SMTP · Authentification · Trousseau · MIME     │
 │                                                              │
 │  Vocabulaire commun : types du domaine + interface provider  │
 └───────────────────────────────▲──────────────────────────────┘
                                 │ (ne dépend que du vocabulaire)
 ┌──────────────────── BLOC PROVIDERS ─────────────────────────┐
 │   Gmail · Outlook · Yahoo · Autre (générique)                │
 └───────────────────────────────▲──────────────────────────────┘
                                 │
 ┌──────────────────── BLOC ASSEMBLAGE ────────────────────────┐
 │  Construit le registre, y enregistre les providers,          │
 │  démarre le moteur. Seul bloc qui connaît tout le monde.     │
 └──────────────────────────────────────────────────────────────┘
```

### 5.2 Les trois blocs

**Le bloc cœur** contient tout ce qui fait le moteur : le contrat d'API, la gestion des comptes, la synchronisation, la file d'actions, le stockage, le bus d'événements, les clients réseau, l'authentification, l'accès au trousseau et l'analyse MIME. Il définit aussi l'**interface** que doit respecter un provider. Il **ne connaît aucun provider concret**.

**Le bloc providers** contient la description de chaque fournisseur : Gmail, Outlook, Yahoo, et le provider générique « Autre ». Chaque provider respecte l'interface définie par le cœur. Les providers sont **purs** : ils ne font aucune entrée-sortie, ne gardent aucun état, et se contentent de répondre à des questions sur une adresse ou sur une donnée brute (D-07).

**Le bloc assemblage** est un petit module, le **seul** à connaître à la fois le cœur et les providers. Il crée le registre, y enregistre chaque provider, et démarre le moteur avec ce registre. Il est partagé par tous les points d'entrée (CLI, liaison Flutter), de sorte que tous les clients voient exactement les mêmes providers.

### 5.3 Règles de dépendance

La règle d'or, héritée d'Arcade, tient en une phrase : **le cœur ne dépend d'aucun provider**. Le tableau suivant précise qui peut dépendre de quoi.

| Bloc | Peut dépendre de | Ne peut pas dépendre de |
|---|---|---|
| Cœur (moteur) | Vocabulaire commun, bibliothèques externes encapsulées | Providers, assemblage, clients |
| Vocabulaire commun | Rien du projet | Tout le reste |
| Providers | Vocabulaire commun uniquement | Le reste du cœur (stockage, synchro, réseau), assemblage |
| Assemblage | Cœur, providers | Clients |
| Clients (CLI, Flutter) | Assemblage, contrat d'API | Détails internes du cœur |

Cette règle est **appliquée par le compilateur** et non par la discipline : un bloc qui ne déclare pas une dépendance ne peut tout simplement pas utiliser le code de l'autre. C'est l'équivalent de la règle de lien en C++ : une cible CMake qui ne fait pas `target_link_libraries` vers une autre ne peut pas inclure ses en-têtes.

**Précision proposée sur D-03 [PROPOSÉ] (D-33).** Pour que la règle « un provider ne dépend que du vocabulaire » soit réellement mécanique, les types du domaine et l'interface provider doivent former une **petite bibliothèque distincte** à l'intérieur du bloc cœur. Sinon, un provider qui dépendrait de « tout le cœur » pourrait atteindre le stockage ou le réseau, et la protection serait perdue. Le bloc cœur compte donc, en pratique, deux bibliothèques : le vocabulaire commun (petit, stable) et le moteur proprement dit. Cette précision ne change pas les trois blocs validés ; elle les rend vérifiables.

### 5.4 Le point d'assemblage

L'assemblage est volontairement minuscule. Son travail tient en quatre actions : créer un registre dont le **provider de repli** est le provider générique ; y enregistrer Gmail, Outlook, Yahoo ; créer le moteur avec ce registre ; fournir ce moteur aux clients. Il n'y a pas de logique métier dans l'assemblage.

C'est aussi l'endroit où l'on décidera, plus tard, quels providers sont compilés dans quelle version de MIRA (par exemple une version sans Yahoo), sans toucher au cœur. Dans Arcade, c'était le rôle du chargement de dossier `./lib` ; ici, c'est une liste statique, vérifiée à la compilation.

### 5.5 Correspondance entre Rust et C++

Le porteur a demandé si un « crate » correspond à un objet. La réponse est non : c'est plutôt l'équivalent d'une **cible CMake**. Voici la table de correspondance qui sert de fil conducteur.

| Rust | Équivalent C++ |
|---|---|
| **crate** (caisse) | une cible CMake (`add_library` ou `add_executable`), compilée à part |
| **workspace** | le projet CMake qui regroupe toutes les cibles |
| `Cargo.toml` d'une caisse | le `CMakeLists.txt` d'une cible, avec ses `target_link_libraries` |
| **module** | un espace de noms, avec un fichier ou un dossier derrière |
| `pub` ou son absence | ce qui est dans l'en-tête public ou privé à l'implémentation |
| **struct** avec ses méthodes | une classe : données et méthodes |
| **trait** | une classe abstraite pure, comme `IGame` ou `IGfx` |
| `dyn Trait` | un pointeur ou une référence vers cette interface |

Une précision a aussi été apportée : les parties ne sont pas des binaires indépendants, ce sont des **bibliothèques statiques** (l'équivalent de fichiers `.a`). Chaque caisse est compilée séparément en bibliothèque, puis une étape finale les **lie** en un seul exécutable. Il n'y a pas de `.so` à charger à l'exécution, ce qui correspond exactement à ce qu'on voulait en abandonnant les plugins dynamiques.

### 5.6 Découpage interne du cœur [PROPOSÉ] (D-34)

Le porteur n'a pas encore examiné le découpage interne du bloc cœur. Voici une proposition de **sous-domaines**, qui peuvent être des modules au début et devenir des bibliothèques séparées plus tard si le besoin s'en fait sentir.

| Sous-domaine | Responsabilité |
|---|---|
| Vocabulaire commun | Types du domaine, interface provider, erreurs |
| API | Contrat : commandes, requêtes, événements |
| Comptes | Cycle de vie des comptes, identités, états |
| Authentification | Flux OAuth2, mots de passe d'application, rafraîchissement des jetons |
| Trousseau | Encapsulation de SecretService |
| Client IMAP | Connexion, `LIST`, `FETCH`, `IDLE`, extensions |
| Client SMTP | Envoi |
| Analyse MIME | Transformation de la source brute en structure exploitable |
| Stockage | SQLite, recherche en texte intégral, migrations, cache de pièces jointes |
| Synchronisation | Stratégies de détection des changements, descente, file d'actions, reprise |
| Événements | Bus interne, numérotation, abonnements |
| Registre des providers | Enregistrement, détection, repli |

**Principe d'encapsulation, hérité du travail sur `libdl`.** Chaque bibliothèque externe est cachée derrière une interface propre au moteur : le client IMAP, le client SMTP, l'analyseur MIME, le trousseau, SQLite. Pourquoi ? Parce que le remplacement d'une bibliothèque (par exemple passer de l'une à l'autre pour IMAP, ou s'appuyer sur les caisses de Himalaya) ne doit pas se répercuter dans tout le moteur. Et parce que cela permet de substituer de faux serveurs dans les tests. C'est exactement la leçon de l'encapsulation de `dlopen` : on ne laisse pas une API étrangère se répandre dans le code.

### 5.7 Ce que l'architecture interdit

Pour qu'elle reste lisible, voici les interdits, tous vérifiables en revue de code ou par la compilation :

1. Le cœur ne contient aucun nom de provider, aucun test d'identité de provider.
2. Un provider ne fait aucune entrée-sortie : pas de réseau, pas de fichier, pas d'accès à la base.
3. Un provider ne garde aucun état entre deux appels.
4. Un provider ne dépend d'aucune partie du cœur autre que le vocabulaire commun.
5. Le moteur n'a aucune notion d'écran, de fenêtre ou de page.
6. Aucun secret ne figure dans la base, dans les journaux ou dans une réponse de l'API.
7. Aucune bibliothèque externe n'est utilisée directement hors de son module d'encapsulation.
8. Aucune lecture ne passe par le réseau (sauf commande explicite).
9. Aucune écriture ne bloque sur le réseau.
10. Aucune requête n'est faite sans être bornée par un curseur de pagination.

### 5.8 Testabilité

Le découpage est pensé pour que la grande majorité du moteur se teste **sans réseau** :

- le cœur se teste avec de **faux providers** définis dans les tests eux-mêmes, ce qui démontre qu'il ne dépend d'aucun provider réel ;
- les providers se testent avec des **tables de cas** (adresse en entrée, valeur attendue en sortie), puisqu'ils sont purs ;
- la synchronisation se teste avec un **faux serveur IMAP** rejouant des scénarios (renumérotation, suppressions concurrentes, coupures) ;
- le CLI sert de **banc d'essai de bout en bout**, scriptable ;
- le temps (horloge) et les identifiants aléatoires sont injectés, pour obtenir des tests déterministes.

### 5.9 Évolutions prévues

L'architecture doit rendre peu coûteuses les évolutions suivantes :

- **Ajouter un provider** : une description dans le bloc providers et une ligne dans l'assemblage. Rien dans le cœur.
- **Ajouter un backend qui ne parle pas IMAP** (API Gmail, Microsoft Graph) : un second niveau d'interface, au-dessus du protocole, que le cœur utiliserait de la même façon que le backend IMAP. Il n'est pas construit en version 1, mais rien ne doit l'empêcher.
- **Ajouter un client** (autre interface, outil de diagnostic) : il se branche sur le contrat d'API sans toucher au cœur.
- **Séparer le moteur en processus** : si la question D-30 est tranchée dans ce sens, le contrat d'API reste identique, seul le transport change.
- **Ajouter l'agent d'IA** : il s'abonne au bus d'événements et interroge les mêmes requêtes que les autres clients, sans coupler le moteur à lui.

---
## 6. Les providers

### 6.1 Rôle d'un provider

Un provider est la **description d'un fournisseur de messagerie** : comment s'y connecter, comment s'y authentifier, et en quoi il se comporte différemment d'un serveur IMAP/SMTP « standard ». Il ne contient ni réseau, ni stockage, ni synchronisation. Il répond à des questions et traduit des données brutes, rien de plus.

L'équivalent dans Arcade est une bibliothèque graphique ou un jeu : un composant que le cœur manipule uniquement à travers une interface, sans jamais le nommer. La différence, expliquée au chapitre 4, est qu'un provider décrit des **différences** (de données et de particularités) au lieu d'apporter un **comportement complet**.

### 6.2 Ce qui varie d'un provider à l'autre

Il est utile de classer ce qui varie en trois catégories, car chacune se traite autrement.

| Catégorie | Exemples | Traitement |
|---|---|---|
| **Données** | hôtes, ports et chiffrement IMAP/SMTP ; adresses et périmètres OAuth2 ; domaines reconnus ; nom affiché | Décrites par le profil du provider |
| **Particularités** | noms des dossiers spéciaux ; copie automatique dans « Envoyés » ; limites de connexions et de fréquence ; extensions fiables ou non ; libellés et identifiants de fil | Hooks de traduction ou déclarations, en code optionnel |
| **Protocole** | un fournisseur qui n'utiliserait pas IMAP mais une API native | Hors version 1 ; prévu comme un second niveau d'interface (voir 6.10) |

La grande majorité du travail est dans la première catégorie, qui est de la pure donnée. C'est ce qui rend cette architecture légère : ajouter un fournisseur qui se comporte normalement revient à écrire un profil.

### 6.3 Les questions que le cœur pose à un provider

Le cœur ne connaît qu'une interface, composée de questions et de traductions. Voici le contenu de cette interface, présenté sous forme de questions, car c'est ainsi qu'il faut la comprendre.

| Le cœur demande | Le provider répond | Remarque |
|---|---|---|
| Gères-tu cette adresse ? | Oui ou non, d'après le domaine | Le provider générique répond toujours oui |
| Quels sont tes serveurs pour cette adresse ? | Hôtes, ports, chiffrement IMAP et SMTP | Le provider générique les déduit du domaine |
| Quelles méthodes d'authentification acceptes-tu, par ordre de préférence ? | OAuth2 (avec adresses et périmètres) ou mot de passe | Un provider peut accepter les deux |
| Quel est le rôle de ce dossier ? | Réception, envoyés, brouillons, corbeille, spam, archive, « tous les messages », autre | Reçoit le nom brut, le séparateur et les attributs standard |
| Quelles limites s'appliquent ? | Nombre de connexions, délai minimal entre deux relevés | Valeurs par défaut prudentes |
| Quelles exceptions connues as-tu à signaler ? | Extensions peu fiables, copies automatiques dans « Envoyés », etc. | Voir 6.5 |
| Que faire de cette donnée brute propre à ton service ? | Une valeur du modèle de domaine | Exemple : identifiant stable d'un message, libellés d'un message |

La dernière ligne est la plus importante conceptuellement : le provider est une **couche de traduction**. Il convertit les notions propres à un service (libellés Gmail, identifiants de fil Gmail) en notions du modèle de domaine, de sorte que ce dernier ne contienne jamais de concept propre à un fournisseur. Cette idée est connue en conception logicielle sous le nom de couche anti-corruption : elle protège le modèle central des particularités externes.

### 6.4 Profil plus particularités (D-05)

Un provider se définit comme **un profil (des données) plus, au besoin, des particularités (du code)**. Cette définition est le résultat direct de la décision d'abandonner les plugins : elle donne une forme unique à tous les providers.

**Le profil** contient les informations de configuration : nom affiché, domaines reconnus, serveurs, méthodes d'authentification, limites par défaut. C'est de la donnée pure, qui pourrait un jour être lue depuis un fichier de configuration.

**Les particularités** sont un petit nombre de fonctions optionnelles qui traduisent des comportements propres au fournisseur : par exemple la façon de déduire le rôle d'un dossier, ou l'extraction d'un identifiant stable à partir de données brutes. Un provider qui n'a **aucune** particularité (comme « Autre ») se contente de son profil.

**Pourquoi cette forme ?**

- **Uniformité** : Gmail, Outlook, Yahoo et « Autre » sont du même type. Aucun n'est un cas spécial, et le cœur les traite tous de la même manière (P3).
- **Le cas « Autre » en découle naturellement.** L'utilisateur y saisit lui-même ses serveurs. Ces informations sont donc une donnée **stockée avec le compte**, et un compte « Autre » est simplement un compte dont le profil est rempli par l'utilisateur plutôt que par le code (voir 7.9).
- **Extension future par la donnée.** Si un jour on veut permettre d'ajouter un fournisseur sans toucher au code, on pourra charger des profils depuis des fichiers de données, sans exécuter du code tiers.

### 6.5 Capacités : le serveur d'abord, le provider pour les exceptions (D-08)

Une correction importante a été apportée au cours de la conception. Une première esquisse faisait **déclarer** par le provider ses capacités (par exemple « supporte la réception en temps réel »). Cette approche est inversée.

La **source de vérité** est le **serveur lui-même** : à la connexion, un serveur IMAP annonce la liste de ses extensions (CONDSTORE, QRESYNC, IDLE, MOVE, SPECIAL-USE, etc.). Le cœur lit cette liste et choisit ses stratégies en conséquence. Le provider n'intervient que pour signaler des **exceptions connues**, par exemple :

- « ce serveur annonce telle extension mais elle est peu fiable » ;
- « ne dépasse pas tant de connexions simultanées » ;
- « l'envoi par SMTP enregistre déjà une copie dans le dossier des envoyés ».

**Avantages.** Le cœur se dégrade proprement sur n'importe quel serveur, y compris un serveur totalement inconnu servi par le provider « Autre », sans qu'on ait eu besoin de le décrire. Le principe P6 (se dégrader selon les capacités, pas selon l'identité) s'applique ainsi à tout le monde.

**Conséquence sur l'interface.** Le provider ne fournit donc pas un « tableau de capacités » complet, mais une liste d'exceptions et de limites, qui complète ce que le serveur dit de lui-même.

### 6.6 Le registre et la détection d'un provider

Le **registre** est la collection des providers connus du moteur. Il offre trois services : enregistrer un provider, retrouver un provider par son identifiant, et **détecter** le provider le plus adapté à une adresse.

**Détection.** Le registre parcourt les providers dans l'ordre d'enregistrement et retient le premier qui répond « oui » à la question « gères-tu cette adresse ? ». Si aucun ne répond oui, il retourne le **provider de repli**, qui est le provider générique. Un registre a donc toujours une réponse, et une adresse inconnue tombe sur « Autre ».

**Choix manuel.** L'utilisateur doit pouvoir imposer un provider, par exemple parce que l'adresse ne dit rien de l'hébergeur. Le contrat d'API offre donc deux formes de préparation de compte : par détection, et par choix explicite d'un provider.

**Limite connue de la détection par domaine** **[OUVERT]**. Une adresse de type `nom@entreprise.fr` peut être hébergée chez Google ou Microsoft : le domaine ne l'indique pas. Pour ces adresses, deux options existent : demander à l'utilisateur de choisir, ou interroger le DNS (enregistrements MX) pour deviner l'hébergeur. La seconde option implique du réseau, donc elle ne peut pas être dans un provider pur ; elle serait un **service du cœur** qui précède la détection. Cette question n'a pas été tranchée.

### 6.7 Catalogue initial

Le catalogue de la version 1 comprend quatre providers. **Les valeurs ci-dessous viennent de la connaissance générale des documentations publiques des fournisseurs, pas d'une vérification en ligne pendant la conception. Elles doivent être vérifiées avant toute utilisation**, en particulier les adresses et périmètres OAuth2, qui changent.

| Provider | Domaines reconnus | IMAP | SMTP | Authentification |
|---|---|---|---|---|
| **Google (Gmail)** | gmail.com, googlemail.com | imap.gmail.com, 993, TLS | smtp.gmail.com, 465, TLS | OAuth2 de préférence ; mot de passe d'application possible |
| **Microsoft (Outlook)** | outlook.com, hotmail.com, live.com, msn.com et variantes régionales | outlook.office365.com, 993, TLS | smtp.office365.com, 587, STARTTLS | OAuth2 uniquement |
| **Yahoo Mail** | yahoo.com, ymail.com, rocketmail.com et variantes régionales | imap.mail.yahoo.com, 993, TLS | smtp.mail.yahoo.com, 465, TLS | Mot de passe d'application |
| **Autre (générique)** | tout domaine, en repli | déduit du domaine ou saisi, 993, TLS | déduit du domaine ou saisi, 465, TLS | Mot de passe |

Microsoft a, d'après les informations générales disponibles lors de la conception, désactivé l'authentification par mot de passe simple pour ses comptes : OAuth2 y est donc la seule voie. Cette information est à confirmer au moment de l'implémentation.

### 6.8 Particularités connues et leur traduction

Cette section recense les particularités déjà identifiées, avec la manière dont chacune se traduit dans l'architecture.

**Noms de dossiers spéciaux.** Selon le fournisseur, le dossier des messages envoyés, de la corbeille ou de l'archive porte un nom différent, parfois traduit dans la langue du compte. La norme IMAP prévoit des **attributs standard** (extension SPECIAL-USE) qui marquent ces dossiers indépendamment de leur nom. Le comportement par défaut est de se fier à ces attributs, puis aux noms usuels ; le provider ne surcharge ce comportement que lorsque le service s'en écarte. Pour Gmail, par exemple, certains dossiers portent un nom préfixé et leur rôle doit être déduit de ce nom.

**Libellés (Gmail).** Gmail attribue des libellés à un message, et expose chaque libellé comme une boîte IMAP. Un même message apparaît donc dans plusieurs boîtes. Le provider doit fournir, à partir des données brutes, l'**identifiant stable** qui permet de reconnaître que ces apparitions sont le même message (voir 9.5), et les libellés eux-mêmes si le serveur les renvoie.

**Identifiants de fil.** Gmail fournit aussi un identifiant de fil, qui peut servir d'**indice** pour le calcul des fils (D-15), sans remplacer l'algorithme de MIRA.

**Copie dans « Envoyés ».** Certains serveurs enregistrent eux-mêmes une copie d'un message envoyé par SMTP ; d'autres non. Si MIRA ajoute sa propre copie dans le dossier des envoyés alors que le serveur l'a déjà fait, il y aura un doublon. D'après le souvenir de la conception, Gmail enregistre sa copie automatiquement, **point à vérifier**. Le provider doit donc déclarer si le serveur enregistre lui-même la copie, et le cœur adapte la procédure d'envoi (voir 10.8).

**Limites de connexions et de fréquence.** Chaque service tolère un nombre limité de connexions simultanées et peut sanctionner les relevés trop fréquents. Le provider fournit des **plafonds prudents** ; le cœur s'y plie. Les valeurs exactes sont à vérifier pour chaque service.

**Fiabilité de la réception en temps réel.** Certains serveurs annoncent la réception en temps réel (IDLE) mais la coupent ou la rendent peu fiable. Le provider peut le signaler, et le cœur retombe alors sur un relevé périodique.

**Authentification.** OAuth2 avec ses adresses d'autorisation et de jeton et ses périmètres, ou mot de passe d'application. Les périmètres OAuth2 appartiennent au provider ; l'**identifiant de l'application MIRA** auprès de Google ou Microsoft (le *client id*) appartient, lui, à la configuration de l'application et pas au provider.

### 6.9 Procédure d'ajout d'un provider

L'équivalent de la documentation « comment ajouter une bibliothèque » d'Arcade tient ici en quelques étapes conceptuelles.

1. **Décrire le profil** : nom, domaines reconnus, serveurs IMAP et SMTP, méthodes d'authentification, limites.
2. **Identifier les particularités** : dossiers spéciaux, copie des envoyés, identifiants stables, indices de fil, extensions peu fiables.
3. **Implémenter uniquement ce qui diffère** du comportement standard ; tout le reste utilise les valeurs par défaut.
4. **Écrire les tests** par table de cas : adresses reconnues ou non, serveurs attendus, rôles de dossiers pour une liste de noms réels.
5. **L'enregistrer dans l'assemblage** : une ligne.
6. **Valider contre un compte réel** : connexion, première synchronisation, envoi, déplacement, suppression, avec le CLI.
7. **Documenter** les particularités découvertes.

Le point à retenir : à aucun moment on ne modifie le cœur.

### 6.10 Évolution : les backends qui ne parlent pas IMAP

Un jour, un provider pourrait offrir une API native plus riche qu'IMAP (libellés natifs, notifications, requêtes plus puissantes). L'architecture ne doit pas l'interdire, mais elle ne le construit pas en version 1.

La façon de le prévoir est un **second niveau d'interface**, placé entre le moteur et le protocole : une interface de « backend de messagerie » qui offre des opérations de haut niveau (lister les boîtes, synchroniser une boîte depuis un repère, récupérer un message, modifier des drapeaux, envoyer, écouter les changements). Le backend IMAP/SMTP en serait l'implémentation par défaut, utilisée par tous les providers, et un provider particulier pourrait en fournir une autre.

Cette interface n'est **pas définie maintenant**. Ce qu'on garde en tête, c'est que la synchronisation (chapitre 10) soit écrite contre des opérations de haut niveau plutôt que contre des commandes IMAP brutes partout, de façon à ne pas fermer cette porte.

### 6.11 Tests des providers

Parce que les providers sont purs, leurs tests sont de simples tables : une entrée, une sortie attendue.

- **Reconnaissance** : une liste d'adresses avec le provider attendu pour chacune, y compris des cas pièges (majuscules, sous-domaines, adresses invalides).
- **Serveurs** : la configuration attendue pour une adresse.
- **Rôles de dossiers** : une liste de noms bruts et d'attributs, avec le rôle attendu.
- **Traductions de données brutes** : identifiant stable, libellés.
- **Limites** : valeurs plafonds.

Des tests de **non-régression par comptes réels** viennent s'y ajouter, mais ils sont manuels ou facultatifs, car ils dépendent du réseau et de comptes personnels.

### 6.12 Récapitulatif des décisions de ce chapitre

| ID | Décision | Statut |
|---|---|---|
| D-05 | Provider = profil plus particularités | DÉCIDÉ |
| D-06 | Le cœur ne nomme jamais un provider | DÉCIDÉ |
| D-07 | Providers purs (aucune entrée-sortie, aucun état) | PROPOSÉ |
| D-08 | Capacités : le serveur d'abord, le provider pour les exceptions | PROPOSÉ |
| (ouvert) | Détection de l'hébergeur par DNS pour les domaines personnalisés | OUVERT |

---

## 7. Profils, comptes et identités

### 7.1 La hiérarchie

La question posée par le porteur était : quelle dépendance entre les utilisateurs et les providers, sachant que l'application pourrait avoir plusieurs utilisateurs, chacun reliant un ou plusieurs comptes de plusieurs providers ? La réponse tient dans une hiérarchie simple.

```
Profil ──1:N──> Compte ──N:1──> Provider (catalogue)
                  │
                  ├── Identités (adresses d'envoi)
                  ├── Boîtes, messages, état de synchro (propres au compte)
                  └── Secrets (trousseau, par profil et par compte)
```

**Un utilisateur ne dépend pas d'un provider ; c'est le compte qui en dépend.** Le provider est un **modèle** et le compte en est une **instance**. Dans le vocabulaire C++, le provider est du code global et sans état (une implémentation unique et partagée d'une interface), tandis que le compte est un objet qui porte un état.

### 7.2 Pas de notion d'utilisateur applicatif (D-09)

Le brief décrit un client Linux natif. Or, sur Linux, le trousseau, les dossiers de données et les notifications sont déjà séparés **par session système** : deux personnes qui partagent un ordinateur ont chacune leur session, donc leur propre MIRA, sans que l'application gère quoi que ce soit.

Trois lectures de « plusieurs utilisateurs » étaient possibles :

1. **Pas de notion d'utilisateur dans MIRA** : un profil par session système, avec les comptes tout en haut. C'est la plus simple et la plus conforme au brief.
2. **Des profils dans l'application** (travail, personnel, séparés, chacun avec sa boîte unifiée) : utile pour cloisonner dans une même session.
3. **Un service multi-utilisateurs** (serveur, comptes applicatifs) : ce serait un autre projet.

**Décision D-09 :** le porteur a retenu la première lecture (« l'option 1 me parle très bien ») et a indiqué que la deuxième était une piste utile à garder.

### 7.3 La porte ouverte aux profils (D-10)

Comme le porteur voulait explorer l'option 2 plus tard, la conception pose dès maintenant **trois règles** qui rendent son ajout peu coûteux. Sans elles, ajouter des profils plus tard serait une migration douloureuse.

1. **Un moteur est ouvert sur un seul profil.** Le profil est le contexte avec lequel on construit le moteur, comme un objet C++ construit avec sa configuration. Aujourd'hui il n'existe qu'un profil par défaut, implicite. Plus tard, on choisira le profil au démarrage (par exemple, en CLI, par une option de sélection de profil).
2. **Chaque profil a son propre espace de données** : son dossier, sa base SQLite, son index de recherche. L'alternative (une base unique avec une colonne « profil » dans chaque table) a été écartée : un oubli de filtre dans une requête ferait fuiter les messages d'un profil dans un autre, alors qu'un espace par profil isole par construction. Il rend aussi triviales la suppression, la sauvegarde et l'export d'un profil.
3. **Les secrets sont rangés par profil et par compte** dans le trousseau, avec l'identifiant du profil dans la clé. Deux profils peuvent contenir le même compte Gmail sans que leurs jetons se mélangent.

**Ce qu'on ne fait pas maintenant** : ni sélecteur de profils dans l'interface, ni plusieurs profils actifs en même temps, ni changement de profil à chaud. Seules les trois règles ci-dessus sont posées, parce que ce sont les seules difficiles à changer après coup.

### 7.4 Réglages d'application et réglages de profil (D-11) [PROPOSÉ]

Il faut distinguer deux niveaux de réglages :

- **Réglages d'application**, partagés par tous les profils : thème, langue, comportement général.
- **Réglages de profil**, propres à chaque profil : comptes, signatures, règles, raccourcis.

Cette distinction est facile à décider tôt, pénible à démêler tard. Elle n'a pas été validée explicitement et reste donc proposée.

### 7.5 Le compte

Un compte est la **connexion d'un profil à un serveur de messagerie**. Il regroupe :

- l'**adresse** principale et le nom affiché ;
- l'**identifiant du provider** (une simple clé du catalogue, par exemple « gmail ») ;
- la **méthode d'authentification** retenue (OAuth2 ou mot de passe) et une **référence** vers le secret correspondant dans le trousseau ;
- pour un compte « Autre », la **configuration des serveurs** saisie par l'utilisateur ;
- les **réglages de synchronisation** (fenêtre de temps, boîtes à synchroniser) ;
- l'**état** du compte.

**États d'un compte.** Un compte est dans l'un des états suivants : *connecté*, *hors ligne*, *authentification requise* (jeton expiré ou révoqué), *en pause* (voulu par l'utilisateur) ou *en erreur* (problème persistant, avec sa cause). Ces états sont émis comme événements pour que les clients puissent les afficher.

**Cycle de vie.** Un compte passe par quelques étapes : *préparation* (détection du provider, présentation des méthodes d'authentification), *test de connexion* (sans rien enregistrer), *ajout* (enregistrement du compte et du secret), *première synchronisation*, vie normale, puis *suppression*. La suppression doit effacer les données locales du compte **et** ses secrets du trousseau, sans toucher au serveur.

### 7.6 Compte et identité

Il convient de distinguer le **compte** (la connexion au serveur) des **identités d'envoi** (les adresses avec lesquelles on peut écrire). Un compte Gmail peut avoir plusieurs alias, et la notion de « de la part de » ne doit donc pas être figée sur le compte.

Une **identité** comprend une adresse d'envoi, un nom affiché, une signature, et le fait d'être l'identité par défaut du compte. Un compte a **au moins une identité** : celle de son adresse principale. Un brouillon référence une identité, pas un compte directement.

### 7.7 Les secrets (P9)

Les mots de passe d'application, les jetons d'accès et les jetons de rafraîchissement sont des **secrets**. Ils sont rangés **uniquement** dans le trousseau du système, via l'interface standard SecretService, et jamais ailleurs. Le compte ne stocke que la **référence** qui permet de les retrouver.

Règles associées :

- la clé de rangement inclut l'identifiant du **profil** et celui du **compte**, ce qui garantit l'isolation (D-10) ;
- aucun secret n'apparaît dans les journaux ni dans les réponses de l'API ;
- si le trousseau est indisponible ou verrouillé, le moteur renvoie une erreur de catégorie « secrets » et **ne se rabat jamais** sur un stockage en clair ;
- la suppression d'un compte supprime ses secrets ;
- le rafraîchissement d'un jeton expiré est automatique et silencieux tant qu'il réussit ; s'il échoue, le compte passe à l'état *authentification requise*.

### 7.8 Le provider référencé par identifiant

Le compte stocke seulement l'identifiant de son provider. Au démarrage, le moteur retrouve le provider correspondant dans le registre. Deux cas particuliers doivent être prévus.

- **Deux comptes du même provider** (deux comptes Gmail) partagent le même provider, qui est sans état, mais ont chacun leurs jetons, leurs boîtes, leurs messages et leur état de synchronisation.
- **Provider introuvable** (par exemple après le retrait d'un provider d'une version de MIRA) : le compte passe en erreur avec une cause explicite, et **ses données locales sont conservées**. Aucune perte n'est tolérée pour cause de version.

### 7.9 Le cas « Autre »

Le provider « Autre » n'a pas de serveurs fixes : c'est l'utilisateur qui les saisit. Ces informations (hôtes, ports, chiffrement, éventuellement méthode d'authentification) sont donc une **donnée du compte**, stockée avec lui, et non une donnée du provider. Le provider générique propose des valeurs initiales déduites du domaine (`imap.<domaine>`, `smtp.<domaine>`), que l'utilisateur peut corriger. Un test de connexion, sans enregistrement, permet de valider la configuration avant l'ajout.

### 7.10 La boîte unifiée

La boîte de réception unifiée est une **vue calculée** qui agrège les messages des boîtes de réception de tous les comptes d'un **même profil**. Elle n'est pas une entité à part : c'est une requête. Elle ne traverse jamais les profils, puisque chaque profil a son propre espace de données.

### 7.11 Question ouverte : le parallélisme entre profils

Si l'option 2 (profils multiples) est un jour retenue, il faudra décider si plusieurs profils peuvent **synchroniser et notifier en parallèle**, ou si un seul est actif à la fois. Cela influence la concurrence et l'intégration au bureau (notifications d'un profil non affiché). Elle est volontairement laissée de côté tant que l'option 2 n'est pas retenue.

---
## 8. Le modèle de domaine

Ce chapitre décrit les **entités** que le moteur manipule et leurs relations. C'est le socle de tout le reste : la synchronisation, le stockage et le contrat d'API s'écrivent en termes de ces entités. Les descriptions sont conceptuelles : elles indiquent ce que chaque entité représente et porte, sans fixer la forme des tables ni du code.

### 8.1 Les trois familles de données (D-18)

La séparation la plus utile du modèle est celle de la **source de vérité**. Les données se rangent en trois familles, qui ne se comportent pas du tout de la même manière.

| Famille | Contenu | Source de vérité | Peut-on la jeter ? |
|---|---|---|---|
| **Configuration** | Profil, compte, identité, vues calculées enregistrées | L'utilisateur, qui l'a saisie | **Non** : irremplaçable |
| **Miroir du serveur** | Boîtes, messages, apparitions, pièces jointes, fils | Le serveur | **Oui** : on peut tout reconstruire par resynchronisation |
| **Intentions locales** | Brouillons, actions en attente | Le moteur local, nulle part ailleurs | **Non** : les perdre, c'est perdre un travail |

Cette séparation a des conséquences concrètes, qui justifient de la traiter comme une décision d'architecture et non comme un simple classement :

- elle dicte ce qu'on **sauvegarde** : la configuration et les intentions, jamais le miroir, qui se reconstruit ;
- elle dicte ce qu'on peut **reconstruire** après un bogue de synchronisation : jeter le miroir d'un compte et resynchroniser est toujours une réparation légitime ;
- elle dicte ce qu'une **migration de base** a le droit de toucher et avec quelle prudence : le miroir peut être migré en le jetant, la configuration et les intentions doivent l'être sans perte ;
- elle clarifie la **résolution des conflits** : le serveur gagne sur le miroir (D-23), mais jamais sur les intentions, qui sont rejouées par-dessus.

### 8.2 Les entités (D-19)

#### Profil

L'espace isolé d'un utilisateur de MIRA. Il possède son dossier de données, sa base, son index de recherche et son espace dans le trousseau. Aujourd'hui, un seul profil implicite existe (D-09, D-10). Il porte les réglages de profil. Tout le reste en dépend.

#### Compte

La connexion d'un profil à un serveur de messagerie. Il porte l'adresse principale, l'identifiant du provider, la méthode d'authentification et la référence au secret, la configuration des serveurs pour un compte « Autre », les réglages de synchronisation et un **état** (connecté, hors ligne, authentification requise, en pause, en erreur). Il possède des identités, des boîtes, des messages, des brouillons et des actions en attente. Il est l'unité d'isolation : une tâche de synchronisation par compte, des secrets par compte, des limites de débit par compte.

#### Identité

Une adresse d'envoi d'un compte : adresse, nom affiché, signature, indicateur d'identité par défaut. Un compte a au moins une identité. Un brouillon référence une identité.

#### Boîte

Une **vue d'appartenance réelle** sur les messages d'un compte, qui correspond à un dossier ou à un libellé côté serveur. Elle porte son nom, sa position dans la hiérarchie, son **rôle** (réception, envoyés, brouillons, corbeille, spam, archive, « tous les messages », autre), un indicateur de synchronisation voulue, et son **état de synchronisation** : le repère nécessaire pour savoir ce qui a changé côté serveur (numéro de validité, dernier numéro vu, compteur de modifications quand le serveur en offre un). Une boîte appartient à un compte.

#### Message

L'entité centrale. Un message existe **une seule fois** dans la base, quel que soit le nombre de boîtes où il apparaît (D-12). Il porte son **identifiant interne MIRA** (D-13), son compte, son expéditeur, ses destinataires (À, Copie, Copie cachée), son objet, ses dates, ses en-têtes de rattachement à un fil (identifiant du message, réponse à, références), ses **drapeaux utilisateur** (lu, suivi, répondu, transféré), sa taille, un aperçu de texte, un indicateur de présence de pièces jointes, les informations de désabonnement s'il y en a, et l'**état de son contenu** : en-tête seul, corps présent. La source brute et l'analyse sont conservées séparément (D-16, chapitre 9).

#### Apparition

Le lien entre un message et une boîte : « ce message est dans cette boîte, avec ce numéro côté serveur ». C'est l'entité qui permet de relier le monde du serveur (où tout est numéroté boîte par boîte) au monde du domaine (où le message est unique). Elle porte la boîte, le message, le **numéro serveur**, et d'éventuels indicateurs internes (par exemple le drapeau « supprimé » d'IMAP, qui n'est pas un drapeau utilisateur). Un message a au moins une apparition tant qu'il n'a pas été supprimé définitivement.

#### Pièce jointe

Une partie d'un message qui est un fichier ou une ressource incorporée : nom, type, taille, identifiant de contenu pour les ressources incorporées, et **état de cache** (non téléchargée, en cache avec son emplacement). Elle appartient à un message et n'est téléchargée qu'à la demande (D-16).

#### Fil

Un regroupement de messages d'une même conversation, au sein d'un compte. Il est **dérivé** : calculé par MIRA à partir des en-têtes de réponse, avec un indice du serveur quand il existe (D-15). Comme il se déduit des messages, on peut le recalculer à tout moment sans rien perdre. Il ne traverse pas les comptes.

#### Brouillon

Un message en cours de rédaction, **local**. Il porte l'identité d'envoi choisie, les destinataires, l'objet, le corps, les pièces jointes, le message auquel il répond le cas échéant, et son état (en cours d'édition, planifié, en cours d'envoi). Une copie peut être enregistrée côté serveur dans la boîte des brouillons, et elle apparaît alors comme un message ordinaire du miroir. Le brouillon lui-même, objet d'édition, est une intention locale.

#### Action en attente

Une opération voulue, déjà appliquée en local mais pas encore confirmée par le serveur : marquer lu, déplacer, ajouter ou retirer d'une boîte, mettre à la corbeille, supprimer, enregistrer un brouillon, envoyer. Elle porte son compte, sa cible (messages concernés), sa **clé d'idempotence**, son état (en attente, en cours, réussie, échouée, annulée), son nombre de tentatives, son dernier motif d'échec, et sa date de création. L'ensemble de ces actions forme la **file d'actions** du compte, qui est aussi l'**outbox** : un envoi est une action comme une autre, et l'envoi différé ou l'annulation d'envoi sont des délais appliqués dans cette file.

#### Vue calculée

La définition d'une requête sur les messages : boîte unifiée, non lus, suivis, recherche enregistrée. Elle ne contient pas de messages ; elle est évaluée à la demande sur la base locale. Seules les vues **enregistrées** par l'utilisateur sont des données de configuration ; les vues standard (unifiée, non lus) sont définies par le moteur.

#### Contacts (plus tard)

Un carnet d'adresses pour l'autocomplétion, alimenté automatiquement par les adresses rencontrées. Il n'est pas dans la version 1.

### 8.3 Les relations

La figure suivante rassemble les entités et leurs cardinalités.

```
Profil ─1:N─> Compte ─1:N─> Identité
                 │
                 ├─1:N─> Boîte ◄──N:N──► Message   (via Apparition)
                 │                          │
                 ├─1:N─> Message ───────────┤
                 │          ├─1:N─> Pièce jointe
                 │          └─N:1─> Fil
                 ├─1:N─> Brouillon ─N:1─> Identité
                 └─1:N─> Action en attente
```

| Relation | Cardinalité | Remarque |
|---|---|---|
| Profil → Compte | 1 vers N | Un compte appartient à un seul profil |
| Compte → Identité | 1 vers N, au moins 1 | |
| Compte → Boîte | 1 vers N | |
| Compte → Message | 1 vers N | Un message appartient à un seul compte |
| Message ↔ Boîte | N vers N, via Apparition | Le cœur du modèle (chapitre 9) |
| Message → Pièce jointe | 1 vers N | |
| Message → Fil | N vers 1 | Dérivé |
| Compte → Brouillon | 1 vers N | Un brouillon référence une identité |
| Compte → Action | 1 vers N | |

### 8.4 Ce qui peut se reconstruire

Quand on décide de reconstruire un compte, on conserve la configuration et les intentions, et l'on rejette le miroir : boîtes, messages, apparitions, pièces jointes en cache, fils. La resynchronisation repart du serveur, dans la fenêtre voulue (D-21). Cette opération est sûre à condition que les **actions en attente** soient conservées, puisqu'elles sont la seule trace de ce que l'utilisateur a fait hors ligne.

Un détail important : les **identifiants internes** des messages changent lors d'une reconstruction complète, puisque ce sont des identifiants locaux. Toute chose qui les référencerait (par exemple une vue enregistrée qui mentionnerait un message précis, ou une action en attente visant un message) doit être résolue avant de jeter le miroir. C'est une raison de plus pour qu'une action en attente porte, en plus de l'identifiant interne, assez d'information pour retrouver sa cible côté serveur.

### 8.5 Identifiants dans le domaine

Chaque entité reçoit un **identifiant interne** attribué par le moteur. Ces identifiants sont **opaques** pour les clients : ils ne signifient rien, ne s'interprètent pas, ne se construisent pas. Pour le message, la correspondance avec les identifiants du serveur est tenue à part (voir 9.5), de façon à ce que les clients et la base n'en dépendent jamais.

### 8.6 États à suivre

Quelques entités ont un cycle de vie qui compte.

- **Message** : *en-tête seul* → *corps présent* ; avec en parallèle l'état de chaque pièce jointe (*non téléchargée* → *en cache*). Un message peut aussi devenir *supprimé côté serveur* (retiré du miroir à la prochaine synchronisation).
- **Action en attente** : *en attente* → *en cours* → *réussie*, ou *échouée* (erreur définitive, signalée à l'utilisateur), ou *annulée*. Une erreur passagère renvoie l'action à *en attente* avec un compteur de tentatives.
- **Compte** : les états décrits au chapitre 7.
- **Brouillon** : *en cours d'édition* → *planifié* ou *en cours d'envoi* → *envoyé* (le brouillon disparaît alors, remplacé par le message envoyé dans la boîte des envoyés).

---

## 9. Messages, boîtes et vues

Ce chapitre explique le choix le plus structurant du modèle de données : un message existe une fois, et les boîtes en sont des vues. Il détaille ensuite les décisions qui s'y rattachent : identité, drapeaux, fils et stockage.

### 9.1 Le problème : dossiers contre libellés

Les messageries se divisent en deux familles.

**Les serveurs classiques** (Yahoo, Outlook, la plupart des autres) fonctionnent comme des tiroirs : un message est dans **un seul dossier** à la fois. Pour le classer ailleurs, on le déplace, et il quitte le premier dossier.

**Gmail** fonctionne autrement. Il utilise des **libellés** : un même message peut porter « Travail » et « Factures » en même temps. En IMAP, Gmail présente chaque libellé comme un dossier, donc le même message apparaît **dans plusieurs dossiers à la fois** (et aussi dans « Tous les messages »). C'est un seul message, vu à plusieurs endroits.

La question de modélisation est donc : comment représente-t-on un message qui peut apparaître à plusieurs endroits, de façon à fonctionner aussi pour les serveurs où il n'apparaît qu'à un seul ?

### 9.2 Les trois options et la décision (D-12)

| Option | Idée | Problème |
|---|---|---|
| **A** | Un message est dans un seul dossier | Un message Gmail à deux libellés devient deux messages : doublons dans la boîte unifiée, et supprimer l'un laisse l'autre |
| **B** | Un message existe une fois et appartient à un ou plusieurs dossiers | Un peu plus de travail, mais cohérent partout |
| **C** | Un dossier, plus des étiquettes à part | Deux concepts à maintenir pour dire presque la même chose |

Le porteur a décrit lui-même la bonne direction : « soit on veut qu'un message apparaisse une seule fois dans une boîte, ou bien qu'il apparaisse deux fois dans deux boîtes différentes, ou bien les deux boîtes sont des vues et il n'y a qu'un seul message. » C'est l'option B, formulée en termes de vues. Elle a été adoptée.

**Décision D-12** : un message est une **entité unique**. Les boîtes sont des **vues d'appartenance** sur les messages. Les serveurs classiques sont le cas particulier où un message n'appartient qu'à une seule boîte. L'interface continue de parler de « dossiers ».

Cette décision a été comparée avec la solution « dossiers seuls », examinée à la demande du porteur. Les dossiers seuls apporteraient un seul concept et une synchronisation plus simple, mais feraient de Gmail un citoyen de seconde zone (doublons ou perte d'information sur les libellés) et obligeraient à une migration de schéma pénible le jour où l'on voudrait plusieurs emplacements. Le passage d'un modèle à emplacements multiples vers un modèle à emplacement unique est facile (c'est un cas particulier) ; le sens inverse ne l'est pas. Cette asymétrie penche en faveur du modèle général.

### 9.3 Boîtes réelles et vues calculées

Le mot « vue » recouvre deux notions, qu'il faut distinguer.

- **Les boîtes réelles** (réception, envoyés, un dossier ou un libellé de l'utilisateur) : un message en fait partie parce que **le serveur le dit**. C'est un fait enregistré, et le modifier (déplacer, ajouter un libellé) déclenche une opération côté serveur.
- **Les vues calculées** (boîte unifiée, recherche, non lus, vues intelligentes) : une simple **requête** sur les messages, sans rien à écrire sur le serveur.

Les deux s'affichent de la même manière dans l'interface, mais elles ne se comportent pas pareil à l'écriture. Dans le code, la première sorte s'appelle **boîte** : c'est d'ailleurs le mot de la spécification IMAP (*mailbox*), et cela évite la confusion avec les « vues » (widgets) de Flutter.

### 9.4 Les apparitions

Pour que le modèle tienne, le moteur retient pour chaque message ses **apparitions** : dans quelle boîte il est, avec quel numéro côté serveur. Ce niveau est nécessaire parce qu'IMAP travaille boîte par boîte, avec une numérotation propre à chaque boîte. Le modèle a donc **deux niveaux** :

- un **niveau synchronisation**, qui reste fidèle au serveur : des apparitions numérotées, boîte par boîte ;
- un **niveau domaine**, que voit l'utilisateur : un message unique, dont les boîtes se déduisent des apparitions.

Le message est la vérité du domaine ; les apparitions sont le reflet du serveur.

### 9.5 L'identité d'un message (D-13)

#### Pourquoi une identité propre

Il faut pouvoir dire « c'est le même message » dans quatre situations :

1. le message est dans plusieurs boîtes (le cas Gmail) ;
2. il a été déplacé, et son numéro côté serveur a changé ;
3. le serveur a **renuméroté** une boîte (cela arrive, c'est le rôle du numéro de validité, `UIDVALIDITY`) ;
4. on compare la copie locale à celle du serveur.

Une hypothèse initiale était que tous les serveurs donnent à chaque message un identifiant unique. Ce n'est pas le cas.

#### Les identifiants possibles et leurs limites

| Identifiant | Limite |
|---|---|
| **Numéro serveur (UID)** | Unique dans **une seule boîte**, change au déplacement, peut être remis à zéro par le serveur |
| **En-tête `Message-ID`** | Posé par l'expéditeur : parfois absent, parfois dupliqué (un bogue d'envoi, ou le même courrier reçu deux fois), parfois modifié |
| **Identifiant stable du serveur** (par exemple l'identifiant de message de Gmail, ou l'identifiant d'objet de l'extension OBJECTID) | Fiable, mais tous les serveurs ne le fournissent pas |

#### La stratégie retenue

**MIRA attribue son propre identifiant interne à chaque message**, que l'interface et la base utilisent partout. À côté, une **table de correspondance** mémorise, pour chaque compte, les identifiants du serveur qui lui correspondent. Pour reconnaître un message, le moteur essaie dans l'ordre :

1. l'**identifiant stable du serveur**, s'il existe (c'est le provider qui l'extrait des données brutes) ;
2. sinon, le **numéro dans la même boîte**, tant que le numéro de validité n'a pas changé ;
3. en dernier recours, un **recoupement** : `Message-ID`, date et taille.

**Règle de prudence (P7)** : quand on hésite, on **préfère afficher un doublon plutôt que fusionner à tort deux messages distincts**. Un doublon est gênant mais récupérable ; une fusion erronée perd un message sans le dire.

L'identifiant interne ne doit jamais changer pour une raison côté serveur (renumérotation, déplacement) : c'est tout son intérêt, comme une clé primaire par rapport à une clé naturelle fragile.

### 9.6 Les drapeaux (D-14)

Un **drapeau** est une petite information attachée à un message : *lu*, *suivi* (l'étoile), *répondu*, *transféré*. Techniquement, en IMAP, ils sont attachés à chaque **apparition** d'un message dans une boîte, pas au message lui-même. Pour un message Gmail présent dans « Réception » et « Travail », il y a donc deux apparitions, donc potentiellement deux jeux de drapeaux. Le comportement de Gmail à cet égard (drapeaux partagés ou non entre apparitions) n'a pas été vérifié pendant la conception : on ne s'appuie pas dessus.

**Décision D-14** : les **drapeaux visibles par l'utilisateur sont au niveau du message**. Quand l'utilisateur marque « lu », MIRA l'applique à toutes les apparitions du message par des actions en attente. En cas de désaccord observé à la synchronisation, une règle simple tranche (par exemple, le changement le plus récent l'emporte).

**Exception** : le drapeau « supprimé » d'IMAP n'est pas un drapeau utilisateur, c'est le mécanisme par lequel une apparition est retirée d'une boîte. Il reste donc **interne à l'apparition**.

### 9.7 Les fils (D-15)

Un fil regroupe les messages d'une même conversation. MIRA les **calcule lui-même**, à partir des en-têtes de réponse (identifiant du message, « réponse à », références), avec **le même algorithme pour tous les providers**, ce qui garantit un comportement identique partout. Un **repli** sur l'objet normalisé peut compléter l'algorithme pour les messages dont les en-têtes sont absents. Quand le serveur fournit un identifiant de fil (Gmail en fournit un, d'autres le proposent via une extension), il sert d'**indice**, pas de vérité.

Comme un fil se déduit des messages, il est **dérivé** : on peut le recalculer à tout moment sans perte, ce qui simplifie les corrections d'algorithme et les migrations.

### 9.8 Le stockage d'un message (D-16)

#### Source brute et analyse

Un mail brut est le texte exact que le serveur envoie, en-têtes compris. À partir de lui, MIRA extrait ce qui est utile : objet, texte, HTML, liste des pièces jointes. Deux options :

- **garder la source brute**, et en tirer le reste (que l'on peut aussi mémoriser pour aller vite) ;
- **ne garder que le résultat de l'analyse**.

Analogie en C++ : la source brute est le fichier `.cpp`, l'analyse est l'objet compilé. Si on jette le `.cpp`, on ne peut plus recompiler quand on corrige un bogue ou qu'on ajoute une fonction. Garder la source permet :

- de **ré-analyser** un message après une correction du parseur ;
- d'afficher « voir la source » ;
- de lire des en-têtes qu'on n'avait pas prévus (par exemple ceux du désabonnement en un clic) ;
- de **vérifier les signatures PGP et S/MIME**, qui exigent les octets exacts du message.

Le prix est l'espace disque.

**Décision D-16** : on **garde la source brute, compressée**. L'analyse (texte, HTML, métadonnées pour l'affichage et la recherche) est un **cache reconstruisible** stocké à côté. Les **grosses pièces jointes ne sont pas stockées** tant que l'utilisateur ne les ouvre pas. En parallèle, le téléchargement se fait par étapes : en-têtes d'abord, corps plus tard, pièces jointes à la demande (D-20).

Dans les termes des familles de données (8.1), la source brute est la copie locale de la vérité du serveur ; l'analyse est dérivée.

### 9.9 Les actions et leur sens

Dans un modèle de vues, « supprimer » n'a plus un seul sens. Il faut distinguer trois actions :

- **retirer de cette boîte** : le message quitte cette vue d'appartenance ;
- **mettre à la corbeille** : il rejoint la boîte de rôle « corbeille » ;
- **supprimer définitivement** : il disparaît du serveur.

Chez Gmail, retirer un libellé ne supprime rien ; dans un dossier classique, retirer un message de son seul dossier revient à le déplacer ailleurs. Le domaine parle donc en termes d'actions (**archiver, mettre à la corbeille, déplacer**), et **c'est le provider qui traduit chaque action en opération côté serveur**. Par exemple, « archiver » signifie retirer le libellé de la boîte de réception chez Gmail, et déplacer vers un dossier d'archive ailleurs.

Les **rôles spéciaux** (réception, envoyés, brouillons, corbeille, spam) deviennent des boîtes « système » que l'utilisateur ne peut pas renommer ni supprimer.

### 9.10 Libellés multiples sur un provider à dossiers (D-17) [PROPOSÉ]

Sur un provider à dossiers, un message ne peut pas être dans deux dossiers sans être **copié** (un doublon réel sur le serveur, qui consomme du quota). Le moteur ne doit donc autoriser le multi-libellé que si le serveur le permet vraiment : c'est une capacité que le provider déclare, et le cœur adapte son comportement.

Que se passe-t-il alors quand l'utilisateur ajoute un second libellé à un message d'un compte à dossiers ? Deux options :

| Option | Effet |
|---|---|
| **i** | L'action est refusée, ou devient un déplacement : un seul libellé reste possible |
| **ii** | MIRA garde ce second libellé **en local seulement** : il n'existe pas côté serveur et disparaît sur un autre appareil |

L'option **i** a été recommandée pour la version 1 : elle est honnête et sans surprise. Les libellés locaux (ii) pourraient venir plus tard comme fonctionnalité à part, avec un avertissement clair. Cette recommandation n'a pas été validée explicitement.

### 9.11 Le vocabulaire côté interface

Le vocabulaire de l'interface reste « dossier » pour tous les comptes. Pour Gmail, on parlera de dossiers qui peuvent se chevaucher. Le modèle interne reste général sans que l'utilisateur ait à apprendre « libellé ». Le vocabulaire et le modèle sont deux questions séparées.

### 9.12 Récapitulatif des décisions de ce chapitre

| ID | Décision | Statut |
|---|---|---|
| D-12 | Message unique, boîtes = vues d'appartenance, vues calculées en plus | DÉCIDÉ |
| D-13 | Identifiant interne MIRA, correspondance avec les identifiants serveur, pas de fusion en cas de doute | DÉCIDÉ |
| D-14 | Drapeaux utilisateur au niveau du message | DÉCIDÉ |
| D-15 | Fils calculés par MIRA, indice serveur utilisé s'il existe | DÉCIDÉ |
| D-16 | Source brute compressée, analyse en cache, pièces jointes à la demande | DÉCIDÉ |
| D-17 | Second libellé sur provider à dossiers : refus ou déplacement | PROPOSÉ |

---
## 10. La synchronisation

La synchronisation est le mécanisme qui garde le miroir local cohérent avec le serveur, et qui fait remonter vers le serveur ce que l'utilisateur a fait. Elle va dans **deux sens**, qu'il faut concevoir ensemble : **descendre** ce que le serveur sait, et **remonter** ce que l'utilisateur a décidé hors ligne ou en attente.

### 10.1 Objectifs

La synchronisation doit atteindre, dans cet ordre de priorité :

1. **ne jamais perdre** un message, une action ou un brouillon de l'utilisateur ;
2. **rendre l'interface utilisable vite**, y compris au premier lancement, avant que tout soit téléchargé ;
3. **rester correcte après n'importe quelle interruption** : coupure réseau, arrêt brutal, changement du serveur ;
4. **limiter le volume transféré** en ne redemandant pas ce qu'on sait déjà ;
5. **respecter les serveurs** : limites de connexions, fréquence des relevés ;
6. **se dégrader proprement** sur les serveurs moins capables.

### 10.2 La descente, par étapes (D-20)

Le téléchargement se fait par étapes, de sorte que l'interface soit utilisable le plus tôt possible.

1. **La liste des boîtes** du compte, avec leurs rôles.
2. Pour chaque boîte synchronisée, les **en-têtes** (expéditeur, objet, date, drapeaux, informations de rattachement aux fils), du plus récent au plus ancien.
3. Les **corps** des messages, à la demande, avec **préchargement** des plus récents.
4. Les **pièces jointes**, seulement quand l'utilisateur les ouvre.

Cet ordre a une conséquence directe sur l'expérience : après la liste des boîtes et les premiers en-têtes, l'utilisateur voit déjà sa boîte de réception. Le reste se complète en arrière-plan. Il a aussi une conséquence sur les données : un message peut exister dans le moteur à l'état **en-tête seul**, sans corps, ce que le modèle de domaine prévoit (8.6).

### 10.3 La fenêtre locale (D-21)

Télécharger toute une messagerie n'a pas de sens : elle peut contenir des centaines de milliers de messages. Le moteur travaille donc dans une **fenêtre** de synchronisation.

**Décision D-21** : par défaut, les **en-têtes des 90 derniers jours** sont synchronisés, les **corps** ne sont préchargés que pour les messages les plus récents, et un **bouton « remonter plus loin »** permet de charger des périodes plus anciennes à la demande. La fenêtre est un réglage du compte.

Conséquences à connaître :

- les messages **hors fenêtre** ne sont pas dans la base locale : ni liste, ni recherche locale ne les voient ;
- la commande de recherche prévoit donc une option **« chercher aussi sur le serveur »**, qui interroge le serveur pour ce qui sort de la fenêtre ;
- les **fils** sont calculés sur ce qui est présent localement : un fil peut paraître incomplet si une partie est hors fenêtre, et il se complète en remontant ;
- remonter la fenêtre est une **tâche longue** (identifiant d'opération, progression, annulation).

Cette décision a un point d'appui manquant : le porteur n'a pas encore indiqué la **taille cible** (nombre de comptes, nombre de messages), qui permettrait de vérifier que 90 jours est un défaut raisonnable (D-32, ouvert).

### 10.4 Savoir ce qui a changé (D-22)

Après la première synchronisation, il faut savoir efficacement **ce qui a changé** côté serveur. IMAP fonctionne boîte par boîte : pour chaque boîte, le moteur mémorise un **repère** (le numéro de validité, le dernier numéro vu, et, si le serveur en tient un, un compteur de modifications). La méthode dépend de ce que le serveur annonce.

| Situation | Stratégie | Coût |
|---|---|---|
| **Serveur moderne** (extensions CONDSTORE et QRESYNC) | On demande « qu'est-ce qui a changé depuis ce repère ? », y compris les suppressions | Très faible |
| **Serveur plus limité** | On compare les drapeaux de tous les messages de la fenêtre avec la copie locale, on détecte les nouveaux par leur numéro, les suppressions par différence de listes | Plus lent, mais fonctionne |
| **Le numéro de validité a changé** | Le serveur a renuméroté la boîte : nos numéros ne valent plus rien, on **resynchronise la boîte** | Plus élevé, mais ponctuel |

**Décision D-22** : la stratégie est **choisie d'après les capacités annoncées par le serveur** à la connexion, jamais d'après l'identité du provider. C'est le principe P6 appliqué à la synchronisation. Cela relève du cœur : une interface de « stratégie de synchronisation » avec plusieurs implémentations, dont le cœur choisit la plus adaptée.

Le choix peut se faire **par connexion** (les capacités sont celles du serveur) et être affiné par les exceptions du provider (une extension annoncée mais peu fiable est ignorée).

### 10.5 Renumérotation et resynchronisation d'une boîte

Quand le numéro de validité d'une boîte change, **tous les numéros que nous avions mémorisés pour elle sont invalides**. Le moteur ne peut plus relier directement nos apparitions aux messages du serveur. La procédure est la suivante :

1. marquer les apparitions de cette boîte comme **à revalider** (sans les supprimer) ;
2. relire la boîte dans la fenêtre voulue ;
3. pour chaque message du serveur, retrouver le message local correspondant par la **stratégie d'identité** (9.5) : identifiant stable du serveur d'abord, recoupement en dernier recours ;
4. recréer les apparitions avec les nouveaux numéros ;
5. traiter ce qui reste : les messages locaux sans correspondance sont retirés de cette boîte (et du miroir s'ils n'ont plus aucune apparition) ; les messages du serveur sans correspondance sont ajoutés.

C'est ici que la règle de prudence (P7, préférer le doublon à la perte) joue pleinement : en cas de doute sur la correspondance, on laisse deux messages plutôt que d'en fusionner à tort.

Les **actions en attente** visant des messages de cette boîte doivent survivre à l'opération : elles sont relues contre les nouvelles apparitions avant d'être rejouées.

### 10.6 Rester à jour en temps réel

Pour être prévenu d'un nouveau message sans interroger le serveur en boucle, on utilise la réception en temps réel d'IMAP (extension **IDLE**) : le moteur garde une connexion ouverte sur la boîte de réception, et le serveur le prévient de lui-même. Une extension plus récente (**NOTIFY**) étend le principe à plusieurs boîtes, avec un seul réglage.

Points d'architecture :

- une connexion en attente (IDLE) est **bloquée** à écouter : le travail se fait sur **une seconde connexion**. Un compte utilise donc au moins deux connexions, ce qui doit rester sous la limite que le provider indique ;
- les **autres boîtes** sont relevées à intervalles, avec un délai minimal fixé par le provider ;
- après **toute coupure**, on resynchronise, car des changements ont pu passer pendant l'absence ;
- la reconnexion se fait avec un **délai croissant** (*backoff*) pour ne pas harceler un serveur en difficulté ;
- si un serveur annonce IDLE mais le rend peu fiable, l'exception connue du provider fait retomber le moteur sur un relevé périodique ;
- un événement « nouveau message reçu » est émis dès qu'un message apparaît, c'est le déclencheur des notifications et de la future intégration de l'agent d'IA.

### 10.7 La remontée : la file d'actions (D-25)

#### Le principe

Chaque action de l'utilisateur (marquer lu, déplacer, mettre à la corbeille, envoyer) est appliquée **tout de suite en local**, donc l'interface réagit sans attendre. Elle est ensuite ajoutée à la **file d'actions** du compte, qui la rejouera sur le serveur dans l'ordre. Cette approche, dite *optimiste*, est ce qui rend le mode hors ligne naturel : hors connexion, la file grossit ; à la reconnexion, elle se vide.

#### Cycle de vie d'une action

Une action passe par les états *en attente*, *en cours*, puis *réussie*, *échouée* ou *annulée* (8.6). À chaque changement d'état, un événement est émis.

#### Règles

- **Ordre** : les actions d'un compte sont rejouées dans l'ordre où elles ont été enregistrées, surtout lorsqu'elles se suivent sur le même message (déplacer, puis marquer comme lu).
- **Idempotence** : chaque action porte une clé fournie à sa création. Rejouée après un plantage, elle ne produit pas deux effets. Marquer un drapeau est naturellement idempotent ; déplacer l'est moins, et demande une vérification de l'état avant de recommencer.
- **Persistance avant application** : une action est écrite dans la base **avant** d'être tentée. Aucune action ne vit uniquement en mémoire.
- **Erreur passagère** (réseau, serveur occupé) : l'action repasse en attente avec un compteur de tentatives et un délai croissant.
- **Erreur définitive** (le message n'existe plus, quota dépassé, opération refusée) : l'action passe à *échouée*, **l'effet local est annulé** (l'interface revient à l'état réel), et un événement prévient l'utilisateur avec la raison.
- **Regroupement** : des actions semblables sur un même compte (marquer lues cent messages) peuvent être regroupées en une opération côté serveur, pour économiser des échanges.
- **Annulation** : pendant une courte fenêtre, l'utilisateur peut annuler une action (le *undo*). Un envoi annulé retourne dans les brouillons.

La file d'actions est aussi la **boîte d'envoi** (*outbox*) : un envoi est une action comme une autre, et l'envoi différé n'est que le report de sa date d'exécution.

### 10.8 L'envoi

L'envoi d'un message se déroule en plusieurs temps, qui portent plusieurs pièges.

1. **Préparation** : le brouillon est assemblé en un message complet (en-têtes, corps, pièces jointes).
2. **Délai d'annulation** : le message reste dans la file pendant une courte période pendant laquelle il peut être annulé.
3. **Envoi par SMTP**, avec l'identité choisie.
4. **Copie dans « Envoyés »** : si le serveur n'enregistre pas lui-même la copie, MIRA l'ajoute dans la boîte des envoyés. Si le serveur la fait lui-même, MIRA ne doit **pas** l'ajouter, sous peine d'un doublon. Le provider déclare lequel des deux cas s'applique (6.8).
5. **Nettoyage** : le brouillon est supprimé (localement, et côté serveur si une copie y avait été enregistrée).

**Le cas le plus délicat** [OUVERT] : un plantage entre l'acceptation du message par le serveur SMTP et l'enregistrement local de cette réussite. Au redémarrage, le moteur ignore si le message est parti. Renvoyer risque un **doublon** que le destinataire verra ; ne rien faire risque de **perdre** un envoi. La mitigation envisagée est de marquer l'envoi comme *tentative en cours* avant d'appeler le serveur, et, au redémarrage, de **vérifier** (par exemple dans la boîte des envoyés) avant de décider, en demandant à l'utilisateur en cas de doute. Le détail n'est pas tranché.

### 10.9 Les conflits (D-23)

Un conflit survient quand le serveur et l'état local divergent : un message supprimé depuis un autre appareil, un drapeau modifié ailleurs, un message déplacé pendant qu'une action locale le visait.

**Décision D-23** : le **serveur gagne toujours** sur le miroir, et les **actions locales en attente sont rejouées par-dessus**. Tant qu'une action n'est pas confirmée, l'interface continue d'afficher son effet. L'image est celle d'un `git rebase` : on rejoue ses commits locaux au-dessus de l'état distant.

Cas courants :

| Situation | Résolution |
|---|---|
| Le message visé par une action a été supprimé du serveur | L'action échoue définitivement, l'effet local est annulé, l'utilisateur est prévenu |
| Un drapeau a changé côté serveur pendant qu'on avait une action locale en attente sur ce drapeau | La valeur du serveur est prise, puis l'action locale est rejouée si elle est encore pertinente |
| Un message a été déplacé côté serveur | Les apparitions sont mises à jour par la synchronisation ; l'action locale est rejouée sur la nouvelle situation |
| Une boîte a été renommée ou supprimée côté serveur | Le miroir suit le serveur ; les actions locales qui la visaient échouent proprement |

### 10.10 Un cas difficile : « Tous les messages » de Gmail (D-24)

Le modèle de vues règle la question de l'**identité** pour Gmail. La difficulté qui reste est un problème de **volume de téléchargement**, pas de modélisation.

IMAP travaille boîte par boîte : pour connaître le contenu du libellé « Travail », on liste cette boîte, et ainsi de suite pour chaque libellé. Un message Gmail qui porte trois libellés apparaît dans trois boîtes, plus « Tous les messages ». Si l'on télécharge naïvement chaque boîte, on récupère le même message plusieurs fois avant de s'apercevoir que c'est le même. À titre d'illustration : avec 10 000 messages portant chacun trois libellés, on ferait 30 000 téléchargements au lieu de 10 000.

**La solution esquissée** consiste à télécharger le **contenu une seule fois**, depuis la boîte qui contient tout (celle de rôle « tous les messages », repérable par un attribut standard d'IMAP et non par un nom propre à Gmail), et à ne relever dans les autres boîtes que la **liste légère** de leurs messages, pour enregistrer les appartenances. Cette optimisation suppose un identifiant stable côté serveur pour reconnaître les messages ; sans lui, on retombe sur le téléchargement complet. Elle reste générique : elle s'exprime en termes de rôle de boîte et de capacité, pas de provider.

**Décision D-24** : l'optimisation est **notée, reportée**. On peut commencer par la version simple (en-têtes seulement, sur 90 jours, ce qui reste léger) et optimiser si les mesures le justifient. Le modèle (message unique, apparitions) n'a pas besoin de changer pour cela.

### 10.11 Reprise après coupure ou plantage

La synchronisation doit pouvoir être **interrompue n'importe quand** sans corruption.

- Les **repères** par boîte ne sont avancés **qu'après** que les données correspondantes sont écrites : un plantage au milieu fait simplement refaire un peu de travail.
- Les **actions en attente** sont persistantes et idempotentes.
- Les **téléchargements** de corps et de pièces jointes écrivent dans un emplacement temporaire et ne deviennent visibles qu'une fois complets.
- Au démarrage, le moteur **reprend** là où il en était : actions restantes, boîtes à rafraîchir, téléchargements incomplets.
- Un **arrêt propre** (commande de fermeture) termine les opérations en cours dans un délai borné, puis ferme les connexions et la base.

### 10.12 Limites de débit et courtoisie

Les serveurs tolèrent un nombre limité de connexions et de requêtes. Le moteur s'impose ses propres plafonds :

- **une tâche de synchronisation par compte**, jamais plusieurs en parallèle sur un même compte ;
- un **plafond de connexions** par compte, fourni par le provider (valeur prudente par défaut) ;
- un **délai minimal** entre deux relevés des boîtes qui ne sont pas en temps réel ;
- un **repli** (*backoff*) sur les refus de type limitation ;
- une **priorité** : la boîte de réception d'abord, puis les boîtes visibles par l'utilisateur, puis le reste.

### 10.13 Observabilité

La synchronisation doit pouvoir être **observée** : événements de démarrage, de progression (étape, n sur total), de fin, d'échec, de passage hors ligne et de retour en ligne ; journal de diagnostic ; état consultable. Cette observabilité est ce qui rend le CLI utile comme banc d'essai, et ce qui permettra à l'interface d'afficher une progression honnête.

### 10.14 Dégradation par capacité : les replis [PROPOSÉ]

Le principe P6 (se dégrader selon les capacités) demande une table des **replis** : pour chaque fonction qui s'appuie sur une extension, ce que fait le moteur quand le serveur ne l'annonce pas. Cette table est une base de travail issue de la connaissance générale d'IMAP ; elle sera confirmée à l'implémentation.

| Fonction | Si l'extension est annoncée | Repli si elle est absente |
|---|---|---|
| Détection des changements | CONDSTORE et QRESYNC : on demande les changements depuis le repère, suppressions comprises | Comparaison des drapeaux sur la fenêtre, détection des nouveaux par leur numéro, des suppressions par différence de listes |
| Réception en temps réel | IDLE sur la réception, NOTIFY pour plusieurs boîtes | Relevé périodique de la boîte de réception, au délai minimal du provider |
| Déplacement d'un message | MOVE : opération atomique | Copie, marquage « supprimé » de l'original, puis purge de cet original |
| Purge ciblée | UIDPLUS : suppression d'un seul message par son numéro | Purge générale de la boîte, risquée si d'autres messages sont déjà marqués supprimés |
| Rôle des dossiers | SPECIAL-USE : attributs standard | Noms usuels (réception, envoyés, brouillons, corbeille, spam), en plusieurs langues, avec l'aide du provider |
| Identité stable d'un message | OBJECTID, ou identifiant propre au service | Numéro dans la boîte, puis recoupement (identifiant de message, date, taille) |
| Édition d'un brouillon | REPLACE : remplacement atomique | Ajout de la nouvelle version puis suppression de l'ancienne, avec un bref risque de doublon |
| Recherche sur le serveur | ESEARCH et MULTISEARCH : réponses compactes, plusieurs boîtes | Recherche classique, boîte par boîte |
| Volume échangé | COMPRESS : connexion compressée | Échange non compressé |
| Envoi de gros contenus | LITERAL+ : envoi sans attendre l'accusé | Envoi avec accusé à chaque bloc, plus lent |
| Noms internationaux | UTF8=ACCEPT | Codage traditionnel des noms de boîtes et des en-têtes |
| Diagnostic | ID : échange des noms de logiciels | Omis |

Quelques règles complètent la table :

- les capacités sont **lues à chaque connexion** et mémorisées dans l'état du compte, car un serveur peut en changer ;
- chaque repli **choisi** est écrit dans le journal de diagnostic, pour que l'on sache pourquoi une synchronisation est lente ;
- le cas de la **purge sans UIDPLUS** reste ouvert : soit le moteur accepte de ne pas purger tant qu'il ne peut pas cibler, soit il purge tout et prend le risque. La prudence indique de ne pas purger ;
- les **tests** de synchronisation sont rejoués contre un faux serveur avec **plusieurs jeux de capacités** (serveur complet, serveur sans QRESYNC, serveur sans MOVE, serveur minimal), de façon à vérifier que chaque repli donne un résultat correct, simplement plus lent.

### 10.15 Récapitulatif des décisions de ce chapitre

| ID | Décision | Statut |
|---|---|---|
| D-20 | Descente par étapes : boîtes, en-têtes, corps, pièces jointes | DÉCIDÉ |
| D-21 | Fenêtre : en-têtes sur 90 jours, corps récents préchargés, bouton pour remonter | DÉCIDÉ |
| D-22 | Détection des changements choisie d'après les capacités du serveur | DÉCIDÉ |
| D-23 | Le serveur gagne toujours ; actions locales rejouées par-dessus | DÉCIDÉ |
| D-24 | Optimisation Gmail « Tous les messages » : notée, reportée | DÉCIDÉ |
| D-25 | Écritures optimistes en local, file d'actions, résultat par événement | PROPOSÉ |
| (ouvert) | Envoi interrompu entre l'acceptation SMTP et l'enregistrement local | OUVERT |

---

## 11. Les fonctionnalités d'un client mail : étude et couverture

Pour que le moteur ne passe pas à côté de fonctionnalités indispensables, une recherche a été menée sur ce qu'on s'attend à trouver dans un client de messagerie. Ce chapitre en présente la synthèse et indique comment l'architecture les couvre.

### 11.1 Méthode et limites

La recherche a croisé des guides d'achat et des comparatifs de clients de messagerie (pour Linux et pour Mac), des pages de présentation de clients de référence, un brouillon de l'IETF sur les extensions IMAP, et des articles techniques sur le rendu sécurisé du HTML et le désabonnement en un clic. Les références figurent en annexe C.

**Limites à garder en tête.** Aucune source ne donne de liste officielle de ce qui est « obligatoire » ; le classement présenté ici est une **synthèse**. La plupart des sources sont des pages commerciales ou des guides d'achat, dont certains critères visent des équipes en entreprise. Seul le brouillon de l'IETF est une référence technique solide, et il traite des protocoles, pas des fonctionnalités. Le classement en trois niveaux est donc un jugement, pas un fait.

### 11.2 Le socle indispensable

Ce sont les fonctions sans lesquelles ce n'est pas un client de messagerie.

- **Réception et envoi** via IMAP et SMTP. POP3 est cité partout mais reste facultatif pour MIRA.
- **Multi-comptes** avec boîte de réception unifiée, couvrant Gmail, Outlook, Yahoo, iCloud et IMAP générique.
- **Accès hors ligne** : lire, rédiger et mettre des actions en attente sans connexion.
- **Organisation de base** : dossiers, archivage, drapeaux, priorité, corbeille et spam.
- **Recherche rapide** sur tout le contenu local.
- **Rédaction** : réponse, transfert, pièces jointes, signatures, brouillons sauvegardés.
- **Affichage fiable** : MIME, HTML et texte brut, pièces jointes, encodages.
- **Notifications** de nouveaux messages.
- **Sécurité de base** : connexions chiffrées, stockage sécurisé des identifiants, protection contre l'hameçonnage, filtrage du spam.

### 11.3 Ce qui est attendu aujourd'hui

Ce sont les fonctions qui font le standard des clients modernes.

- **Règles et filtres automatiques.**
- **Envoi différé et annulation d'envoi** (le message retourne dans les brouillons).
- **Rappels et relances** sur les messages envoyés sans réponse, et **mise en attente** (*snooze*).
- **Contacts** avec autocomplétion, et idéalement calendrier ou invitations.
- **Chiffrement PGP et S/MIME**, que les comparatifs citent systématiquement.
- **Confidentialité** : images distantes bloquées par défaut, avec option par message ou par expéditeur.
- **Désabonnement en un clic.**
- **Clavier et vitesse** : raccourcis, navigation sans souris.
- **Personnalisation** : thèmes, modèles de réponse, dossiers intelligents.
- **Migration** : import depuis d'autres clients ou depuis des fichiers locaux.

### 11.4 Ce qui différencie

Ce qui n'est pas du socle, et où MIRA peut se démarquer :

- résumés de fils, tri et priorisation, aide à la rédaction, recherche par le sens (c'est l'agent d'IA prévu par le brief) ;
- mise en avant des vrais interlocuteurs avant les newsletters ;
- intégrations et suivi d'équipe (boîtes partagées, CRM, tâches), qui visent surtout l'entreprise et ne sont pas dans le périmètre de MIRA.

### 11.5 Les extensions IMAP à prévoir

Le brouillon de l'IETF intitulé *IMAP Extensions Suggestions* recommande les extensions suivantes, que le moteur doit savoir détecter et exploiter quand elles sont présentes (D-22).

| Besoin | Extension | Référence |
|---|---|---|
| Activer d'autres extensions | ENABLE | RFC 5161 |
| Support d'UTF-8 | UTF8=ACCEPT | RFC 9755 |
| Rôles de dossiers (envoyés, corbeille…) | SPECIAL-USE | RFC 6154 |
| Identifier client et serveur (débogage) | ID | RFC 2971 |
| Compression de la connexion | COMPRESS | RFC 4978 |
| **Synchronisation incrémentale** (le plus important pour un cache local) | CONDSTORE et QRESYNC | RFC 7162 |
| Envoyer sans attendre l'accusé | LITERAL+ | RFC 7888 |
| **Identifiants stables** des messages, fils et boîtes | OBJECTID | RFC 8474 |
| Déplacement atomique | MOVE | RFC 6851 |
| Suppression ciblée par numéro | UIDPLUS | RFC 4315 |
| Remplacement atomique (édition de brouillons) | REPLACE | RFC 8508 |
| Recherche compacte | ESEARCH | RFC 4731 |
| Recherche multi-boîtes | MULTISEARCH | RFC 7377 |
| Réception en temps réel | IDLE | RFC 2177 |
| Réception en temps réel sur plusieurs boîtes | NOTIFY | RFC 5465 |
| Sessions utilisant uniquement les numéros | UIDONLY | RFC 9586 |

Le brouillon note que CONDSTORE et QRESYNC sont les extensions d'efficacité les plus importantes, mais qu'elles n'apportent rien à un client sans cache local. Comme MIRA a un cache local, elles sont centrales pour lui.

### 11.6 Deux détails qui touchent l'architecture

**Les pixels espions et le rendu HTML.** Un pixel espion est une minuscule image hébergée chez l'expéditeur : quand le client la charge, le serveur de l'expéditeur enregistre une requête liée au destinataire. Trois défenses sont possibles. **Bloquer le contenu distant par défaut** : tant que l'utilisateur ne l'approuve pas, la plupart des pixels ne se déclenchent pas, au prix de masquer aussi des images légitimes. **Faire passer les images par un relais** : l'expéditeur voit l'adresse du relais plutôt que celle de l'utilisateur, mais l'ouverture reste enregistrée. **Supprimer les références d'images distantes** : insuffisant si on ne retire que les images de un pixel, puisqu'un traceur peut être un logo visible avec une adresse propre au destinataire. Le brief de MIRA prévoit le blocage par défaut ; la question d'un relais reste ouverte.

**Le désabonnement en un clic (RFC 8058).** La norme repose sur deux en-têtes : `List-Unsubscribe` (qui doit contenir une adresse HTTPS) et `List-Unsubscribe-Post` (qui doit contenir exactement la valeur `List-Unsubscribe=One-Click`), tous deux couverts par une signature DKIM valide. L'action consiste à envoyer une requête HTTPS POST à l'adresse indiquée, sans cookies ni autorisation. La source consultée décrit surtout le rôle du **fournisseur de boîte mail**, pas celui d'un client. Pour un client comme MIRA, proposer ce bouton est une extension raisonnable, à condition de s'appuyer sur la vérification DKIM déjà faite par le serveur (information généralement portée par un en-tête de résultat d'authentification). Cette dernière partie est une **inférence** de la conception, non une information de la source. C'est aussi une raison de conserver la source brute (D-16) : les en-têtes de désabonnement s'y trouvent.

### 11.7 Couverture par le moteur

Le tableau rapproche chaque fonction du socle et de l'attendu de la partie de l'architecture qui la porte.

| Fonction | Entités ou mécanismes | Phase |
|---|---|---|
| Réception, envoi | Clients IMAP et SMTP, synchronisation, file d'actions | v1 |
| Multi-comptes, boîte unifiée | Comptes, vue calculée unifiée | v1 |
| Hors ligne | Miroir local, file d'actions, écritures optimistes | v1 |
| Dossiers, archivage, corbeille | Boîtes, rôles, actions traduites par le provider | v1 |
| Recherche | Index texte intégral local, option serveur | v1 |
| Rédaction, signatures, brouillons | Brouillon, identités | v1 |
| Pièces jointes | Pièce jointe, cache à la demande | v1 |
| Notifications | Événement « nouveau message reçu » | v1 |
| Rendu HTML sûr | Contenu assaini, liste des ressources bloquées | v1 |
| Sécurité des identifiants | Trousseau | v1 |
| Envoi différé, annulation d'envoi | File d'actions, délai d'exécution | v1 |
| Fils | Calcul par le moteur | v1 |
| Règles et filtres | À concevoir | plus tard |
| Contacts, autocomplétion | Contacts alimentés automatiquement | plus tard |
| Désabonnement en un clic | En-têtes conservés, action dédiée | plus tard |
| PGP, S/MIME | Exige la source brute (D-16) | plus tard |
| Rappels, mise en attente | À concevoir | plus tard |
| Recherche par le sens, résumés, aide à la rédaction | Agent d'IA via le bus d'événements | plus tard |

### 11.8 Conséquences sur l'architecture

Quatre conséquences se dégagent de cette étude :

- le **hors ligne** impose la file d'actions et les écritures optimistes (10.7) ;
- les **extensions IMAP** imposent de détecter les capacités et de se dégrader proprement (D-22) ;
- le **rendu HTML sûr** et la **vérification de signatures** exigent de conserver la source brute (D-16) ;
- la **recherche** dans une fenêtre limitée exige une option de recherche sur le serveur (10.3).

---
## 12. Le contrat d'API du moteur

### 12.1 Rôle et statut

Le contrat d'API est la **frontière entre le moteur et ses clients** : le CLI d'abord, l'interface Flutter ensuite, un éventuel agent d'IA plus tard. Il dit ce que le moteur promet, et rien de plus : les clients n'ont aucun autre moyen d'agir sur lui. Dans l'esprit d'Arcade, c'est le document d'interface que deux équipes pourraient se partager pour que leurs briques fonctionnent ensemble.

Le contrat a été rédigé **à la demande du porteur**, qui souhaitait un contrat « assez exhaustif, bien rempli, couvrant tous les cas possibles ». Une réserve s'impose : on ne peut pas garantir la couverture de tous les cas sur le papier, certains n'apparaîtront qu'au contact de vrais serveurs. Le contrat couvre toutes les **catégories** de cas connues ; il sera complété avec le CLI. Il n'a pas été relu commande par commande : son statut est **PROPOSÉ** (D-26), avec trois points ouverts (D-27, D-28, D-29).

### 12.2 Principes transversaux

1. **Lire, c'est toujours local.** Les lectures (liste, message, recherche) interrogent uniquement la base locale, jamais le réseau : elles sont instantanées et fonctionnent hors ligne. Quand le réseau est nécessaire (corps absent, remonter au-delà de la fenêtre), c'est une commande explicite.
2. **Écrire, c'est enregistrer une intention.** Une commande de modification s'applique tout de suite en local, entre dans la file d'actions et rend la main avec un **identifiant d'action** ; le résultat côté serveur arrive ensuite sous forme d'événement.
3. **Idempotence.** Chaque écriture porte une **clé fournie par le client** : rejouée (double clic, relance après un plantage), elle ne s'exécute qu'une fois. C'est vital pour l'envoi.
4. **Tâches longues** (synchronisation, téléchargement d'une pièce jointe, remontée de la fenêtre, recherche sur le serveur) : elles retournent un **identifiant d'opération**, émettent leur progression par événements et sont **annulables**.
5. **Listes toujours paginées par curseur**, jamais « tout » : une boîte peut contenir des centaines de milliers de messages.
6. **Identifiants opaques** : ceux de MIRA, que les clients ne construisent ni n'interprètent. Aucune notion d'écran ou de page. **Aucun secret en sortie**, jamais.
7. **Contrat versionné** : le client demande la version du contrat à l'ouverture et refuse proprement en cas d'incompatibilité.
8. **Événements numérotés** : un client qui se reconnecte indique « j'ai vu jusqu'au numéro X » et rattrape ce qu'il a manqué.

### 12.3 Les quatre formes d'échange

| Forme | Sens | Réseau ? | Exemple |
|---|---|---|---|
| **Requête** | Client → moteur, réponse immédiate | Non (base locale) | Lister les messages d'une boîte |
| **Commande d'intention** | Client → moteur, rend un identifiant d'action | Différé (via la file d'actions) | Marquer lu, déplacer, envoyer |
| **Commande réseau explicite** | Client → moteur, tâche longue | Oui, annoncé | Remonter la fenêtre, tester une connexion |
| **Événement** | Moteur → client | — | Nouveau message, progression, erreur |

### 12.4 Les commandes et requêtes, par domaine

La colonne « Phase » indique si le domaine est prévu en version 1 ou plus tard. Ce découpage est lui-même ouvert (D-28).

#### Moteur

| Opération | Nature | Phase |
|---|---|---|
| Ouvrir un profil | Commande | v1 |
| Fermer proprement (terminer les opérations en cours, fermer connexions et base) | Commande | v1 |
| Version du contrat et capacités du moteur | Requête | v1 |
| État global (comptes, synchronisations en cours, file d'actions) | Requête | v1 |

#### Providers

| Opération | Nature | Phase |
|---|---|---|
| Lister les providers disponibles | Requête | v1 |
| Détecter le provider d'une adresse | Requête | v1 |

#### Comptes

| Opération | Nature | Phase |
|---|---|---|
| Préparer un compte (détection ou choix explicite, méthodes d'authentification proposées) | Requête | v1 |
| Tester la connexion sans rien enregistrer | Réseau explicite | v1 |
| Ajouter un compte | Commande | v1 |
| Modifier un compte (nom, serveurs pour « Autre », fenêtre de synchronisation, boîtes synchronisées) | Commande | v1 |
| Mettre en pause ou reprendre | Commande | v1 |
| Supprimer (efface les données locales et les secrets, ne touche pas au serveur) | Commande | v1 |
| Lister les comptes et leur état | Requête | v1 |

#### Authentification

| Opération | Nature | Phase |
|---|---|---|
| Démarrer un flux OAuth (renvoie l'adresse à ouvrir dans le navigateur) | Réseau explicite | v1 |
| Terminer ou annuler un flux OAuth | Commande | v1 |
| Enregistrer un mot de passe d'application (rangé dans le trousseau) | Commande | v1 |
| Ré-authentifier un compte dont le jeton a expiré | Réseau explicite | v1 |

#### Identités

| Opération | Nature | Phase |
|---|---|---|
| Lister les identités d'un compte | Requête | v1 |
| Ajouter un alias, modifier la signature ou le nom affiché, choisir l'identité par défaut | Commande | v1 |

#### Boîtes

| Opération | Nature | Phase |
|---|---|---|
| Lister les boîtes (arbre, rôles, compteurs lus et non lus) | Requête | v1 |
| Créer, renommer, supprimer une boîte | Commande d'intention | v1 |
| Choisir les boîtes synchronisées | Commande | v1 |
| Vider la corbeille ou le spam | Commande d'intention | v1 |

#### Messages (lecture)

| Opération | Nature | Phase |
|---|---|---|
| Lister une boîte ou une vue (curseur, tri, filtres) | Requête | v1 |
| Lire les en-têtes d'un message | Requête | v1 |
| Lire le corps (déclenche le téléchargement si le corps est absent) | Requête, réseau si nécessaire | v1 |
| Obtenir la source brute | Requête | v1 |
| Lister les pièces jointes ; récupérer une pièce jointe vers un fichier | Requête ; réseau explicite avec progression | v1 |
| Obtenir le contenu HTML sécurisé, avec la liste des ressources distantes bloquées | Requête | v1 |
| Autoriser les ressources distantes pour un message ou un expéditeur | Commande | v1 |

#### Fils

| Opération | Nature | Phase |
|---|---|---|
| Lister les fils d'une boîte ; lire un fil | Requête | v1 |

#### Recherche et vues

| Opération | Nature | Phase |
|---|---|---|
| Recherche en texte intégral locale, avec filtres (expéditeur, destinataire, date, pièce jointe, boîte, drapeau) | Requête | v1 |
| Recherche étendue au serveur (au-delà de la fenêtre locale) | Réseau explicite, annulable | v1 |
| Vues standard : boîte unifiée, non lus, suivis | Requête | v1 |
| Vues enregistrées : créer, modifier, supprimer | Commande | plus tard |
| Recherche par le sens (vecteurs) | Requête | plus tard |

#### Actions sur les messages

| Opération | Nature | Phase |
|---|---|---|
| Marquer lu ou non lu, suivi ou non | Commande d'intention | v1 |
| Déplacer, ajouter ou retirer d'une boîte | Commande d'intention | v1 |
| Archiver, mettre à la corbeille, supprimer définitivement, signaler comme spam ou non | Commande d'intention | v1 |
| Appliquer en lot (une liste de messages, une sélection, ou le résultat d'une requête) | Commande d'intention | v1 |
| Annuler une action récente (*undo*) | Commande | v1 |

#### Rédaction et envoi

| Opération | Nature | Phase |
|---|---|---|
| Créer un brouillon (neuf, réponse, répondre à tous, transfert) | Commande | v1 |
| Modifier, joindre ou retirer une pièce jointe, enregistrer, supprimer | Commande | v1 |
| Envoyer ; planifier l'envoi ; annuler un envoi pendant son délai | Commande d'intention | v1 |
| Lister la boîte d'envoi (messages en cours d'envoi, planifiés, échoués) | Requête | v1 |

#### File d'actions

| Opération | Nature | Phase |
|---|---|---|
| Lister les actions en attente et échouées | Requête | v1 |
| Relancer, annuler | Commande | v1 |

#### Synchronisation

| Opération | Nature | Phase |
|---|---|---|
| Lancer une synchronisation (tout, un compte, une boîte) ; la mettre en pause | Réseau explicite | v1 |
| État de la synchronisation | Requête | v1 |
| Remonter la fenêtre locale plus loin | Réseau explicite, annulable | v1 |
| Forcer une resynchronisation complète d'une boîte | Réseau explicite | v1 |

#### Maintenance

| Opération | Nature | Phase |
|---|---|---|
| Statistiques de stockage | Requête | v1 |
| Purger le cache de pièces jointes | Commande | v1 |
| Reconstruire l'index de recherche, vérifier l'intégrité de la base | Commande | v1 |
| Exporter un diagnostic (sans secrets) | Commande | v1 |
| Sauvegarde et export d'un profil | Commande | OUVERT (D-29) |

#### Plus tard

Contacts et autocomplétion, règles et filtres, désabonnement en un clic, rappels et mise en attente, PGP et S/MIME.

### 12.5 Les événements

Les événements sont des **notifications légères** : ils portent des identifiants, pas des données volumineuses. Le client relit ensuite par les requêtes locales ce dont il a besoin.

| Catégorie | Événements |
|---|---|
| **Moteur** | démarré, arrêt en cours, erreur fatale |
| **Comptes** | ajouté, modifié, supprimé, changement d'état (connecté, hors ligne, authentification requise, en pause, en erreur) |
| **Synchronisation** | démarrée, progression (étape, n sur total), terminée, échouée, passage hors ligne, retour en ligne |
| **Boîtes** | créée, modifiée, supprimée, compteurs mis à jour |
| **Messages** | **nouveau message reçu**, modifié (drapeaux, boîtes), supprimé, corps disponible, progression et fin d'un téléchargement de pièce jointe |
| **Fils** | mis à jour |
| **Actions** | mise en file, appliquée, échouée (avec la raison), annulée |
| **Envoi** | planifié, parti, échoué, copie dans « Envoyés » enregistrée |
| **Alertes** | quota plein, certificat à valider par l'utilisateur, limite de débit atteinte, boîte resynchronisée après renumérotation |
| **Journal** | messages de diagnostic |

Règles :

- les événements sont **numérotés** ; un client qui se reconnecte rattrape ce qu'il a manqué ;
- ils sont **filtrables** à l'abonnement (par compte, par catégorie) ;
- ils ne contiennent **jamais de secret** ni de contenu de message ;
- l'événement « nouveau message reçu » est le point d'entrée prévu pour l'agent d'IA : il suffit de s'y abonner, sans coupler le moteur à l'agent.

### 12.6 Les erreurs

Toute erreur du contrat a une forme commune :

- une **catégorie** ;
- un **code stable** (qui ne change pas d'une version à l'autre) ;
- un **message lisible** (le texte peut changer, jamais le code) ;
- un indicateur **« réessayable »** ;
- une **action suggérée** à l'utilisateur (ré-authentifier, accepter un certificat, libérer de l'espace) ;
- le **contexte** (compte, message, opération concernés).

| Catégorie | Exemples |
|---|---|
| **Entrée invalide** | adresse mal formée, paramètre manquant |
| **Introuvable** | compte ou message inexistant |
| **État incompatible** | supprimer une boîte système, modifier un compte en cours de suppression |
| **Réseau** | hors ligne, délai dépassé, serveur injoignable |
| **Certificat et TLS** | invalide, expiré, non fiable (requiert une décision de l'utilisateur) |
| **Authentification** | identifiants refusés, jeton expiré ou révoqué, mot de passe d'application requis, flux OAuth annulé |
| **Serveur** | quota dépassé, message trop gros, limite de débit, boîte inexistante, extension absente |
| **Stockage local** | disque plein, base corrompue, migration nécessaire |
| **Secrets** | trousseau indisponible ou verrouillé |
| **Annulé** | par l'utilisateur |
| **Interne** | bogue, avec un identifiant de diagnostic |

### 12.7 Les cas limites que le contrat doit traiter

- **Hors ligne complet** : les lectures fonctionnent, les écritures entrent dans la file.
- **Message supprimé côté serveur** avant que notre action ne l'atteigne.
- **Plantage ou arrêt brutal** pendant une synchronisation ou un envoi : reprise sans doublon d'envoi (voir l'ambiguïté signalée en 10.8).
- **Jeton expiré en plein envoi** : l'action attend la ré-authentification, elle n'échoue pas définitivement.
- **Suppression d'un compte pendant sa synchronisation** : la synchronisation est interrompue proprement, puis les données sont supprimées.
- **Boîte renumérotée** par le serveur (10.5).
- **Message mal formé**, encodage invalide, corps absent : on affiche ce qu'on peut, sans échouer.
- **Pièce jointe énorme** et téléchargement annulé : aucun fichier partiel n'est exposé.
- **Adresses internationales** (UTF-8) et dates invalides.
- **Deux clients sur le même profil** (le CLI et Flutter en même temps) : voir D-27.

### 12.8 Le CLI comme premier client

Chaque commande du contrat a son **pendant en sous-commande du CLI**. Le CLI est donc à la fois un outil utilisable et la **preuve vivante** que le contrat est complet : tout ce que Flutter pourra faire, le CLI doit pouvoir le faire d'abord.

Orientations **proposées**, non validées :

- une **sortie lisible par un humain** par défaut, et une **sortie structurée** facultative pour les scripts ;
- les **erreurs sur la sortie d'erreur**, comme dans Arcade ;
- des **codes de sortie** liés aux catégories d'erreur : 0 en cas de succès, un code non nul sinon. Arcade utilisait 84 ; MIRA pourrait garder ce code pour l'erreur générale et en réserver d'autres aux catégories. Le détail reste à fixer ;
- la **sélection du profil** par une option, même s'il n'existe qu'un profil implicite aujourd'hui.

### 12.9 Versionnement

Le contrat porte un **numéro de version**. Un client le lit à l'ouverture. Les changements compatibles (ajout d'une commande, d'un événement, d'un champ) ne cassent pas les clients existants ; les changements incompatibles incrémentent la version majeure. Cela permettra au CLI et à Flutter d'évoluer à des rythmes différents, et de détecter calmement un moteur trop ancien ou trop récent.

### 12.10 Les trois points ouverts du contrat

**D-27 : un seul moteur actif par profil ?** Deux processus qui écrivent dans la même base provoqueraient des conflits. Pour la version 1, un **verrou** est proposé : un moteur actif par profil, et le second client reçoit l'erreur « profil déjà utilisé ». Cela retrouve la question D-30 (processus séparé ou appel direct), qui sera tranchée avec la concurrence.

**D-28 : le découpage v1 et plus tard.** Le tableau du 12.4 propose un découpage ; il reste à le valider, notamment les règles et filtres, les contacts, le désabonnement en un clic et les rappels.

**D-29 : sauvegarde et export en version 1 ?** Ces commandes ont été placées « plus tard », mais perdre sa base est un risque réel pour l'utilisateur. À rapprocher des familles de données (8.1) : seules la configuration et les intentions sont à sauvegarder, le miroir se reconstruit.

---

## 13. Scénarios de bout en bout

Les scénarios suivants mettent à l'épreuve l'architecture en suivant des situations réelles de bout en bout. Ils sont **descriptifs** : ils montrent comment les décisions s'enchaînent, pas comment le code est écrit. Les détails des flux d'authentification (chapitre 15) restent à valider.

### S1 — Ajouter un compte Gmail par OAuth

1. L'utilisateur donne une adresse. Le client demande au moteur de **préparer** le compte : le registre détecte le provider Google (D-06) et renvoie ses serveurs et ses méthodes d'authentification, OAuth2 en tête.
2. Le client demande de **démarrer un flux OAuth**. Le moteur renvoie une adresse à ouvrir dans le navigateur ; l'utilisateur s'authentifie chez Google.
3. Le moteur reçoit l'autorisation, obtient les jetons et les range **dans le trousseau** (P9), sous une clé qui contient l'identifiant du profil et du compte (D-10).
4. Le moteur **teste la connexion** avec ces jetons, puis **enregistre le compte** avec la référence du secret. Un événement « compte ajouté » est émis.
5. La **première synchronisation** démarre (S4).

### S2 — Ajouter un compte Yahoo avec un mot de passe d'application

La préparation détecte le provider Yahoo, qui ne propose que le mot de passe d'application. Le client collecte le mot de passe et le **transmet une seule fois** au moteur, qui le range dans le trousseau et ne le renvoie jamais. Le reste est identique à S1.

### S3 — Ajouter un compte « Autre »

La détection tombe sur le provider de repli (D-05). Il propose des serveurs déduits du domaine (`imap.<domaine>`, `smtp.<domaine>`), que l'utilisateur corrige si besoin. Le **test de connexion** valide la configuration **sans rien enregistrer**. Ce n'est qu'à l'ajout que la configuration des serveurs est stockée **dans le compte** (7.9).

### S4 — La première synchronisation

1. Le moteur lit la **liste des boîtes** et déduit leurs rôles (réception, envoyés…) par les attributs standard, avec l'aide du provider si nécessaire.
2. Il télécharge les **en-têtes des 90 derniers jours** de la boîte de réception d'abord (D-21), du plus récent au plus ancien ; l'interface peut afficher la liste dès les premiers lots.
3. Il télécharge ensuite les en-têtes des autres boîtes, puis précharge les **corps** des messages les plus récents.
4. Pour chaque boîte, il enregistre un **repère** (numéro de validité, dernier numéro vu, compteur de modifications si disponible) qui servira à la synchronisation incrémentale (D-22).
5. Des événements de progression sont émis tout du long ; un événement « terminée » conclut.

### S5 — Un nouveau message arrive en temps réel

La connexion en attente (IDLE) sur la boîte de réception reçoit l'annonce du serveur. Le moteur lit les en-têtes du nouveau message sur sa seconde connexion, crée le **message** et son **apparition**, calcule son **fil** (D-15), puis émet l'événement **« nouveau message reçu »**. Le CLI ou l'interface peuvent notifier ; l'agent d'IA, un jour, s'y abonnera. Aucune de ces utilisations n'est connue du moteur.

### S6 — Lire un message hors ligne

L'utilisateur est sans connexion. La requête de lecture du corps interroge la base locale : si le **corps est présent** (message récent, préchargé), il est affiché immédiatement. S'il est à l'état **en-tête seul**, la requête signale que le corps n'est pas disponible hors ligne, et le téléchargement sera tenté quand la connexion reviendra. Les pièces jointes non téléchargées ne sont pas disponibles.

### S7 — Déplacer des messages hors ligne

1. L'utilisateur déplace trois messages vers un dossier. Le moteur applique le **changement local immédiatement** (les apparitions changent, l'interface réagit) et enregistre **trois actions en attente**, avec leurs clés d'idempotence.
2. À la reconnexion, la file est rejouée **dans l'ordre**. Chaque déplacement utilise l'opération atomique du serveur si l'extension MOVE est présente, sinon une copie suivie d'un marquage de suppression.
3. Le serveur confirme ; les actions passent à *réussie* ; la synchronisation met à jour les numéros côté serveur des nouvelles apparitions.

### S8 — Un message supprimé ailleurs

Pendant que l'utilisateur est hors ligne, il marque comme lu un message qui, entre-temps, a été supprimé du serveur depuis un autre appareil. À la reconnexion, la synchronisation descendante **retire** le message du miroir (le serveur gagne, D-23). L'action en attente visant ce message **échoue définitivement** ; l'effet local est déjà sans objet ; l'utilisateur reçoit une notification explicite plutôt qu'un silence.

### S9 — Envoyer un message, avec annulation

1. L'utilisateur envoie. Le brouillon devient une **action d'envoi** dans la file, avec un délai d'annulation de quelques secondes.
2. Dans ce délai, il peut annuler : le message retourne dans les brouillons.
3. Le délai écoulé, le moteur envoie par SMTP avec l'**identité** choisie.
4. Il consulte la particularité du provider : si le serveur enregistre lui-même la copie, MIRA n'ajoute rien ; sinon, MIRA ajoute le message dans la boîte des envoyés.
5. Il émet « envoi parti », puis supprime le brouillon.

### S10 — Un jeton expire en pleine opération

Le moteur tente de rafraîchir le jeton de façon automatique et silencieuse. Si cela réussit, l'opération continue sans que personne s'en aperçoive. Si le rafraîchissement échoue (jeton révoqué, par exemple), le compte passe à l'état **authentification requise**, un événement le signale, et les **actions en attente restent en file** sans échouer. Une fois l'utilisateur ré-authentifié, la file reprend.

### S11 — Un message Gmail à plusieurs libellés

Gmail présente le même message dans « Réception » et « Travail ». À la synchronisation, le provider extrait de chaque apparition l'**identifiant stable** du serveur. Le moteur reconnaît qu'il s'agit du **même message** (D-13) : il n'en crée qu'un, avec **deux apparitions**. La boîte unifiée l'affiche une fois ; marquer « lu » applique le drapeau aux deux apparitions (D-14) ; le retirer de « Travail » supprime seulement cette apparition.

### S12 — Remonter au-delà de 90 jours, et chercher

L'utilisateur fait défiler jusqu'à la limite de la fenêtre et demande « remonter plus loin ». Le moteur lance une **tâche longue** : téléchargement des en-têtes de la période précédente, progression par événements, annulable. Les fils se complètent au fur et à mesure. S'il cherche un mot qui n'est pas dans la fenêtre locale, il peut lancer une **recherche étendue au serveur**, annoncée comme telle et annulable.

### S13 — Un serveur renumérote une boîte

À la synchronisation incrémentale, le numéro de validité d'une boîte a changé. Le moteur marque ses apparitions comme **à revalider**, relit la boîte dans la fenêtre, **recoupe** les messages par identifiant stable puis, à défaut, par `Message-ID`, date et taille, recrée les apparitions, et émet l'alerte « boîte resynchronisée ». En cas de doute sur une correspondance, il garde **deux messages** plutôt qu'un (P7). Les actions en attente visant cette boîte sont relues contre les nouvelles apparitions.

### S14 — Supprimer un compte

L'utilisateur supprime un compte. Le moteur **interrompt** sa synchronisation, **efface** ses données locales (miroir, brouillons, actions en attente) et ses **secrets du trousseau**, puis émet l'événement « compte supprimé ». **Rien n'est modifié sur le serveur.** Les autres comptes du profil ne sont pas touchés.

---
## 14. Invariants et règles de cohérence

Un invariant est une propriété qui doit être **vraie à tout moment** (ou après chaque opération complète), quelle que soit la façon dont le moteur a été utilisé. Ce chapitre les rassemble pour servir de liste de contrôle : en revue de code comme en test, une violation d'invariant est un bogue.

### 14.1 Invariants d'architecture

1. Le cœur ne contient aucun nom de provider et aucun test d'identité de provider (D-06).
2. Un provider ne dépend que du vocabulaire commun (D-33).
3. Un provider ne réalise aucune entrée-sortie et ne garde aucun état (D-07).
4. Le moteur n'a aucune notion d'écran, de fenêtre ou de page (P1).
5. Toute bibliothèque externe est utilisée uniquement à travers son module d'encapsulation (D-34).
6. Tout ce que peut faire l'interface peut être fait par le CLI (P1).

### 14.2 Invariants de données

7. Un message a **un seul identifiant interne**, qui ne change jamais pour une raison venant du serveur (D-13).
8. Un message a **au moins une apparition** tant qu'il n'est pas supprimé définitivement.
9. Chaque apparition référence une **boîte du même compte** que le message.
10. Un message appartient à **un seul compte**, un compte à **un seul profil**.
11. Un compte a **au moins une identité**.
12. Les drapeaux visibles par l'utilisateur sont **portés par le message**, jamais par l'apparition (D-14). Le drapeau interne « supprimé » est le seul qui reste attaché à l'apparition.
13. Un fil est **dérivé** : le supprimer ne perd aucune information, on peut toujours le recalculer (D-15).
14. La source brute d'un message présent localement est **conservée** ; l'analyse est un cache reconstruisible (D-16).
15. Aucun secret n'est présent dans la base, dans les journaux, dans les événements ni dans les réponses de l'API (P9).
16. Le miroir peut toujours être jeté et reconstruit sans perte de configuration ni d'intention (8.1).

### 14.3 Invariants de synchronisation

17. Une action est **écrite dans la base avant d'être tentée** ; aucune action ne vit uniquement en mémoire.
18. Une action rejouée ne produit **pas deux effets** (idempotence).
19. Les actions d'un même compte sont rejouées **dans l'ordre**.
20. Un repère de boîte n'est avancé **qu'après** l'écriture des données qu'il couvre.
21. En cas de désaccord entre le miroir et le serveur, **le serveur gagne** ; les actions en attente sont rejouées par-dessus (D-23).
22. Une erreur définitive sur une action **annule son effet local** et en informe l'utilisateur ; elle n'est jamais silencieuse.
23. Quand le moteur hésite sur l'identité de deux messages, il en **garde deux** (P7).
24. Il y a au plus **une tâche de synchronisation active par compte**.
25. Le nombre de connexions par compte ne dépasse jamais le plafond indiqué par le provider (ou la valeur par défaut prudente).
26. Un envoi n'est **jamais effectué deux fois** sans que l'utilisateur en ait été informé et ait décidé (voir l'ambiguïté signalée en 10.8).

### 14.4 Invariants de contrat

27. Aucune lecture ordinaire ne passe par le réseau (P5).
28. Toute écriture rend la main sans attendre le réseau.
29. Toute liste est paginée par curseur.
30. Tout identifiant exposé est opaque pour le client.
31. Toute erreur a une catégorie, un code stable et un indicateur « réessayable ».
32. Les événements sont numérotés et ne contiennent ni secret ni contenu de message.
33. Les clients vérifient la version du contrat à l'ouverture.

### 14.5 Comment les utiliser

Chacun de ces invariants doit pouvoir se traduire en **test automatisé** : un test d'architecture (l'arbre de dépendances respecte la règle), des tests de propriété sur les données (après une suite quelconque d'opérations, les invariants 7 à 12 tiennent), des scénarios de synchronisation rejoués contre un faux serveur (invariants 17 à 26), et des tests de contrat (27 à 33). Les invariants non testables sont des vœux : ils doivent être reformulés ou supprimés.

---

## 15. Sujets restant à traiter

La conception a traité, dans l'ordre, le périmètre, le modèle de domaine, le découpage en blocs, le système de providers, la synchronisation et le contrat d'API. Les sujets suivants **n'ont pas encore été discutés** avec le porteur. Pour chacun, ce chapitre rappelle ce que dit le brief, énumère les questions, et esquisse des pistes à titre de simple point de départ : toutes les pistes ci-dessous sont **PROPOSÉES** ou **OUVERTES**, aucune n'est décidée.

### 15.1 Le stockage

**Ce que dit le brief [BRIEF]** : SQLite local pour les comptes, dossiers et messages ; recherche en texte intégral FTS5 ; extension `sqlite-vec` pour l'indexation sémantique ; cache des pièces jointes.

**Ce qui est déjà fixé** : un espace de données par profil (D-10) ; la séparation en trois familles (D-18) ; la source brute compressée (D-16) ; l'identité interne des messages avec table de correspondance (D-13).

**Questions ouvertes :**

- la **forme des tables** : comment représenter le message, ses apparitions, la table de correspondance des identifiants serveur, les actions en attente ;
- l'**organisation des fichiers** : une base unique par profil, ou séparer la source brute et le cache de pièces jointes dans des fichiers ou une base à part (la source brute peut devenir volumineuse) ;
- la **stratégie de migration** du schéma, et la règle selon laquelle le miroir peut être migré en le jetant alors que la configuration et les intentions doivent l'être sans perte ;
- l'**indexation** en texte intégral : quels champs, quelle langue, comment reconstruire l'index ;
- la **place des représentations vectorielles** pour l'IA : dans la même base ou à part ;
- la **taille attendue** (D-32), qui conditionne tout ce qui précède ;
- la **concurrence d'accès** à la base (un écrivain, plusieurs lecteurs) et son lien avec le verrou de profil (D-27).

### 15.2 L'authentification et les secrets

**Ce que dit le brief [BRIEF]** : OAuth2 pour Gmail et Outlook, mot de passe d'application ou TLS avec authentification de base pour les autres ; secrets dans le trousseau SecretService.

**Ce qui est déjà fixé** : les secrets ne sortent jamais du trousseau (P9) ; ils sont rangés par profil et par compte (D-10) ; le rafraîchissement d'un jeton est automatique, et son échec fait passer le compte à l'état « authentification requise » (7.7).

**Pistes proposées :**

- le flux OAuth2 de type **code d'autorisation avec PKCE**, avec une **redirection vers une adresse locale** reçue par le moteur : le moteur renvoie une adresse à ouvrir dans le navigateur, écoute le retour, puis échange le code contre les jetons. Ce schéma évite d'embarquer un secret d'application et correspond à ce qui se fait pour les clients de bureau ;
- la mise en tampon des actions pendant une ré-authentification (S10).

**Questions ouvertes :**

- **l'identifiant d'application** (*client id*) de MIRA auprès de Google et de Microsoft : qui l'enregistre, où il est configuré, et comment on gère les restrictions que ces fournisseurs imposent aux applications non vérifiées ;
- la gestion du **trousseau indisponible** (session sans trousseau, par exemple en environnement minimal) : le moteur refuse-t-il, ou propose-t-il une alternative explicite ? Le principe actuel est qu'il ne se replie **jamais** sur un stockage en clair ;
- la **durée de vie** des jetons et la fréquence de rafraîchissement ;
- l'authentification des serveurs à **certificat particulier** (serveur d'entreprise), et la décision de confiance qu'elle demande à l'utilisateur.

### 15.3 La concurrence et la résilience

**Ce que dit le brief [BRIEF]** : asynchronisme avec `tokio`.

**Ce qui est déjà fixé** : une tâche de synchronisation par compte (invariant 24) ; reconnexion avec délai croissant ; reprise après plantage (10.11) ; plafonds de connexions par provider.

**Pistes proposées :** une **tâche** par compte, avec ses connexions, supervisée par le moteur ; une **file** d'actions par compte ; des **canaux** pour émettre les événements ; un **arrêt propre** à délai borné ; une **annulation coopérative** des tâches longues.

**Questions ouvertes :**

- comment les tâches échangent avec la base (accès sérialisé ?) ;
- l'ordre de **priorité** entre comptes et entre boîtes quand les ressources sont limitées ;
- le comportement quand **plusieurs profils** sont actifs (7.11) ;
- la **détection du réseau** (hors ligne, retour en ligne) : écoute du système, ou simple constat d'échec de connexion ;
- le moteur doit-il pouvoir continuer à synchroniser et notifier quand **l'interface est fermée** ? C'est le cœur de D-30.

### 15.4 La sécurité du rendu HTML

**Ce que dit le brief [BRIEF]** : rendu HTML dans une WebView avec **JavaScript désactivé** et **blocage par défaut des ressources distantes** et des pixels espions.

**Pistes proposées :**

- faire l'**assainissement du HTML dans le moteur** (et pas seulement dans la WebView), de sorte que tous les clients, CLI compris, reçoivent un contenu déjà sûr, c'est ce que suppose le contrat d'API (« contenu HTML sécurisé » avec la liste des ressources bloquées) ;
- défense en profondeur : assainissement dans le moteur **et** restrictions dans la WebView ;
- une liste de **confiance** par message et par expéditeur pour charger les images ;
- examiner l'idée d'un **relais d'images** (11.6).

**Questions ouvertes :**

- la **politique** précise sur les liens (ouverture dans le navigateur, avertissement sur les adresses déguisées) ;
- le traitement des **pièces jointes dangereuses** (types exécutables, archives) ;
- l'affichage des **messages chiffrés ou signés** (PGP, S/MIME), qui n'est pas en version 1 mais qui conditionne la conservation de la source brute.

### 15.5 Le bus d'événements et l'agent d'IA

**Ce que dit le brief [BRIEF]** : un bus d'événements interne émettant des événements structurés comme « nouveau message reçu » ; recherche sémantique sur des représentations vectorielles stockées dans SQLite ; résumés de fils, tri et priorisation, aide à la rédaction, modèles locaux ou distants.

**Ce qui est déjà fixé** : l'événement « nouveau message reçu » existe dans le contrat (12.5) ; l'agent est un **client** du moteur, qui s'abonne aux événements et utilise les mêmes requêtes.

**Questions ouvertes :**

- **où** vivent les représentations vectorielles (même base ou à part) et comment elles se synchronisent avec le miroir (que devient un vecteur quand on jette et reconstruit le miroir ?) ;
- le **respect de la vie privée** : quels contenus sont autorisés à quitter la machine quand un modèle distant est utilisé ;
- l'**ordre de calcul** des représentations (en arrière-plan, après synchronisation ?) ;
- les **actions** que l'agent peut déclencher (lecture seule, ou aussi des écritures comme archiver) et la **confirmation** de l'utilisateur.

### 15.6 La liaison avec Flutter (D-30)

**Ce que dit le brief [BRIEF]** : communication en processus entre Dart et Rust par `flutter_rust_bridge`.

**Le point ouvert** : le porteur a voulu que le moteur fonctionne **sans interface**, ce qui rouvre la question de la forme de la liaison. Deux voies :

| Voie | Principe | Avantages | Inconvénients |
|---|---|---|---|
| **Appel direct** | L'interface charge la bibliothèque du moteur dans son propre processus | Simple, rapide, conforme au brief | Le moteur s'arrête avec l'interface ; pas de notifications fenêtre fermée |
| **Processus séparé** | Le moteur tourne en arrière-plan, l'interface s'y connecte | Notifications et synchronisation continues, plusieurs clients simultanés | Plus complexe : transport, cycle de vie, sécurité de la connexion |

Comme le contrat d'API est le même dans les deux cas, la décision peut attendre, mais il faudra la prendre avant de concevoir la concurrence (15.3). Le verrou de profil (D-27) est une solution provisoire qui couvre le cas de l'appel direct.

### 15.7 Les tests et l'observabilité

**Pistes proposées :** les cinq niveaux décrits en 5.8 (providers par tables, cœur avec faux providers, synchronisation avec faux serveur, CLI de bout en bout, tests d'architecture) ; un **journal structuré** avec niveaux et sans secret ; des **identifiants de diagnostic** dans les erreurs internes ; un export de diagnostic (12.4).

**Questions ouvertes :** comment fabriquer un **faux serveur IMAP** réaliste (le construire, ou utiliser un serveur de test réel en conteneur) ; quels **comptes réels** servent aux tests manuels ; quelle politique de **couverture**.

### 15.8 La distribution

**Ce que dit le brief [BRIEF]** : un client Linux natif ; aucun format de distribution n'est fixé.

**Questions ouvertes :** **Flatpak**, **AppImage**, paquets de distribution ; leur effet sur l'accès au trousseau, au réseau et aux dossiers (un bac à sable restreint les accès) ; la compilation de la liaison Flutter ; la mise à jour. Le choix de modules statiques (D-02) simplifie fortement ces questions.

---

## 16. Risques et questions ouvertes

### 16.1 Risques

| Risque | Gravité | Mitigation |
|---|---|---|
| **Identité des messages mal reconnue** (fusion à tort ou doublons massifs) | Haute | Règle de prudence (P7), identifiant stable du serveur d'abord, tests de renumérotation et de déplacements |
| **Perte ou doublon à l'envoi** après un plantage | Haute | État « tentative en cours » persistant, vérification au redémarrage, confirmation de l'utilisateur en cas de doute (10.8) |
| **Perte de données locales irremplaçables** (brouillons, actions en attente) | Haute | Séparation en familles (D-18), persistance avant application, sauvegarde (D-29) |
| **Changements chez les fournisseurs** (adresses OAuth, limites, règles d'authentification) | Moyenne | Profils de provider en données, tests réels réguliers, valeurs à vérifier signalées |
| **Courbe d'apprentissage de Rust** | Moyenne | Conception en interfaces et responsabilités ; analogies avec le C++ (annexe B) ; avancée par petites briques testables |
| **Volume de la source brute** (espace disque) | Moyenne | Compression, pièces jointes à la demande, fenêtre locale (D-21), purge du cache |
| **Complexité de la synchronisation** (renumérotations, extensions capricieuses) | Moyenne | Stratégies par capacités, faux serveur de tests, démarrage par le cas simple |
| **Particularités de Gmail** (libellés, copie des envoyés, « Tous les messages ») | Moyenne | Couche de traduction du provider, optimisation reportée (D-24), tests sur compte réel |
| **Dérive de périmètre** (contacts, règles, IA, agenda) | Moyenne | Découpage v1 et plus tard (D-28), non-objectifs écrits (1.4) |
| **Choix de liaison Flutter tardif** (D-30) | Moyenne | Contrat d'API indépendant du transport ; décision avant la conception de la concurrence |
| **Restrictions des applications OAuth non vérifiées** | Moyenne à haute | À instruire tôt (15.2) |

### 16.2 Points ouverts à trancher

| ID | Question | Où |
|---|---|---|
| D-27 | Un seul moteur actif par profil (verrou) ? | 12.10 |
| D-28 | Découpage des commandes v1 et plus tard | 12.10 |
| D-29 | Sauvegarde et export en version 1 ? | 12.10 |
| D-30 | Liaison avec Flutter : appel direct ou processus séparé ? | 15.6 |
| D-32 | Taille cible : nombre de comptes et de messages | 10.3, 15.1 |
| D-17 | Second libellé sur provider à dossiers : refus ou déplacement (proposition à valider) | 9.10 |
| D-33, D-34 | Vocabulaire commun et découpage interne du cœur (propositions à valider) | 5.3, 5.6 |
| — | Détection de l'hébergeur par DNS pour les domaines personnalisés | 6.6 |
| — | Envoi interrompu : comportement exact au redémarrage | 10.8 |
| — | Politique de relais d'images | 15.4 |

### 16.3 Ordre de traitement suggéré

Pour lever les incertitudes dans l'ordre où elles bloquent le plus de choses :

1. **Taille cible (D-32)** : elle conditionne le stockage et la fenêtre.
2. **Liaison avec Flutter (D-30) et verrou (D-27)** : ils conditionnent la concurrence.
3. **Stockage** (15.1) : le modèle de domaine étant fixé, la forme des tables peut être discutée.
4. **Authentification** (15.2) : prérequis pour tester sur de vrais comptes.
5. **Sécurité du rendu** (15.4) et **IA** (15.5) : plus tard, car ils s'appuient sur le reste.

---

## 17. Feuille de route

Cette feuille de route reprend les cinq étapes du brief en tenant compte des décisions d'architecture. Elle est **PROPOSÉE** : aucune étape n'a été validée.

| Phase | Contenu | Résultat vérifiable |
|---|---|---|
| **0. Conception** | Ce document, complété des sujets du chapitre 15 | Registre des décisions sans point bloquant |
| **1. Fondations** | Vocabulaire commun, interface provider, registre, quatre providers (descriptions), assemblage, squelette du CLI | `mira provider detect <adresse>` répond correctement ; tests des providers par tables ; test d'architecture sur les dépendances |
| **2. Lecture IMAP et analyse MIME** | Client IMAP encapsulé (connexion, liste des boîtes, en-têtes, corps), analyse MIME, authentification par mot de passe d'abord | Le CLI liste les boîtes et affiche un message d'un compte réel |
| **3. Persistance** | Base SQLite par profil, modèle de domaine, recherche en texte intégral, cache de pièces jointes | Le CLI lit et cherche hors ligne |
| **4. Synchronisation** | Descente par étapes, stratégies par capacités, renumérotation, temps réel, file d'actions, résolution de conflits | Scénarios S4 à S13 passent contre un faux serveur, puis contre des comptes réels |
| **5. Envoi, OAuth, trousseau** | Client SMTP, brouillons, envoi différé, flux OAuth, SecretService | Envoi réel sans doublon ; compte Gmail et compte Outlook par OAuth |
| **6. Liaison Flutter** | Décision D-30, liaison, première interface | L'interface exécute le même contrat que le CLI |
| **7. Intégration Linux et sécurité** | Notifications, assainissement HTML, relais ou blocage d'images, restrictions de distribution | Rendu sans JavaScript, ressources bloquées par défaut |
| **8. Agent d'IA** | Représentations vectorielles, recherche sémantique, résumés, aide à la rédaction | Abonnement à l'événement « nouveau message reçu » |

**Correspondance avec le brief.** Les phases 1 à 2 prolongent l'étape 1 du brief ; la phase 3 correspond à l'étape 2 ; la phase 6 à l'étape 3 ; la phase 7 à l'étape 4 ; la phase 8 à l'étape 5. Les phases 4 et 5 précisent le contenu de l'étape 1 et de l'étape 2, que le brief laissait en bloc. La différence majeure tient au **CLI**, qui devient le premier client et permet de **valider le moteur avant toute interface** : l'étape 3 du brief arrive plus tard, sur un moteur déjà éprouvé.

---

## 18. Annexes

### Annexe A — Glossaire

| Terme | Définition |
|---|---|
| **Action en attente** | Opération voulue par l'utilisateur, appliquée en local mais pas encore confirmée par le serveur |
| **Agent d'IA** | Futur client du moteur qui s'abonne aux événements et exploite les messages pour l'assistance |
| **Apparition** | Présence d'un message dans une boîte précise, avec son numéro côté serveur |
| **Assemblage** | Bloc qui crée le registre, y enregistre les providers et démarre le moteur |
| **Backend de messagerie** | Interface de haut niveau, prévue mais pas construite, qui permettrait à un provider de ne pas utiliser IMAP |
| **Boîte** | Vue d'appartenance réelle sur les messages d'un compte (dossier ou libellé côté serveur) |
| **Capacité** | Fonction qu'un serveur annonce savoir faire (une extension IMAP, par exemple) |
| **Compte** | Connexion d'un profil à un serveur de messagerie, via un provider |
| **Contrat d'API** | Ensemble des commandes, requêtes, événements et erreurs que le moteur offre à ses clients |
| **Cœur** | Partie du moteur qui ne connaît aucun provider concret |
| **CONDSTORE, QRESYNC** | Extensions IMAP permettant de ne récupérer que ce qui a changé depuis un repère |
| **Crate (caisse)** | Unité de compilation de Rust, équivalent d'une cible CMake |
| **DKIM** | Mécanisme de signature qui permet de vérifier l'origine d'un message |
| **Drapeau** | Information attachée à un message : lu, suivi, répondu, transféré |
| **Fil** | Regroupement des messages d'une même conversation, calculé par le moteur |
| **IDLE** | Extension IMAP où le serveur prévient le client des changements en temps réel |
| **Identité** | Adresse d'envoi d'un compte, avec nom et signature |
| **IMAP, SMTP** | Protocoles de réception et d'envoi du courrier |
| **Intention locale** | Donnée qui n'existe que localement et doit être conservée (brouillon, action en attente) |
| **Miroir** | Copie locale de l'état du serveur, reconstructible |
| **Message** | Courrier existant une seule fois dans la base, quel que soit le nombre de boîtes où il apparaît |
| **OAuth2** | Méthode d'authentification par jetons, sans partager le mot de passe |
| **OBJECTID** | Extension IMAP fournissant des identifiants stables aux messages, fils et boîtes |
| **Offline-first** | Principe selon lequel l'application fonctionne d'abord en local et se synchronise ensuite |
| **Outbox (boîte d'envoi)** | Partie de la file d'actions qui contient les envois |
| **Profil** | Espace isolé de données et de secrets (un seul implicite aujourd'hui) |
| **Provider** | Description d'un fournisseur de messagerie : profil de données plus particularités |
| **Registre** | Collection des providers connus, avec détection et repli |
| **Repère** | Information mémorisée par boîte pour savoir ce qui a changé côté serveur |
| **Source brute** | Texte exact d'un message tel que le serveur l'envoie |
| **SPECIAL-USE** | Extension IMAP qui marque le rôle d'un dossier (envoyés, corbeille…) |
| **Trousseau** | Stockage sécurisé des secrets du système (SecretService) |
| **UID, UIDVALIDITY** | Numéro d'un message dans une boîte, et numéro de validité de cette numérotation |
| **Vue calculée** | Requête sur les messages (boîte unifiée, non lus, recherche) |
| **Vue d'appartenance** | Boîte réelle : les messages en font partie parce que le serveur le dit |

### Annexe B — Correspondance entre Rust et C++

| Rust | Équivalent C++ | Remarque |
|---|---|---|
| Crate | Cible CMake | Se compile séparément en bibliothèque |
| Workspace | Projet CMake | Regroupe les cibles |
| `Cargo.toml` | `CMakeLists.txt` | Déclare dépendances et options |
| Module | Espace de noms et fichiers | `pub` marque ce qui est public |
| Struct et ses méthodes | Classe | Données et comportement |
| Trait | Classe abstraite pure | Comme `IGame`, `IGfx` |
| `dyn Trait` | Pointeur ou référence vers l'interface | Appel virtuel |
| `Option` | `std::optional` | Valeur absente ou présente |
| `Result` | `std::expected` | Succès ou erreur typée, sans exception |
| Propriété et emprunts | RAII, `unique_ptr`, références | Vérifiés **à la compilation** |
| `async` et `tokio` | Coroutines et un moteur d'exécution (type Boost.Asio) | Tâches concurrentes sur peu de fils |
| `cargo` | CMake plus un gestionnaire de dépendances | Un seul outil |
| `unsafe` | Code sans filet | Isolé et rare |
| Bibliothèque statique | `.a` | Liée dans l'exécutable final |

### Annexe C — Sources

Les sources consultées pendant l'étude des fonctionnalités (chapitre 11) sont les suivantes. Les contenus ont été paraphrasés ; aucune citation longue n'a été reprise.

- Zoho, présentation des fonctions d'un client de messagerie : https://www.zoho.com/workplace/articles/email-client.html
- EngageBay, article sur les clients de messagerie : https://www.engagebay.com/blog/email-client/
- It's FOSS, meilleurs clients de messagerie sous Linux : https://itsfoss.com/best-email-clients-linux.md
- Canary Mail, comparatif des clients pour Mac : https://canarymail.io/blog/best-email-clients-for-mac
- Comparatif des logiciels clients de messagerie : https://www.rajeshkumar.xyz/blog/?p=1635
- IETF, brouillon *IMAP Extensions Suggestions* : https://ftp.sjtu.edu.cn/pub/internet-drafts/draft-ietf-mailmaint-imap-extensions-suggestions-02.html
- Suped, pixels espions et défenses : https://www.suped.com/email-deliverability/answers/how-do-tracking-pixels-in-html-emails-work-and-can-they-be-removed
- Palisade, RFC 8058 et désabonnement en un clic : https://www.palisade.email/learning/one-click-unsubscribe.md

Autres sources internes : le brief du projet MIRA, et le sujet du projet Arcade (document de référence « Arcade : a retro platform », version 2.3.1).

### Annexe D — Journal chronologique des échanges

Pour mémoire, l'enchaînement des échanges de conception a été le suivant.

1. Présentation du brief MIRA et du projet Arcade ; première proposition d'architecture en Rust avec providers modulaires, mise en garde sur l'absence d'interface binaire stable en Rust, quatre options.
2. Question sur le C++ ; analyse des gains et des pertes ; confirmation de Rust (D-01).
3. Production d'un premier brouillon de code, que le porteur n'avait pas demandé ; recadrage : la conception se fait sur l'architecture, sans code.
4. Plan de travail en douze sujets ; trois questions de cadrage (moteur sans interface, taille cible, périmètre d'envoi).
5. Recherche sur les fonctionnalités d'un client de messagerie ; synthèse en trois niveaux et liste des extensions IMAP.
6. Choix : moteur en CLI d'abord ; analyse de ce que l'abandon des plugins apporte et enlève (trois niveaux A, B, C).
7. Abandon des plugins dynamiques, frontière interne (D-02, D-03) ; mise au point sur le sens de « crate » en termes de C++ ; validation des trois blocs et de la définition d'un provider.
8. Hiérarchie profil, compte, provider ; choix de l'option 1 (pas d'utilisateur applicatif) avec porte ouverte aux profils (D-09, D-10).
9. Modèle des messages : dossiers contre libellés ; formulation « un message, des boîtes comme vues » (D-12).
10. Liste des entités ; explications sur les drapeaux, les fils, le stockage et l'identité du message ; validation (D-13 à D-16).
11. Synchronisation : descente par étapes, fenêtre de 90 jours, stratégies par capacités, règle de conflit (D-20 à D-24).
12. Clarification sur le cas difficile de Gmail (volume de téléchargement, pas modélisation).
13. Rédaction du contrat d'API à la demande du porteur (D-26).
14. Demande du présent document de référence.

### Annexe E — Le brouillon de code existant

Pendant la conception, un premier brouillon de code a été produit, que le porteur n'avait pas demandé. Il se trouve dans un dossier `mira` de l'espace de travail et contient un workspace Rust avec : un crate de types, un crate d'interface provider avec registre et détection, un moteur minimal, quatre crates de providers (un par fournisseur), et un binaire de démonstration. Il compile et ses tests passent.

**Ce brouillon ne fait pas référence.** Il a été écrit avant que l'architecture soit fixée et il en diffère sur plusieurs points : un crate par provider (alors que l'architecture retient un bloc providers regroupant les modules), des capacités déclarées par le provider (alors que la source de vérité est le serveur, D-08), un provider qui reçoit l'adresse en paramètre pour la configuration des serveurs sans notion de profil et de particularités. Il peut servir de base de travail ou être abandonné ; ce document prime dans les deux cas.

### Annexe F — Mode de révision

Toute modification de ce document suit les règles du chapitre 0 : une nouvelle décision reçoit un identifiant, une décision remplacée est conservée avec sa raison, et les statuts sont mis à jour. Il convient en particulier de réviser, à chaque phase de la feuille de route (chapitre 17), les points ouverts du chapitre 16 qui la concernent.

### Annexe G — Questions fréquentes sur les décisions

Cette annexe répond, en peu de mots, aux objections les plus prévisibles. Chaque réponse renvoie au chapitre qui la justifie.

**Pourquoi ne pas faire des plugins comme dans Arcade ?** Parce qu'un provider ne diffère des autres que par des données et quelques particularités, alors qu'un jeu d'Arcade apportait un comportement complet. Charger du code à l'exécution coûte de la sécurité (du code tiers dans le processus qui détient les jetons), de la complexité (pas d'interface binaire stable en Rust) et du packaging, pour un gain faible (4.5, D-02).

**Pourquoi Rust plutôt que C++ ?** Pour la sûreté mémoire sur l'analyse de contenu hostile, la liaison directe avec Flutter, la maturité de l'écosystème asynchrone et réseau, et parce que la stabilité d'une interface dynamique vient du C, pas du C++ (4.4, D-01).

**Pourquoi un seul exécutable ?** Pour un packaging simple, un moteur qui ne charge aucun code étranger et un comportement identique pour tous les clients (5.5).

**Pourquoi un message unique plutôt qu'une copie par dossier ?** Pour que Gmail, où un message porte plusieurs libellés, ne produise pas de doublons dans la boîte unifiée et pour qu'une suppression ou un drapeau s'applique au vrai message. Les serveurs à dossiers sont le cas particulier à un seul emplacement. Passer plus tard d'un emplacement à plusieurs serait une migration pénible (9.2, D-12).

**Pourquoi garder la source brute de chaque message ?** Pour pouvoir ré-analyser après une correction, afficher la source, lire des en-têtes imprévus, et vérifier des signatures qui exigent les octets exacts. Le prix est de l'espace disque, réduit par la compression et la fenêtre locale (9.8, D-16).

**Pourquoi le serveur gagne-t-il toujours ?** Parce qu'il est la source de vérité de ce qui est en ligne, et qu'un conflit à arbitrer à chaque divergence serait ingérable. Les intentions locales ne sont pas perdues : elles sont rejouées par-dessus (10.9, D-23).

**Pourquoi pas de notion d'utilisateur dans l'application ?** Parce que le système sépare déjà les utilisateurs par session, et que le trousseau et les dossiers de données le suivent. Des profils pourront s'ajouter, grâce aux trois règles d'isolation déjà posées (7.2 et 7.3, D-09 et D-10).

**Pourquoi les providers sont-ils sans entrées-sorties et sans état ?** Pour qu'ils soient testables par de simples tables, interchangeables, et qu'aucun provider ne puisse atteindre le stockage ou le réseau du cœur (6.4, D-07).

**Que fait MIRA avec un serveur qu'il ne connaît pas ?** Il le sert avec le provider « Autre », dont les serveurs sont saisis ou déduits du domaine, et il adapte ses stratégies aux capacités que le serveur annonce (6.5 et 10.14).

**Que se passe-t-il si l'application se ferme au milieu d'une opération ?** Les actions sont écrites avant d'être tentées, les repères ne sont avancés qu'après l'écriture des données, et les téléchargements ne deviennent visibles qu'une fois complets. Le seul cas non résolu est un envoi interrompu entre l'acceptation du serveur et son enregistrement local (10.8 et 10.11).

### Annexe H — Interfaces conceptuelles des modules encapsulés [PROPOSÉ]

En application du principe d'encapsulation (D-34), chaque bibliothèque externe est cachée derrière une interface propre au moteur. Cette annexe décrit, **sans code**, ce que le cœur attend de chaque module. Elle sert de cahier des charges pour choisir ou remplacer une bibliothèque.

| Module | Ce que le cœur attend de lui | Ce qu'il cache |
|---|---|---|
| **Client IMAP** | Se connecter avec chiffrement et authentification donnés ; annoncer les capacités du serveur ; lister les boîtes avec leurs attributs ; sélectionner une boîte et fournir son repère ; récupérer en-têtes, corps et pièces jointes par numéro ; lire les changements depuis un repère ; modifier des drapeaux ; copier, déplacer, ajouter un message ; attendre un événement en temps réel ; rechercher | La bibliothèque choisie, la syntaxe des commandes, l'analyse des réponses |
| **Client SMTP** | Envoyer un message complet avec une identité et une authentification, et renvoyer un succès ou une erreur typée (refus, quota, réseau) | La bibliothèque, le dialogue avec le serveur |
| **Analyse MIME** | Transformer une source brute en structure : en-têtes, parties texte et HTML, pièces jointes, identifiants de contenu ; **tolérer** les messages mal formés sans échouer | La bibliothèque d'analyse |
| **Assainissement HTML** | Produire, à partir d'un HTML quelconque, un HTML sûr sans script, avec la liste des ressources distantes retirées | La bibliothèque de nettoyage |
| **Trousseau** | Stocker, lire et supprimer un secret identifié par profil, compte et type ; signaler clairement qu'il est indisponible ou verrouillé | SecretService et le bus de session |
| **Stockage** | Transactions, requêtes, recherche en texte intégral, migrations, un espace de données par profil | Le moteur de base de données et ses extensions |
| **Horloge et identifiants** | Fournir l'heure et des identifiants uniques, **remplaçables dans les tests** | Les sources système |
| **Réseau** | Établir des connexions chiffrées, avec délais d'attente et vérification des certificats ; remonter les décisions de confiance à l'utilisateur | La pile TLS et la résolution de noms |
| **Chemins et configuration** | Fournir les emplacements de données, de cache et de configuration d'un profil | Les conventions du système |

Deux exigences s'appliquent à tous ces modules :

- aucune de leurs **erreurs propres** ne remonte telle quelle : chacune est convertie en catégorie d'erreur du contrat (12.6) ;
- chacun doit pouvoir être **remplacé par une fausse implémentation** dans les tests (faux serveur IMAP, faux trousseau, horloge figée).

---

*Fin du document de référence. Version 0.1 du 10 octobre 2026.*