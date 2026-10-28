//! Choix manuel de l'utilisateur : poser ou retirer un libellé système, valable jusqu'au prochain message du fil.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MiraError, SystemLabelKind, ThreadId};

use crate::storage::{SystemState, Tx};

/// Pose (`on`) ou retire un libellé système sur des conversations ; une conversation inconnue refuse tout
pub fn set_system_label(
    tx: &Tx<'_>,
    kind: SystemLabelKind,
    threads: &[ThreadId],
    on: bool,
) -> Result<(), MiraError> {
    for thread in threads {
        let (account, key) = tx.thread_root_key(thread)?.ok_or_else(unknown_thread)?;
        let as_of = newest_message_date(tx, thread)?;
        let state = SystemState {
            on,
            manual: true,
            as_of,
        };
        tx.put_system_label_state(&account, &key, kind, state)?;
    }
    Ok(())
}

/// Date du message le plus récent du fil, 0 s'il n'en a pas
fn newest_message_date(tx: &Tx<'_>, thread: &ThreadId) -> Result<i64, MiraError> {
    let dates = tx
        .messages_of_thread(thread)?
        .into_iter()
        .map(|m| m.date.or(m.internal_date).map_or(0, |t| t.millis));
    Ok(dates.max().unwrap_or(0))
}

/// Erreur « fil inconnu »
fn unknown_thread() -> MiraError {
    MiraError::new(ErrorCategory::NotFound, not_found::THREAD, "fil inconnu")
}
