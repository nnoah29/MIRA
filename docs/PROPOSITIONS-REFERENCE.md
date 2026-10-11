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
