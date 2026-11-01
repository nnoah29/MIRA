# Outils de brouillon pour l'IA et « Écrire avec l'IA » — design

Date : 2026-11-01 · Statut : implémenté, approuvé par l'utilisateur (« considère tes recommandations comme validées », objectif `/goal`) · Chemin : architectural (contrat 1.4 → 1.5, boucle d'outils, composeur)

## 1. Objectif

L'assistant écrit **à la place d'un brouillon à recopier** : « écris un mail pour dire à mon boss que je finis mon stage mercredi » produit un vrai brouillon (destinataire, objet, corps), sans trous (`[Prénom/Nom]`, `[date]`), sans signature en double, que l'utilisateur relit et envoie lui-même. Un bouton **« Écrire avec l'IA »** dans le composeur écrit ou retouche le brouillon ouvert (instruction globale) et retouche un passage sélectionné (reformuler, raccourcir, corriger, traduire).

Premier cycle du « chantier 4 » (outils d'écriture). **Rien n'est envoyé ni supprimé par l'IA.** Les actions sur la boîte (archiver, déplacer…) et l'envoi sont des cycles suivants.

## 2. Décisions

| # | Décision | Raison |
|---|---|---|
| D1 | Outils de **proposition** : le moteur valide les arguments et émet un delta `DraftProposed` ; **l'app seule applique** le texte. Le moteur n'écrit aucun brouillon. | Le composeur détient le texte vivant et l'enregistre lui-même : deux auteurs se marcheraient dessus. L'annulation reste celle du composeur. |
| D2 | Deux modes de génération, choisis par le client dans `ChatStart.mode` : `Assistant` (défaut) et `Composer { selection }`. Le client choisit seulement parmi des jeux d'outils définis par le moteur. | Un client ne peut pas ajouter d'outil ; `edit_draft` n'existe qu'en mode composeur, `compose_draft` qu'en mode assistant. |
| D3 | Outils : `compose_draft` (assistant), `edit_draft` (composeur), `list_identities` (lecture, les deux modes). | Le modèle doit connaître nom et adresse de l'utilisateur. |
| D4 | Les outils de proposition **n'exigent aucun type de contenu autorisé** : ils n'envoient rien au service (la réponse au modèle est une phrase fixe). `list_identities` rend du `tool_result` : soumis au consentement existant. | Écrire un mail ne doit pas exiger d'autoriser la lecture de la boîte. |
| D5 | Le brouillon ouvert part au service comme partie de contexte `mail_message` (« Mail ouvert »). Aucun nouveau type de contenu. | Pas de nouveau type à autoriser ni à traduire ; c'est le mail en cours d'écriture. |
| D6 | Le moteur **avertit** le modèle quand le texte proposé contient des passages entre crochets (trous probables) : la proposition passe, le résultat demande de la compléter. L'app applique la dernière proposition du tour. | Une règle stricte ferait refuser des crochets légitimes ; l'avertissement règle le cas de la capture. |
| D7 | Consigne ajoutée par le moteur aux modes à outils de brouillon : pas de formule finale ni de signature (l'app l'ajoute), aucun trou, utiliser les outils plutôt que recopier le mail dans la réponse, impossible d'envoyer. Date du jour dans la consigne de l'app. | Évite « Cordialement » en double et `[Votre nom]`. |
| D8 | Dans le chat : une **carte « Brouillon proposé »** (À, objet, extrait) avec « Ouvrir le brouillon » ; rien ne s'ouvre seul. Dans le composeur : la proposition est **appliquée tout de suite** avec un bouton « Annuler » qui restaure le texte d'avant. | Ne pas arracher l'utilisateur à sa lecture ; dans le composeur il a demandé l'écriture. |
| D9 | Contrat 1.5 : `ChatMode`, `DraftProposal`, `ChatDelta::DraftProposed`, `ChatStart.mode`. Le pont relaie le delta tel quel (serde). | Le pont sérialise déjà chaque delta. |

## 3. Moteur

- **Vocabulaire** (`mira-vocabulary/src/ai/draft.rs`) : `ChatMode { Assistant, Composer { selection: bool } }` (défaut `Assistant`, `#[serde(tag = "kind", rename_all = "snake_case")]`) ; `DraftProposalKind { New, Edit }` ; `DraftProposal { kind, to, cc, subject, body, selection, reply_to, identity }` (tous les champs sauf `kind` optionnels) ; `ChatDelta::DraftProposed { id, proposal }`.
- **Cœur** (`mira-core/src/ai/tools/`) : `compose_draft.rs`, `edit_draft.rs`, `list_identities.rs`, `proposal.rs` (adresses, bornes, trous entre crochets, sélection) ; `catalog::specs_for(mode)` ; `is_proposal(name)` ; `propose(mode, call) -> ToolRun` ; `ToolRun.proposal`.
- **Boucle** : `ChatPlan.mode` ; `execute` aiguille les outils de proposition avant la décision de consentement ; un outil hors du jeu du mode est refusé (« outil indisponible dans ce contexte ») ; `finish_call` émet `DraftProposed` juste avant `ToolFinished` quand l'état est `ok`.
- **Bornes** : corps ≤ 50 000 caractères, objet ≤ 300 sur une ligne, ≤ 20 destinataires par liste, adresses valides (`nom <adresse>` ou `adresse`), au moins un champ pour `edit_draft`, `selection` exclusif des autres champs en mode sélection et interdit sinon.
- **Journal et événements** : le journal garde noms et nombres d'appels (inchangé) ; `DraftProposed` est éphémère (flux), jamais écrit ni journalisé ; aucun contenu dans les erreurs.
- **Risque** : aucune requête d'écriture n'est exécutée par un outil ; `risk_of` et la garde « lecture seule » sont inchangés.

## 4. App

- **Flux** : `AiDraftProposal` (parse `draft_proposed`), `AiAdmin.chat(..., mode:)`.
- **Chat** : `ChatMsg.drafts` ; carte dans la bulle ; `MailStore.openProposal` ouvre un composeur prérempli (nouveau message ou réponse avec citation, signature ajoutée par l'app) ; cartes enregistrées dans l'historique des conversations.
- **Composeur** : `DraftAiController` (instruction globale ou action sur sélection → `ChatStart` mode composeur, contexte = brouillon, réponse texte affichée si l'IA pose une question) ; application dans le `ComposeState` (objet, destinataires, corps avant signature, ou remplacement de la sélection) avec instantané pour « Annuler » ; bouton « Écrire avec l'IA » (icône `spark`) dans la barre du bas, champ d'instruction ; actions de sélection dans la barre flottante.
- **Consigne de l'app** : retire « tu ne peux ni envoyer, ni supprimer, ni modifier quoi que ce soit : tu proposes seulement du texte » ; ajoute la date du jour ; décrit les outils de brouillon.

## 5. Tests

Moteur : jeux d'outils par mode, schémas, validation (adresses, bornes, exclusivités), avertissement des crochets, ordre des deltas (`called`, `proposed`, `finished`), outil hors mode refusé, aucun type de contenu requis pour une proposition, `list_identities` soumis au consentement, aucun contenu dans journal/événements, contrat 1.5. App : parse du flux, carte du chat, ouverture du composeur prérempli (nouveau, réponse, signature, date), application et annulation dans le composeur, sélection, question de l'IA affichée, erreurs, i18n.

## 6. Hors périmètre

Envoi par l'IA ; actions sur la boîte ; contacts (ils vivent dans l'app) ; pièces jointes ; synchronisation. Un fournisseur sans outils : la barre du composeur affiche la réponse texte et l'absence de proposition.
