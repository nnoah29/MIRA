# Invariants et scénarios → tests

| # | Invariant | Test(s) |
|---|---|---|
| 1 | Le cœur ne nomme aucun provider (D-06) | manquant |
| 2 | Un provider ne dépend que du vocabulaire (D-33) | manquant |
| 3 | Provider sans I/O ni état (D-07) | manquant |
| 4 | Aucune notion d'écran (P1) | manquant |
| 5 | Bibliothèque externe via son module d'encapsulation (D-34) | manquant |
| 6 | Tout ce que fait l'interface, le CLI le fait (P1) | manquant |
| 7 | Identifiant interne unique et stable (D-13) | manquant |
| 8 | Message : au moins une apparition | manquant |
| 9 | Apparition → boîte du même compte | manquant |
| 10 | Message → un compte ; compte → un profil | manquant |
| 11 | Compte : au moins une identité | manquant |
| 12 | Drapeaux utilisateur au niveau du message (D-14) | manquant |
| 13 | Fil dérivé, recalculable (D-15) | manquant |
| 14 | Source brute conservée, analyse en cache (D-16) | manquant |
| 15 | Aucun secret en base, journaux, événements, API (P9) | manquant |
| 16 | Miroir jetable sans perte (8.1) | manquant |
| 17 | Action écrite avant d'être tentée | manquant |
| 18 | Action rejouée : pas deux effets | manquant |
| 19 | Actions d'un compte rejouées dans l'ordre | manquant |
| 20 | Repère avancé après écriture des données | manquant |
| 21 | Le serveur gagne ; actions rejouées (D-23) | manquant |
| 22 | Erreur définitive : effet local annulé, utilisateur informé | manquant |
| 23 | Doute d'identité : garder deux messages (P7) | manquant |
| 24 | Une tâche de synchro par compte | manquant |
| 25 | Connexions ≤ plafond du provider | manquant |
| 26 | Jamais deux envois sans décision de l'utilisateur | manquant |
| 27 | Aucune lecture ordinaire par le réseau (P5) | manquant |
| 28 | Toute écriture rend la main sans réseau | manquant |
| 29 | Toute liste paginée par curseur | manquant |
| 30 | Identifiants exposés opaques | manquant |
| 31 | Erreur : catégorie, code stable, réessayable | manquant |
| 32 | Événements numérotés, sans secret ni contenu | manquant |
| 33 | Version du contrat vérifiée à l'ouverture | manquant |

| Scénario | Description | Test(s) |
|---|---|---|
| S1 | Ajouter un compte Gmail par OAuth | manquant |
| S2 | Ajouter un compte Yahoo (mot de passe d'application) | manquant |
| S3 | Ajouter un compte « Autre » | manquant |
| S4 | Première synchronisation | manquant |
| S5 | Nouveau message en temps réel | manquant |
| S6 | Lire un message hors ligne | manquant |
| S7 | Déplacer des messages hors ligne | manquant |
| S8 | Message supprimé ailleurs | manquant |
| S9 | Envoyer, avec annulation | manquant |
| S10 | Jeton expiré en pleine opération | manquant |
| S11 | Message Gmail à plusieurs libellés | manquant |
| S12 | Remonter au-delà de 90 jours, chercher | manquant |
| S13 | Le serveur renumérote une boîte | manquant |
| S14 | Supprimer un compte | manquant |
