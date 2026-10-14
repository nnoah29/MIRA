//! `mira draft send` : programme l'envoi d'un brouillon, annulable pendant le délai.

use clap::Args;
use mira_assembly::api::ComposeRequest;
use mira_assembly::mira_vocabulary::{DraftId, MiraError};

use crate::argument_context::ArgumentContext;
use crate::commands::instant_arg::parse_instant;
use crate::commands::key_args::KeyArgs;
use crate::commands::waiting::WaitArgs;

/// Options de `mira draft send`.
#[derive(Debug, Args)]
pub struct SendArgs {
    /// Identifiant du brouillon.
    pub draft: String,
    /// Instant d'envoi (millisecondes Unix, AAAA-MM-JJ ou AAAA-MM-JJTHH:MM, UTC) ; sinon après le délai d'annulation.
    #[arg(long)]
    pub at: Option<String>,
    #[command(flatten)]
    pub key: KeyArgs,
    #[command(flatten)]
    pub wait: WaitArgs,
}

/// Construit la requête d'envoi d'un brouillon.
pub fn request(args: &SendArgs, context: &ArgumentContext) -> Result<ComposeRequest, MiraError> {
    Ok(ComposeRequest::SendDraft {
        draft: DraftId::from_raw(args.draft.as_str()),
        at: args.at.as_deref().map(parse_instant).transpose()?,
        idempotency_key: args.key.idempotency_key(context),
    })
}
