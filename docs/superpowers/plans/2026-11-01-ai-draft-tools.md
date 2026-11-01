# Outils de brouillon pour l'IA — plan d'implémentation

> Exécution inline, TDD (test rouge vu, puis vert), un commit par tâche, jamais `./ci.sh` sans accord, commits au nom de l'utilisateur sans trailer, ne jamais indexer `app/lib/ui/ai_connection_card.dart` ni le retrait de `Tester` dans `en.dart` (travail de l'utilisateur), moteur : `CARGO_BUILD_JOBS=4` et tests ciblés (`--test`), normes `tools/check-norms` (fonction ≤ 30 lignes, ≤ 6 fonctions par fichier, aucun commentaire dans un corps, une ligne `///` avant chaque `fn`).

**Spec** : `docs/superpowers/specs/2026-11-01-ai-draft-tools-design.md` (D1–D9).

## Review Focus
1. Une adresse malformée, un corps énorme, un objet multiligne : refus clair au modèle, aucune proposition émise (T2, T3).
2. Un outil de proposition appelé hors de son mode (`edit_draft` en mode assistant) : refusé, aucune proposition (T3).
3. Une proposition sans aucun type de contenu autorisé : passe (D4) ; `list_identities` sans autorisation : refusé avec pastille « Autoriser… » (T3).
4. Appliquer une proposition dans le composeur puis « Annuler » : le texte d'avant revient exactement, mise en forme comprise (T7).
5. Aucune proposition en fin de tour (l'IA pose une question, ou le fournisseur n'a pas d'outils) : la réponse texte s'affiche, rien n'est modifié (T7).

## T1 — Vocabulaire, `ChatStart.mode`, contrat 1.5
Fichiers : `mira-vocabulary/src/ai/draft.rs` (+ `mod.rs`, `lib.rs`), `delta.rs`, `version.rs` (1.5), `mira-core/src/api/chat_stream.rs` (`#[serde(default)] pub mode: ChatMode`), les 5 constructions de `ChatStart` des tests, les `match` exhaustifs sur `ChatDelta` (`ai_loop_support/turns.rs::shape` → `"proposed"`, `mira-vocabulary/tests/ai_tool_types.rs`, `progress.rs`), tests de version 1.4 → 1.5 et instantané.
Tests : `mira-vocabulary/tests/ai_draft_types.rs` — `chat_mode_defaults_to_assistant`, `chat_mode_serializes_with_a_kind_tag`, `draft_proposal_round_trips`, `draft_proposed_delta_has_snake_case_type`.

## T2 — Outils de proposition et `list_identities`
Fichiers : `mira-core/src/ai/tools/{proposal,compose_draft,edit_draft,list_identities}.rs`, `catalog.rs` (`specs_for(mode) -> Vec<ToolSpec>`, `is_proposal(&str) -> bool`, `kind_of("list_identities") = ToolResult`, `summary_of` : objet pour les outils de brouillon), `run.rs` (dispatch `list_identities`), `run_result.rs` (`proposal: Option<DraftProposal>`, `ToolRun::proposed`), `mod.rs` (`pub use` + `pub fn propose(mode: ChatMode, call: &ToolCall) -> ToolRun`).
Tests (`mira-core/tests/ai_tools_draft_*.rs`) : jeux d'outils par mode ; schémas JSON valides ; `compose_draft` valide → proposition complète, adresses normalisées (`nom <a@b>`), refus d'adresse invalide, corps trop long, objet multiligne, trop de destinataires ; `edit_draft` : exclusivité `selection`, au moins un champ, interdit hors sélection ; avertissement des crochets dans le texte rendu ; `list_identities` rend nom/adresse/défaut/signature bornés.

## T3 — Intégration dans la boucle
Fichiers : `chat/prepare.rs` (`ChatPlan.mode`), `chat/tools_plan.rs` (`with_tools(request, enabled, mode)` : jeu d'outils du mode + `COMPOSE_NOTICE` dans la consigne), `chat/framing.rs` (la consigne), `chat/tool_exec.rs` (`Finished.proposal`, routage des outils de proposition avant `decide_tool`, refus hors mode), `chat/tool_step.rs` (émettre `DraftProposed` avant `ToolFinished`).
Tests (`ai_tools_loop_draft.rs`, deux formats) : ordre `called, proposed, finished:ok` ; résultat renvoyé au modèle = phrase fixe (+ avertissement) ; sans aucun type autorisé la proposition passe, `list_identities` est refusé ; `edit_draft` en mode assistant → `finished:error`, pas de delta `proposed` ; mode composeur sélection n'annonce pas `compose_draft` ; masquage : l'adresse restituée dans la proposition ; journal sans contenu (le titre et le corps n'y figurent pas) ; annulation entre deux appels.

## T4 — Pont
Test `bridge/tests/chat_stream_draft.rs` : une ligne JSON `{"type":"draft_proposed","id":…,"proposal":{…}}` arrive dans le flux.

## T5 — App : flux, consigne, carte dans le fil
`ai_models.dart` (`AiDraftProposal`), `ai_admin.dart` (`_event('draft_proposed')`, `chat(..., mode:)` → `'mode'` dans `ChatStart`), `chat_context.dart` (consigne : date du jour, outils de brouillon, plus de « tu ne peux rien modifier »), `chat_message.dart` (`drafts`), `chat_message_json.dart` (clé `drafts`, optionnelle), `chat_controller.dart` (attache les propositions à la réponse).
Tests : parse du flux, consigne (date, mention des outils), carte attachée, aller-retour JSON, une proposition par appel, la dernière remplace la précédente pour un même appel.

## T6 — App : carte « Brouillon proposé » et ouverture
`ui/chat_draft_card.dart`, `assistant_panel.dart` (carte dans la bulle), `store.dart` (`openProposal(AiDraftProposal)` → `ComposeInit` : nouveau message ou réponse avec citation, signature de l'identité, expéditeur par identité si fournie).
Tests : la carte montre À/objet/extrait ; « Ouvrir » ouvre le composeur prérempli (nouveau, réponse, signature, une seule formule finale) ; l'historique garde la carte.

## T7 — App : `DraftAiController`
`state/draft_ai_controller.dart` : `Future<void> run(String instruction, {String? selection})`, `apply(AiDraftProposal)`, `undo()`, `busy`, `reply` (texte quand pas de proposition), `error`, `canUndo`. Instantané `(text, flags, subject, to, cc)`. Application : objet, destinataires, corps **avant la signature** (`sigText`), ou remplacement de la plage sélectionnée.
Tests : application du corps (signature gardée), objet/destinataires, sélection, annulation exacte (mise en forme), aucune proposition → `reply`, erreur traduite, pas d'application après `dispose`.

## T8 — App : interface du composeur
`ui/composer.dart` + `ui/composer_ai_bar.dart` : bouton « Écrire avec l'IA » (barre du bas), champ d'instruction, état en cours, bouton Annuler, réponse texte ; `_FmtBar` : actions de sélection (Reformuler, Raccourcir, Corriger, Traduire en anglais).
Tests de widgets : bouton absent en démonstration, instruction → proposition appliquée, Annuler, actions de sélection, largeur étroite.

## T9 — Finition
`en.dart` et `engine_errors.dart` (aucun nouveau code), `ARCHITECTURE-IMPLEMENTEE.md` §6.10, `docs/dev/DECISIONS-AGENT.md` AG-86, normes, clippy, suites complètes moteur et Flutter, revue finale indépendante, notification sur le bureau.
