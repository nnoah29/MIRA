//! `mira draft attach` et `mira draft detach` : pièces jointes d'un brouillon.

use clap::Args;
use mira_assembly::api::ComposeRequest;

use crate::commands::targets::DraftArg;

/// Options de `mira draft attach`.
#[derive(Debug, Args)]
pub struct AttachArgs {
    #[command(flatten)]
    pub draft: DraftArg,
    /// Fichier à joindre.
    pub path: String,
}

/// Options de `mira draft detach`.
#[derive(Debug, Args)]
pub struct DetachArgs {
    #[command(flatten)]
    pub draft: DraftArg,
    /// Position de la pièce jointe dans le brouillon (à partir de 0).
    pub position: u32,
}

/// Requête d'ajout d'une pièce jointe.
pub fn attach_request(args: &AttachArgs) -> ComposeRequest {
    ComposeRequest::AttachFile {
        draft: args.draft.id(),
        path: args.path.clone(),
    }
}

/// Requête de retrait d'une pièce jointe.
pub fn detach_request(args: &DetachArgs) -> ComposeRequest {
    ComposeRequest::RemoveAttachment {
        draft: args.draft.id(),
        position: args.position,
    }
}
