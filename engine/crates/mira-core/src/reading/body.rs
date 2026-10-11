//! Corps et source brute d'un message, lus seulement s'ils sont déjà présents en local (S6).

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{ContentState, ErrorCategory, ErrorContext, MessageId, MiraError};
use serde::{Deserialize, Serialize};

use crate::storage::Database;
use crate::storage::repositories::messages::message_not_found;

/// Corps disponible d'un message : le texte et la présence d'une version HTML.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyView {
    pub text: String,
    pub html_available: bool,
}

/// Rend le corps du message s'il est présent localement, sinon `incompatible_state.body_unavailable_offline`
pub fn get_body(db: &Database, id: &MessageId) -> Result<BodyView, MiraError> {
    db.read(|tx| {
        let message = tx.get_message(id)?.ok_or_else(|| message_not_found(id))?;
        let parsed = match message.content {
            ContentState::BodyPresent => tx.load_parsed(id)?,
            ContentState::HeadersOnly => None,
        };
        parsed
            .map(|content| BodyView {
                text: content.text,
                html_available: content.html.is_some(),
            })
            .ok_or_else(|| body_unavailable(id))
    })
}

/// Rend la source brute du message si elle est stockée localement, sinon `incompatible_state.body_unavailable_offline`
pub fn get_raw_source(db: &Database, id: &MessageId) -> Result<Vec<u8>, MiraError> {
    db.read(|tx| {
        tx.get_message(id)?.ok_or_else(|| message_not_found(id))?;
        tx.load_raw_source(id)?.ok_or_else(|| body_unavailable(id))
    })
}

/// Erreur « corps absent hors ligne » : le téléchargement est une commande réseau explicite
fn body_unavailable(id: &MessageId) -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::BODY_UNAVAILABLE_OFFLINE,
        "le corps du message n'est pas disponible hors ligne",
    )
    .with_context(ErrorContext {
        message: Some(id.clone()),
        ..ErrorContext::default()
    })
}
