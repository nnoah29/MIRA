//! Exécution d'un appel d'outil : décision, lecture par la façade, résultat masqué puis borné et cadré.

use mira_vocabulary::{ContentKind, ToolCall, ToolState};

use super::ending::Ending;
use super::framing::frame_result;
use super::guard::Guard;
use super::prepare::ChatPlan;
use super::progress::Progress;
use super::tool_consent::{ToolDecision, decide_tool};
use super::truncate::{MAX_INPUT_CHARS, request_chars};
use crate::ai::tools::body::{RESULT_CHARS, bounded};
use crate::ai::tools::{kind_of, run};

const MIN_ROOM_CHARS: usize = 300;
const DENIED_TEXT: &str =
    "accès refusé : l'utilisateur n'a pas autorisé ce type de contenu pour cette connexion";

/// Ce qu'un appel a donné, avant cadrage.
pub(super) struct Finished {
    pub(super) text: String,
    pub(super) state: ToolState,
    pub(super) results: u32,
    pub(super) sources: Vec<String>,
    pub(super) denied: Option<ContentKind>,
    pub(super) sent: Option<ContentKind>,
}

impl Finished {
    /// Échec expliqué au modèle : rien n'a été lu.
    pub(super) fn error(text: &str) -> Self {
        Self {
            text: text.to_string(),
            state: ToolState::Error,
            results: 0,
            sources: Vec::new(),
            denied: None,
            sent: None,
        }
    }

    /// Refus faute d'autorisation de ce type : rien n'a été lu.
    fn denied(kind: ContentKind) -> Self {
        Self {
            state: ToolState::Denied,
            denied: Some(kind),
            ..Self::error(DENIED_TEXT)
        }
    }
}

/// Décide l'appel selon son type de contenu, puis l'exécute si on le doit.
pub(super) async fn execute(
    plan: &ChatPlan,
    guard: &Guard<'_>,
    progress: &mut Progress,
    call: &ToolCall,
) -> Result<Finished, Ending> {
    let kind = kind_of(&call.name);
    match decide_tool(plan.local, &plan.allowed_kinds, kind) {
        ToolDecision::Unknown => Ok(Finished::error("outil inconnu")),
        ToolDecision::Denied(kind) => Ok(Finished::denied(kind)),
        ToolDecision::Run(_) => run_known(plan, guard, progress, call).await,
    }
}

/// Exécute l'outil par la façade du moteur ; le résultat est masqué avant tout envoi.
async fn run_known(
    plan: &ChatPlan,
    guard: &Guard<'_>,
    progress: &mut Progress,
    call: &ToolCall,
) -> Result<Finished, Ending> {
    let Some(engine) = plan.host.as_ref().and_then(|host| host.upgrade()) else {
        return Ok(Finished::error("outils indisponibles : moteur fermé"));
    };
    let ran = guard.wait(run(&engine, call)).await.map_err(Ending::from)?;
    drop(engine);
    Ok(Finished {
        text: progress.answer.mask_tool_text(&ran.text),
        state: ran.state,
        results: ran.results,
        sent: (ran.state == ToolState::Ok).then_some(ran.kind),
        sources: ran.sources,
        denied: None,
    })
}

/// Borne le texte à la place restante de la conversation puis le cadre comme donnée.
pub(super) fn fit(plan: &ChatPlan, name: &str, text: &str) -> String {
    let used = request_chars(&plan.request) as usize;
    let room = MAX_INPUT_CHARS.saturating_sub(used);
    let kept = match room < MIN_ROOM_CHARS {
        true => "(résultat omis : conversation trop longue)".to_string(),
        false => bounded(text, room.min(RESULT_CHARS)),
    };
    frame_result(name, &kept)
}
