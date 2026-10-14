//! Autorisation du contenu distant pour un message ou un expéditeur, mémorisée dans le profil.

use mira_vocabulary::{EmailAddress, MiraError};

use crate::api::contract::{RemoteContentTarget, Response};
use crate::api::engine::Engine;
use crate::storage::repositories::allowances::RemoteContentScope;
use crate::storage::repositories::messages::message_not_found;

/// Autorise le contenu distant ; un message inconnu ou une adresse invalide est refusé
pub(crate) fn handle_allow_remote_content(
    engine: &Engine,
    target: &RemoteContentTarget,
) -> Result<Response, MiraError> {
    engine.database.write(|tx| match target {
        RemoteContentTarget::Message { message } => {
            tx.get_message(message)?
                .ok_or_else(|| message_not_found(message))?;
            tx.add_remote_content_allowance(RemoteContentScope::Message, message.as_str())
        }
        RemoteContentTarget::Sender { address } => {
            let parsed = EmailAddress::parse(address)?;
            tx.add_remote_content_allowance(RemoteContentScope::Sender, parsed.as_str())
        }
    })?;
    Ok(Response::Done)
}
