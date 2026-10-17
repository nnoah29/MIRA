//! `mira message save-attachment` : enregistre une pièce jointe dans un fichier.

use clap::Args;
use mira_assembly::api::{AttachmentTarget, MessageRequest};
use mira_assembly::mira_vocabulary::AttachmentId;

use crate::commands::waiting::OperationWaitArgs;

/// Options de `mira message save-attachment`.
#[derive(Debug, Args)]
pub struct SaveAttachmentArgs {
    /// Identifiant de la pièce jointe.
    pub attachment: String,
    /// Fichier de destination (écrit de façon atomique).
    pub destination: String,
    #[command(flatten)]
    pub wait: OperationWaitArgs,
}

/// Construit la requête d'enregistrement d'une pièce jointe.
pub fn request(args: &SaveAttachmentArgs) -> MessageRequest {
    MessageRequest::SaveAttachment(AttachmentTarget {
        attachment: AttachmentId::from_raw(args.attachment.as_str()),
        destination: args.destination.clone(),
    })
}
