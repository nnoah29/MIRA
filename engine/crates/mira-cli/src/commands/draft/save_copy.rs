//! `mira draft save-copy` : dépose une copie du brouillon dans la boîte Brouillons du serveur.

use clap::Args;
use mira_assembly::api::ComposeRequest;
use mira_assembly::mira_vocabulary::DraftId;

use crate::argument_context::ArgumentContext;
use crate::commands::key_args::KeyArgs;

/// Options de `mira draft save-copy`.
#[derive(Debug, Args)]
pub struct SaveCopyArgs {
    /// Identifiant du brouillon.
    pub draft: String,
    #[command(flatten)]
    pub key: KeyArgs,
}

/// Construit la requête de copie serveur d'un brouillon.
pub fn request(args: &SaveCopyArgs, context: &ArgumentContext) -> ComposeRequest {
    ComposeRequest::SaveDraftCopy {
        draft: DraftId::from_raw(args.draft.as_str()),
        idempotency_key: args.key.idempotency_key(context),
    }
}
