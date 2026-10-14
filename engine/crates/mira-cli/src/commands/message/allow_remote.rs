//! `mira message allow-remote` : autorise le contenu distant d'un message ou d'un expéditeur.

use clap::Args;
use mira_assembly::api::{MessageRequest, RemoteContentTarget};
use mira_assembly::mira_vocabulary::{MessageId, MiraError};

use crate::argument_error::invalid_argument;

/// Options de `mira message allow-remote` : exactement un message ou un expéditeur.
#[derive(Debug, Args)]
pub struct AllowRemoteArgs {
    /// Identifiant du message dont le contenu distant est autorisé.
    #[arg(long, conflicts_with = "sender")]
    pub message: Option<String>,
    /// Adresse de l'expéditeur dont le contenu distant est autorisé.
    #[arg(long)]
    pub sender: Option<String>,
}

/// Construit la requête d'autorisation ; il faut désigner un message ou un expéditeur.
pub fn request(args: &AllowRemoteArgs) -> Result<MessageRequest, MiraError> {
    let scope = match (&args.message, &args.sender) {
        (Some(message), _) => RemoteContentTarget::Message {
            message: MessageId::from_raw(message.as_str()),
        },
        (None, Some(address)) => RemoteContentTarget::Sender {
            address: address.clone(),
        },
        (None, None) => return Err(invalid_argument("indiquer --message ou --sender")),
    };
    Ok(MessageRequest::AllowRemoteContent { scope })
}
