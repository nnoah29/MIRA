# Style de rédaction de l'assistant (lot 2) — design

Date : 2026-11-15 · Statut : rédigé sans questions (l'utilisateur a dit « lance-toi directement ») ; les choix ci-dessous sont des propositions à corriger · Suite de `2026-11-15-ia-memoire-design.md` · Branche `feat/ia-style-redaction`

## 1. Intention

Les brouillons de l'assistant doivent ressembler à l'utilisateur : même registre (tu ou vous), mêmes ouvertures, même longueur, même ponctuation. Le style est appris des mails que l'utilisateur a déjà envoyés.

## 2. Objectifs, non-objectifs, anti-objectifs

- **Objectifs** : un profil de style calculé à la demande ; visible, corrigeable et effaçable ; ajouté à la consigne de rédaction seulement s'il est activé.
- **Non-objectifs** : imitation fine mot à mot ; apprentissage en continu ; style par destinataire ; style par compte ; synchronisation entre appareils.
- **Anti-objectifs** : un contenu de mail qui part chez un tiers pour apprendre le style ; un profil qui contient un nom, une adresse ou un extrait de mail ; un style appliqué sans que l'utilisateur l'ait activé.

## 3. Approches considérées

| Approche | Confidentialité | Qualité | Coût |
|---|---|---|---|
| **A. Statistiques locales** (retenue) : le moteur compte registre, ouvertures, longueurs, ponctuation sur les mails envoyés et en tire une consigne courte | Aucun mail ne sort ; testable sans modèle | Cadre le ton, pas les tournures fines | Faible |
| B. Exemples joints : quelques mails envoyés ajoutés à chaque rédaction | Les mails partent chez le service d'IA à chaque fois | La meilleure imitation | Jetons à chaque requête ; consentement par type |
| C. Résumé fait par un modèle : un modèle lit des mails et décrit le style | Les mails partent une fois | Description riche | Un appel par analyse, consentement |

Retenue : **A**, car elle respecte l'anti-objectif principal ; B et C restent possibles plus tard, derrière un consentement propre.

## 4. Décisions

| # | Décision | Raison |
|---|---|---|
| S1 | Source : les messages de la boîte à rôle `Sent` de chaque compte, les 200 plus récents au plus, corps déjà disponibles (un corps absent hors ligne est ignoré) | Aucun téléchargement forcé ; borne de temps |
| S2 | Avant analyse, le texte est nettoyé : citations (`>`, « Le … a écrit : », « On … wrote: »), signature (`-- `), lignes vides | On mesure ce que l'utilisateur a écrit |
| S3 | Le profil (`StyleProfile`) contient : nombre de mails, registre (`tu`, `vous`, `mixte`, `inconnu`) avec seuil de 70 %, au plus 3 mots d'ouverture (Bonjour, Salut, Hello…) **sans nom propre**, longueur médiane en mots, longueur médiane des phrases, taux de points d'exclamation, emploi d'émojis | Aucune donnée personnelle ; tout est reconstructible |
| S4 | Moins de 5 mails exploitables : profil `inconnu`, rien n'est injecté | Une statistique sur trop peu de mails trompe |
| S5 | Réglages dans `engine_settings` sous `ai_style` : `enabled` (faux par défaut), `notes` (texte libre de l'utilisateur, 500 caractères), profil et date d'analyse | Même stockage que la mémoire |
| S6 | Injection : si activé, un paragraphe de consigne (≤ 600 caractères) est ajouté au message système des discussions d'assistant et de composeur ; il dit d'écrire dans ce registre et cette longueur et cite les notes ; la signature et la formule finale restent ajoutées par l'app (consigne existante) | Même point d'entrée que la mémoire |
| S7 | API (contrat 1.12) : `GetStyleStatus` (lecture), `AnalyzeStyle`, `SetStyleSettings { enabled, notes }`, `ClearStyle` | Une opération par intention |
| S8 | App : section « Style de rédaction » dans les réglages de l'IA : interrupteur, bouton Analyser, profil en phrases lisibles, notes modifiables, Effacer | Visible et corrigeable |

## 5. Laissé au réalisateur

Seuils exacts de détection du registre et des phrases, libellés, ordre des lignes du profil. Les langues autres que le français et l'anglais ne reçoivent pas de détection de registre (profil `inconnu` pour ce critère).
