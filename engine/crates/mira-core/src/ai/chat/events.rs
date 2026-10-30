//! Événements du journal pour une génération : fournisseur, modèle, jetons, durée, issue ; aucun contenu.

use mira_vocabulary::EventKind;

use super::ending::Ending;
use super::prepare::ChatPlan;
use super::progress::Progress;
use crate::events::{EventBus, EventSink};
use crate::operations::OperationHandle;

/// Annonce le début de la génération.
pub(super) fn emit_started(events: &EventBus, handle: &OperationHandle, plan: &ChatPlan) {
    let _ = events.emit(EventKind::AiChatStarted {
        operation: handle.id().clone(),
        provider: plan.descriptor.id.0.clone(),
        model: plan.model.clone(),
    });
}

/// Annonce la fin de la génération avec sa consommation, sa durée et son issue.
pub(super) fn emit_finished(
    events: &EventBus,
    handle: &OperationHandle,
    progress: &Progress,
    ending: &Ending,
    duration_ms: u64,
) {
    let (input_tokens, output_tokens) = progress.tokens();
    let _ = events.emit(EventKind::AiChatFinished {
        operation: handle.id().clone(),
        input_tokens,
        output_tokens,
        duration_ms,
        outcome: ending.outcome(),
    });
}
