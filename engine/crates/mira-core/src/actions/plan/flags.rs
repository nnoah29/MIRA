//! Étapes des actions sur les drapeaux d'un message.

use mira_vocabulary::MessageId;

use crate::actions::model::LocalStep;

/// Fixe « lu » sur chaque message
pub(super) fn set_seen(targets: &[MessageId], seen: bool) -> Vec<LocalStep> {
    targets
        .iter()
        .map(|message| LocalStep::SetSeen {
            message: message.clone(),
            seen,
        })
        .collect()
}

/// Fixe « suivi » sur chaque message
pub(super) fn set_flagged(targets: &[MessageId], flagged: bool) -> Vec<LocalStep> {
    targets
        .iter()
        .map(|message| LocalStep::SetFlagged {
            message: message.clone(),
            flagged,
        })
        .collect()
}
