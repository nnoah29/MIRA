//! `mira draft create` : brouillon neuf, réponse, réponse à tous ou transfert.

use clap::Args;
use mira_assembly::api::{ComposeRequest, DraftMode};
use mira_assembly::mira_vocabulary::{AccountId, IdentityId, MessageId};

/// Options de `mira draft create` ; au plus un des modes réponse, réponse à tous, transfert.
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Identifiant du compte qui enverra le message.
    pub account: String,
    /// Identité d'envoi (par défaut celle du compte).
    #[arg(long)]
    pub identity: Option<String>,
    /// Répond à ce message.
    #[arg(long, conflicts_with_all = ["reply_all", "forward"])]
    pub reply: Option<String>,
    /// Répond à l'expéditeur et aux destinataires de ce message.
    #[arg(long, conflicts_with = "forward")]
    pub reply_all: Option<String>,
    /// Transfère ce message.
    #[arg(long)]
    pub forward: Option<String>,
}

/// Mode de départ du brouillon d'après les options données.
fn draft_mode(args: &CreateArgs) -> DraftMode {
    let message = |id: &String| MessageId::from_raw(id.as_str());
    match (&args.reply, &args.reply_all, &args.forward) {
        (Some(id), _, _) => DraftMode::Reply {
            message: message(id),
        },
        (None, Some(id), _) => DraftMode::ReplyAll {
            message: message(id),
        },
        (None, None, Some(id)) => DraftMode::Forward {
            message: message(id),
        },
        (None, None, None) => DraftMode::New,
    }
}

/// Construit la requête de création de brouillon.
pub fn request(args: &CreateArgs) -> ComposeRequest {
    ComposeRequest::CreateDraft {
        account: AccountId::from_raw(args.account.as_str()),
        identity: args.identity.as_deref().map(IdentityId::from_raw),
        mode: draft_mode(args),
    }
}
