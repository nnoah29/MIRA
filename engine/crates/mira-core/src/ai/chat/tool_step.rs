//! Un tour d'outils : chaque appel est annoncé, décidé, exécuté, masqué, borné, cadré puis renvoyé au modèle.

use mira_vocabulary::{ChatDelta, ChatMessage, ChatRole, ToolCall, ToolOutcome, ToolState};

use super::ending::Ending;
use super::guard::Guard;
use super::prepare::ChatPlan;
use super::progress::Progress;
use super::tool_exec::{Finished, execute, fit};
use crate::ai::tools::summary_of;

const MAX_CALLS_PER_ROUND: usize = 8;

/// Répond à tous les appels du tour dans l'ordre et ajoute à l'historique le message de l'assistant puis les résultats.
pub(super) async fn run_tools(
    plan: &mut ChatPlan,
    guard: &Guard<'_>,
    progress: &mut Progress,
    calls: Vec<ToolCall>,
) -> Result<(), Ending> {
    let mut assistant = ChatMessage::plain(ChatRole::Assistant, progress.take_turn_text());
    assistant.tool_calls = calls.clone();
    plan.request.messages.push(assistant);
    for (position, call) in calls.iter().enumerate() {
        let permitted = position < MAX_CALLS_PER_ROUND;
        let outcome = answer_call(plan, guard, progress, call, permitted).await?;
        let mut message = ChatMessage::plain(ChatRole::Tool, String::new());
        message.tool_result = Some(outcome);
        plan.request.messages.push(message);
    }
    Ok(())
}

/// Annonce l'appel, l'exécute si permis, puis rend le résultat à renvoyer au modèle.
async fn answer_call(
    plan: &ChatPlan,
    guard: &Guard<'_>,
    progress: &mut Progress,
    call: &ToolCall,
    permitted: bool,
) -> Result<ToolOutcome, Ending> {
    let local = progress.answer.restored_call(call);
    let announced = ChatDelta::ToolCalled {
        id: call.id.clone(),
        name: call.name.chars().take(64).collect(),
        summary: summary_of(&local),
    };
    guard.send(announced).await.map_err(Ending::from)?;
    let finished = match permitted {
        true => execute(plan, guard, progress, &local).await?,
        false => Finished::error("limite d'appels par tour atteinte"),
    };
    finish_call(plan, guard, progress, call, finished).await
}

/// Cadre le résultat, le consigne au bilan des outils, annonce la fin de l'appel et rend le résultat du modèle.
async fn finish_call(
    plan: &ChatPlan,
    guard: &Guard<'_>,
    progress: &mut Progress,
    call: &ToolCall,
    finished: Finished,
) -> Result<ToolOutcome, Ending> {
    let text = fit(plan, &call.name, &finished.text);
    progress
        .used
        .record(&call.name, finished.sent, text.chars().count());
    let is_error = finished.state != ToolState::Ok;
    announce_end(guard, call, finished).await?;
    Ok(ToolOutcome {
        call_id: call.id.clone(),
        text,
        is_error,
    })
}

/// Annonce les propositions et cartes éventuelles, puis la fin de l'appel.
async fn announce_end(
    guard: &Guard<'_>,
    call: &ToolCall,
    mut finished: Finished,
) -> Result<(), Ending> {
    announce_extras(guard, call, &mut finished).await?;
    let end = ChatDelta::ToolFinished {
        id: call.id.clone(),
        state: finished.state,
        results: finished.results,
        sources: finished.sources,
        denied_kind: finished.denied,
    };
    guard.send(end).await.map_err(Ending::from)?;
    Ok(())
}

/// Annonce, dans l'ordre, brouillon, action, cartes de messages et proposition de mémoire.
async fn announce_extras(
    guard: &Guard<'_>,
    call: &ToolCall,
    finished: &mut Finished,
) -> Result<(), Ending> {
    let id = call.id.clone();
    if let Some(proposal) = finished.proposal.take() {
        let proposed = ChatDelta::DraftProposed {
            id: id.clone(),
            proposal,
        };
        guard.send(proposed).await.map_err(Ending::from)?;
    }
    if let Some(proposal) = finished.action.take() {
        let proposed = ChatDelta::ActionProposed {
            id: id.clone(),
            proposal,
        };
        guard.send(proposed).await.map_err(Ending::from)?;
    }
    announce_cards_and_memory(guard, id, finished).await
}

/// Annonce les cartes de messages puis la proposition de mémoire éventuelles.
async fn announce_cards_and_memory(
    guard: &Guard<'_>,
    id: String,
    finished: &mut Finished,
) -> Result<(), Ending> {
    if let Some(cards) = finished.cards.take() {
        let shown = ChatDelta::MailCardsShown {
            id: id.clone(),
            cards,
        };
        guard.send(shown).await.map_err(Ending::from)?;
    }
    if let Some(proposal) = finished.memory.take() {
        let proposed = ChatDelta::MemoryProposed { id, proposal };
        guard.send(proposed).await.map_err(Ending::from)?;
    }
    Ok(())
}
