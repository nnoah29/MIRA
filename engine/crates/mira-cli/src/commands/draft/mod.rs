//! Commandes `mira draft …` : rédaction, pièces jointes, envoi différé et boîte d'envoi.

pub mod attachments;
pub mod create;
pub mod recipients;
pub mod resolve_send;
pub mod send;
pub mod update;

use clap::Subcommand;
use mira_assembly::api::{ComposeRequest, Request};
use mira_assembly::mira_vocabulary::MiraError;

use super::targets::{AccountArg, ActionArg, DraftArg};
use super::waiting::WaitPolicy;
use crate::argument_context::ArgumentContext;

/// Commandes du domaine `draft`.
#[derive(Debug, Subcommand)]
pub enum DraftCommand {
    /// Crée un brouillon neuf, une réponse ou un transfert.
    Create(create::CreateArgs),
    /// Modifie les champs d'un brouillon en édition.
    Update(update::UpdateArgs),
    /// Joint un fichier à un brouillon.
    Attach(attachments::AttachArgs),
    /// Retire une pièce jointe d'un brouillon.
    Detach(attachments::DetachArgs),
    /// Supprime un brouillon.
    Delete(DraftArg),
    /// Affiche un brouillon.
    Get(DraftArg),
    /// Liste les brouillons d'un compte.
    List(AccountArg),
    /// Programme l'envoi d'un brouillon (annulable pendant le délai).
    Send(send::SendArgs),
    /// Annule un envoi encore en attente.
    CancelSend(ActionArg),
    /// Liste la boîte d'envoi d'un compte.
    Outbox(AccountArg),
    /// Tranche un envoi incertain : renvoyer ou le tenir pour parti.
    ResolveSend(resolve_send::ResolveSendArgs),
}

/// Traduit une commande `draft` en requête du contrat.
pub fn build_request(
    command: &DraftCommand,
    context: &ArgumentContext,
) -> Result<Request, MiraError> {
    let request = match command {
        DraftCommand::Create(args) => create::request(args),
        DraftCommand::Update(args) => update::request(args)?,
        DraftCommand::Attach(args) => attachments::attach_request(args),
        DraftCommand::Detach(args) => attachments::detach_request(args),
        DraftCommand::Delete(target) => ComposeRequest::DeleteDraft { draft: target.id() },
        DraftCommand::Get(target) => ComposeRequest::GetDraft { draft: target.id() },
        DraftCommand::List(target) => ComposeRequest::ListDrafts {
            account: target.id(),
        },
        DraftCommand::Send(args) => send::request(args, context)?,
        DraftCommand::CancelSend(target) => ComposeRequest::CancelSend {
            action: target.id(),
        },
        DraftCommand::Outbox(target) => ComposeRequest::ListOutbox {
            account: target.id(),
        },
        DraftCommand::ResolveSend(args) => resolve_send::request(args),
    };
    Ok(Request::Compose(request))
}

/// Attente demandée par une commande `draft` : seuls l'envoi et sa résolution peuvent attendre.
pub fn wait_policy(command: &DraftCommand) -> WaitPolicy {
    match command {
        DraftCommand::Send(args) => args.wait.policy(),
        DraftCommand::ResolveSend(args) => args.wait.policy(),
        DraftCommand::Create(_)
        | DraftCommand::Update(_)
        | DraftCommand::Attach(_)
        | DraftCommand::Detach(_)
        | DraftCommand::Delete(_)
        | DraftCommand::Get(_)
        | DraftCommand::List(_)
        | DraftCommand::CancelSend(_)
        | DraftCommand::Outbox(_) => WaitPolicy::Immediate,
    }
}
