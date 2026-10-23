//! Application des changements de libellés locaux décidés par les règles, sur la conversation du message.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{MiraError, ThreadId};

use crate::rules::outcome::LabelChange;
use crate::storage::Tx;

/// Pose ou retire les libellés sur la conversation (un libellé supprimé entre-temps est ignoré) ; vrai si elle a changé
pub(crate) fn apply_label_changes(
    tx: &Tx<'_>,
    thread: Option<&ThreadId>,
    changes: &[LabelChange],
) -> Result<bool, MiraError> {
    let Some(thread) = thread else {
        return Ok(false);
    };
    let before = tx.labels_of_thread(thread)?;
    for change in changes {
        match tx.set_label_on_thread(&change.label, thread, change.on) {
            Err(error) if error.code == not_found::LOCAL_LABEL.0 => {}
            other => other?,
        }
    }
    Ok(before != tx.labels_of_thread(thread)?)
}
