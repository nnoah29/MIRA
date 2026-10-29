//! Tâche de fond d'une génération : diffusion, fin unique, ligne de journal complétée, clôture de l'opération.

use std::sync::Arc;

use mira_vocabulary::{ChatDelta, Timestamp};
use tokio::sync::{mpsc, oneshot};

use super::answer::Answer;
use super::ending::Ending;
use super::events::{emit_finished, emit_started};
use super::guard::Guard;
use super::journal::{close_row, finish_of};
use super::prepare::ChatPlan;
use super::progress::Progress;
use super::stream::converse;
use crate::events::EventBus;
use crate::operations::{OperationHandle, OperationRegistry};
use crate::ports::Clock;
use crate::storage::Database;

/// Ce que la tâche emporte : plan, opération, événements, base, horloge et canaux vers le lecteur.
pub(crate) struct ChatTask {
    pub(crate) plan: ChatPlan,
    pub(crate) handle: OperationHandle,
    pub(crate) operations: OperationRegistry,
    pub(crate) events: EventBus,
    pub(crate) database: Arc<Database>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) deltas: mpsc::Sender<ChatDelta>,
    pub(crate) done: oneshot::Sender<ChatDelta>,
}

/// Mène la génération ; le `Done` final n'est visible du lecteur qu'une fois l'opération close.
pub(crate) async fn run(task: ChatTask) {
    let ChatTask {
        mut plan,
        handle,
        operations,
        events,
        database,
        clock,
        deltas,
        done,
    } = task;
    let started = clock.now();
    emit_started(&events, &handle, &plan);
    let mut progress = Progress::new(Answer::new(std::mem::take(&mut plan.masker)));
    let guard = Guard::new(&handle, &deltas);
    let ending = converse(&plan, &guard, &mut progress).await;
    let ending = settle(deliver_usage(ending, &guard, &progress).await, &handle);
    close_row(
        &database,
        handle.id(),
        &finish_of(&progress, &ending),
        clock.now(),
    );
    if let Some(delta) = ending.final_delta() {
        let _ = done.send(delta);
    }
    ending.close(&handle, &operations);
    let elapsed = elapsed_ms(started, clock.now());
    emit_finished(&events, &handle, &progress, &ending, elapsed);
}

/// Remet la consommation agrégée avant la fin d'une génération aboutie ou échouée.
async fn deliver_usage(ending: Ending, guard: &Guard<'_>, progress: &Progress) -> Ending {
    let (Ending::Finished(_) | Ending::Failed(_), Some(usage)) = (&ending, progress.usage_delta())
    else {
        return ending;
    };
    match guard.send(usage).await {
        Ok(()) => ending,
        Err(interrupt) => interrupt.into(),
    }
}

/// Une annulation arrivée pendant la fin l'emporte sur une fin normale, pour un statut cohérent.
fn settle(ending: Ending, handle: &OperationHandle) -> Ending {
    match ending {
        Ending::Finished(_) if handle.is_cancelled() => Ending::Cancelled,
        other => other,
    }
}

/// Durée écoulée entre deux instants de l'horloge, nulle si elle recule.
fn elapsed_ms(started: Timestamp, ended: Timestamp) -> u64 {
    u64::try_from(ended.as_millis().saturating_sub(started.as_millis())).unwrap_or(0)
}
