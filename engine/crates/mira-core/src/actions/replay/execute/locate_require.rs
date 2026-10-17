//! Garde-fou commun des actions sur des messages : au moins un message doit exister sur le serveur.

use std::collections::BTreeSet;

use mira_vocabulary::{MessageId, MiraError};

use crate::actions::replay::errors::message_gone;
use crate::actions::replay::job::ServerMessage;

/// Échec définitif (message disparu) si aucun des messages visés n'a été retrouvé (S8)
pub(super) fn require_any_handled(
    messages: &[ServerMessage],
    handled: &BTreeSet<MessageId>,
) -> Result<(), MiraError> {
    if !messages.is_empty() && handled.is_empty() {
        return Err(message_gone());
    }
    Ok(())
}
