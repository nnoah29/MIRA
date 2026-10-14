//! `mira action apply <nature>` : action en lot sur une liste de messages ou sur une requête.

use clap::Args;
use mira_assembly::api::ActionRequest;
use mira_assembly::mira_vocabulary::MiraError;

use super::kind::{KindName, action_kind};
use super::selection::target_selection;
use crate::argument_context::ArgumentContext;
use crate::commands::key_args::KeyArgs;
use crate::commands::list_selection::ListSelectionArgs;
use crate::commands::waiting::WaitArgs;

/// Options de `mira action apply`.
#[derive(Debug, Args)]
pub struct ApplyArgs {
    /// Nature de l'action.
    #[arg(value_enum)]
    pub kind: KindName,
    /// Message visé (répétable) ; exclusif avec --mailbox et --view.
    #[arg(long = "message")]
    pub messages: Vec<String>,
    /// Boîte de destination (move, add-to-mailbox) ou à retirer (remove-from-mailbox).
    #[arg(long)]
    pub to: Option<String>,
    #[command(flatten)]
    pub selection: ListSelectionArgs,
    #[command(flatten)]
    pub key: KeyArgs,
    #[command(flatten)]
    pub wait: WaitArgs,
}

/// Construit la requête d'action en lot (une action par compte concerné).
pub fn request(args: &ApplyArgs, context: &ArgumentContext) -> Result<ActionRequest, MiraError> {
    Ok(ActionRequest::ApplyAction {
        kind: action_kind(args.kind, args.to.as_deref())?,
        targets: target_selection(&args.messages, &args.selection)?,
        idempotency_key: args.key.idempotency_key(context),
    })
}
