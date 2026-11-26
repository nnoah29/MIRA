# Modèle de menaces MIRA

Complète `AUDIT-SECURITE-CHECKLIST.md` (surface d'attaque) et `CONSTATS.md`. Méthode : STRIDE, par frontière de confiance.

## Biens à protéger

1. Le contenu du courrier (base locale, brouillons, pièces jointes).
2. Les secrets : mots de passe d'application, jetons OAuth, clés d'API d'IA (trousseau système).
3. La capacité d'envoyer et de supprimer au nom de l'utilisateur.
4. La confidentialité de la lecture (qui a ouvert quoi, adresse IP).

## Attaquants

| Attaquant | Moyen | Ce qu'il cherche |
|---|---|---|
| Expéditeur inconnu | un mail (HTML, MIME, liens, pièces jointes, texte lu par l'IA) | exécuter du code, tracer la lecture, hameçonner, détourner l'IA |
| Serveur distant compromis ou usurpé | réponses IMAP, SMTP, OAuth | injecter des commandes, rétrograder TLS, saturer |
| Autre utilisateur local | socket ou port du démon, fichiers | lire le jeton, piloter le démon |
| Fournisseur d'IA | ce qui lui est envoyé | recueillir du contenu privé |
| Chaîne d'approvisionnement | dépendances, paquets, CI | code malveillant livré |
| Voleur d'appareil | disque | lire la base locale |

## Par frontière

| Frontière | Menace (STRIDE) | Parade en place | Reste |
|---|---|---|---|
| Mail → vue web | E : script, S : fausse page, I : pixel espion | nettoyage côté moteur, CSP, navigation filtrée, images distantes bloquées | vérification réelle par plateforme ; alerte d'hameçonnage (constat 8) |
| Mail → analyseur MIME | D : parties ou imbrication géantes | plafonds de taille, tests de structures hostiles (`mime_structure_limits`) | fuzzing |
| Application → SMTP | T : injection d'en-têtes | `single_line`, `bare_message_id`, adresses validées, tests CRLF y compris pièces jointes | — |
| Moteur → IMAP | T : injection de commandes | `quote`/`quote_mailbox` refusent CR, LF, NUL ; drapeaux en énumération fixe | fuzzing |
| Application → démon | S, E : client local non autorisé | jeton HMAC en temps constant, 0600/0700, plafond de clients | ACL Windows (constat 5) |
| Fichier importé (YAML) | D, T : bombe, alias | alias et doublons refusés, 2 Mo, 500 éléments, imbrication testée | — |
| Mail → IA | T : injection de prompt, I : exfiltration | outils en lecture seule, propositions validées par clic, consentement par type de contenu, masquage, pas de rendu d'image ni de lien dans les réponses | tests avec mail hostile de bout en bout |
| Moteur → fournisseur d'IA | I : fuite de clé | HTTPS (clair seulement en boucle locale), aucune redirection, clé jamais journalisée | — |
| Disque | I : lecture de la base | secrets dans le trousseau, fichiers privés | base non chiffrée (constat 4, décision à prendre) |
| Dépendances | T : composant vulnérable | `cargo deny` (avis compris), workflow hebdomadaire, Dependabot | SBOM, audit Dart |

## Hors périmètre

Un attaquant qui contrôle déjà la session de l'utilisateur (même compte, même processus) : il peut tout lire de toute façon.
