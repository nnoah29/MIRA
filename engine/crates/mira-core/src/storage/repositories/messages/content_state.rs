//! Nom stocké de l'état du contenu d'un message.

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ContentState, ErrorCategory, MiraError};

/// Donne le nom stocké d'un état de contenu
pub(super) fn content_state_name(state: ContentState) -> &'static str {
    match state {
        ContentState::HeadersOnly => "headers_only",
        ContentState::BodyPresent => "body_present",
    }
}

/// Relit un état de contenu depuis son nom stocké
pub(super) fn content_state_from_name(name: &str) -> Result<ContentState, MiraError> {
    match name {
        "headers_only" => Ok(ContentState::HeadersOnly),
        "body_present" => Ok(ContentState::BodyPresent),
        _ => Err(MiraError::new(
            ErrorCategory::LocalStorage,
            local_storage::CORRUPTED,
            "état de contenu inconnu en base",
        )),
    }
}
