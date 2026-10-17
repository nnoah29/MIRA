//! Commandes `mira message …` : listes, lecture, corps, source, pièces jointes, contenu distant.

pub mod allow_remote;
pub mod list;
pub mod save_attachment;

use clap::{Args, Subcommand};
use mira_assembly::api::{MessageRequest, Request};
use mira_assembly::mira_vocabulary::{MessageId, MiraError};

use super::targets::MessageArg;
use super::waiting::{OperationWaitArgs, WaitPolicy};

/// Commandes du domaine `message`.
#[derive(Debug, Subcommand)]
pub enum MessageCommand {
    /// Liste les messages d'une boîte ou d'une vue standard (lecture locale).
    List(list::ListArgs),
    /// Affiche les en-têtes et l'état d'un message.
    Get(MessageArg),
    /// Affiche le corps d'un message déjà téléchargé.
    Body(MessageArg),
    /// Télécharge le corps d'un message depuis le serveur (opération réseau explicite).
    Fetch(FetchArgs),
    /// Affiche la source brute d'un message déjà téléchargé.
    Raw(MessageArg),
    /// Liste les pièces jointes d'un message.
    Attachments(MessageArg),
    /// Enregistre une pièce jointe dans un fichier (opération réseau explicite).
    SaveAttachment(save_attachment::SaveAttachmentArgs),
    /// Affiche le HTML assaini d'un message.
    SafeHtml(MessageArg),
    /// Autorise le contenu distant d'un message ou d'un expéditeur.
    AllowRemote(allow_remote::AllowRemoteArgs),
}

/// Options de `mira message fetch`.
#[derive(Debug, Args)]
pub struct FetchArgs {
    /// Identifiant du message.
    pub message: String,
    #[command(flatten)]
    pub wait: OperationWaitArgs,
}

/// Traduit une commande `message` en requête du contrat.
pub fn build_request(command: &MessageCommand) -> Result<Request, MiraError> {
    let request = match command {
        MessageCommand::List(args) => list::request(args)?,
        MessageCommand::Get(target) => MessageRequest::GetMessage {
            message: target.id(),
        },
        MessageCommand::Body(target) => MessageRequest::GetMessageBody {
            message: target.id(),
        },
        MessageCommand::Fetch(args) => MessageRequest::FetchMessageBody {
            message: MessageId::from_raw(args.message.as_str()),
        },
        MessageCommand::Raw(target) => MessageRequest::GetRawSource {
            message: target.id(),
        },
        MessageCommand::Attachments(target) => MessageRequest::ListAttachments {
            message: target.id(),
        },
        MessageCommand::SaveAttachment(args) => save_attachment::request(args),
        MessageCommand::SafeHtml(target) => MessageRequest::GetSafeHtml {
            message: target.id(),
        },
        MessageCommand::AllowRemote(args) => allow_remote::request(args)?,
    };
    Ok(Request::Message(request))
}

/// Attente demandée par une commande `message` : les opérations longues sont toujours attendues.
pub fn wait_policy(command: &MessageCommand) -> WaitPolicy {
    match command {
        MessageCommand::Fetch(args) => args.wait.policy(),
        MessageCommand::SaveAttachment(args) => args.wait.policy(),
        MessageCommand::List(_)
        | MessageCommand::Get(_)
        | MessageCommand::Body(_)
        | MessageCommand::Raw(_)
        | MessageCommand::Attachments(_)
        | MessageCommand::SafeHtml(_)
        | MessageCommand::AllowRemote(_) => WaitPolicy::Immediate,
    }
}
