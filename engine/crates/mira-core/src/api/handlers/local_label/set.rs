//! Pose ou retrait d'un libellé local sur des conversations, tout ou rien, idempotente par clé.

use mira_vocabulary::{IdempotencyKey, LocalLabelId, MiraError, ThreadId};

use super::checks::MAX_THREADS_PER_CALL;
use super::idempotency::LabelCommand;
use super::writing::{announce, done};
use crate::actions::errors::input::invalid_parameter;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Pose (ou retire) le libellé sur ces conversations ; une erreur sur l'une refuse toute la pose
pub(crate) fn handle_set_local_label(
    engine: &Engine,
    label: &LocalLabelId,
    threads: &[ThreadId],
    on: bool,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    if threads.len() > MAX_THREADS_PER_CALL {
        return Err(invalid_parameter("trop de conversations en une fois"));
    }
    let command = LabelCommand::new(key, "set_local_label", &(label, threads, on))?;
    let written = engine.database.write(|tx| {
        if command.replay::<()>(tx)?.is_some() {
            return Ok(false);
        }
        for thread in threads {
            tx.set_label_on_thread(label, thread, on)?;
        }
        command.record(tx, &())?;
        Ok(true)
    })?;
    if written {
        announce(engine)?;
    }
    Ok(done())
}
