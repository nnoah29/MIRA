//! `mira draft resolve-send` : tranche un envoi dont le résultat est incertain.

use clap::Args;
use mira_assembly::api::ComposeRequest;

use crate::commands::targets::ActionArg;
use crate::commands::waiting::WaitArgs;

/// Options de `mira draft resolve-send`.
#[derive(Debug, Args)]
pub struct ResolveSendArgs {
    #[command(flatten)]
    pub action: ActionArg,
    /// Renvoie le message ; sans cette option, l'envoi est tenu pour parti.
    #[arg(long)]
    pub resend: bool,
    #[command(flatten)]
    pub wait: WaitArgs,
}

/// Construit la requête de résolution d'un envoi incertain.
pub fn request(args: &ResolveSendArgs) -> ComposeRequest {
    ComposeRequest::ResolveUncertainSend {
        action: args.action.id(),
        resend: args.resend,
    }
}
