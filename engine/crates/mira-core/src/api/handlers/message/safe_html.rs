//! HTML assaini d'un message, avec les ressources distantes bloquées (11.6, décision 10).

use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{ErrorCategory, MessageId, MiraError};

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::html::{SafeHtml, sanitize_html};
use crate::reading::body_unavailable;
use crate::storage::Tx;
use crate::storage::repositories::allowances::RemoteContentScope;
use crate::storage::repositories::messages::message_not_found;

/// Dit si le contenu distant est autorisé pour ce message ou pour son expéditeur
fn remote_content_allowed(
    tx: &Tx<'_>,
    id: &MessageId,
    sender: Option<&str>,
) -> Result<bool, MiraError> {
    if tx.is_remote_content_allowed(RemoteContentScope::Message, id.as_str())? {
        return Ok(true);
    }
    match sender {
        Some(address) => tx.is_remote_content_allowed(RemoteContentScope::Sender, address),
        None => Ok(false),
    }
}

/// Lit l'HTML du message et l'autorisation de contenu distant, sans assainir
fn load_html_and_consent(
    engine: &Engine,
    message: &MessageId,
) -> Result<(Option<String>, bool), MiraError> {
    engine.database.read(|tx| {
        let summary = tx
            .get_message(message)?
            .ok_or_else(|| message_not_found(message))?;
        let parsed = tx
            .load_parsed(message)?
            .ok_or_else(|| body_unavailable(message))?;
        let sender = summary.from.first().map(|entry| entry.address.as_str());
        let allowed = remote_content_allowed(tx, message, sender)?;
        Ok((parsed.html, allowed))
    })
}

/// Assainit la version HTML du message hors verrou de base ; sans HTML, rend un HTML vide
pub(crate) async fn handle_get_safe_html(
    engine: &Engine,
    message: &MessageId,
) -> Result<Response, MiraError> {
    let (html, allowed) = load_html_and_consent(engine, message)?;
    let Some(html) = html else {
        return Ok(Response::SafeHtml(SafeHtml {
            html: String::new(),
            blocked_resources: Vec::new(),
        }));
    };
    let safe = tokio::task::spawn_blocking(move || sanitize_html(&html, allowed))
        .await
        .map_err(|_| {
            MiraError::new(
                ErrorCategory::Internal,
                internal::BUG,
                "assainissement interrompu",
            )
        })?;
    Ok(Response::SafeHtml(safe))
}
