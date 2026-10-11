//! Inverse et test « déjà en place » des étapes sur les drapeaux.

use mira_vocabulary::{Flags, MessageId, MiraError};

use crate::actions::model::LocalStep;
use crate::storage::Tx;

/// Étape qui rétablit la valeur actuelle de « lu »
pub(super) fn capture_seen(tx: &Tx<'_>, message: &MessageId) -> Result<Vec<LocalStep>, MiraError> {
    Ok(read_flags(tx, message)?
        .map(|flags| LocalStep::SetSeen {
            message: message.clone(),
            seen: flags.seen,
        })
        .into_iter()
        .collect())
}

/// Étape qui rétablit la valeur actuelle de « suivi »
pub(super) fn capture_flagged(
    tx: &Tx<'_>,
    message: &MessageId,
) -> Result<Vec<LocalStep>, MiraError> {
    Ok(read_flags(tx, message)?
        .map(|flags| LocalStep::SetFlagged {
            message: message.clone(),
            flagged: flags.flagged,
        })
        .into_iter()
        .collect())
}

/// Dit si « lu » a déjà la valeur voulue (un message absent compte comme en place)
pub(super) fn seen_in_effect(
    tx: &Tx<'_>,
    message: &MessageId,
    seen: bool,
) -> Result<bool, MiraError> {
    Ok(read_flags(tx, message)?.is_none_or(|flags| flags.seen == seen))
}

/// Dit si « suivi » a déjà la valeur voulue (un message absent compte comme en place)
pub(super) fn flagged_in_effect(
    tx: &Tx<'_>,
    message: &MessageId,
    flagged: bool,
) -> Result<bool, MiraError> {
    Ok(read_flags(tx, message)?.is_none_or(|flags| flags.flagged == flagged))
}

/// Lit les drapeaux du message, ou rien s'il est inconnu
fn read_flags(tx: &Tx<'_>, message: &MessageId) -> Result<Option<Flags>, MiraError> {
    Ok(tx.get_message(message)?.map(|summary| summary.flags))
}
