//! Exécution d'une action sur le serveur : routage exhaustif vers un handler par intention (9.9).

mod delete;
mod flags;
mod labels;
mod landed;
mod locate;
mod locate_require;
mod mailbox_admin;
mod moves;
mod purge;

use mira_vocabulary::{ActionKind, MiraError, PendingAction};

use super::context::ReplayContext;
use super::errors::mailbox_gone;
use super::job::{NamedMailbox, ReplayJob, load_job};
use crate::actions::model::LocalIntent;
use crate::imap::{ImapClient, ImapFlag};
use crate::send::{execute_send, save_draft_copy};

pub(crate) use purge::{effective, purge_selected};

/// Applique l'action au serveur ; une erreur dit si la tentative est passagère ou définitive
pub(super) async fn execute_action(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
) -> Result<(), MiraError> {
    match &action.kind {
        ActionKind::Send { .. } => execute_send(ctx, client, action).await,
        ActionKind::SaveDraft { draft } => save_draft_copy(ctx, client, draft).await,
        ActionKind::MarkRead { .. }
        | ActionKind::MarkFlagged { .. }
        | ActionKind::Move { .. }
        | ActionKind::AddToMailbox { .. }
        | ActionKind::RemoveFromMailbox { .. }
        | ActionKind::Archive
        | ActionKind::Trash
        | ActionKind::DeletePermanently
        | ActionKind::MarkSpam { .. }
        | ActionKind::CreateMailbox { .. }
        | ActionKind::RenameMailbox { .. }
        | ActionKind::DeleteMailbox { .. }
        | ActionKind::EmptyMailbox { .. } => {
            let job = load_job(ctx, action)?;
            execute_intent(ctx, client, &job).await
        }
    }
}

/// Route l'intention résolue vers l'opération serveur qui la réalise
async fn execute_intent(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    job: &ReplayJob,
) -> Result<(), MiraError> {
    match &job.intent {
        LocalIntent::SetSeen { seen } => flags::set_flag(client, job, ImapFlag::Seen, *seen).await,
        LocalIntent::SetFlagged { flagged } => {
            flags::set_flag(client, job, ImapFlag::Flagged, *flagged).await
        }
        LocalIntent::MoveTo { .. } => {
            moves::move_to(client, ctx.quirks, job, &named(job)?.name).await
        }
        LocalIntent::DropFrom { .. } => labels::remove_label(client, job, named(job)?).await,
        LocalIntent::AddTo { .. } => labels::add_label(client, job, named(job)?).await,
        LocalIntent::MarkDeleted => delete::delete_permanently(client, ctx.quirks, job).await,
        LocalIntent::CreateMailbox { mailbox } => {
            mailbox_admin::create(client, &mailbox.name).await
        }
        LocalIntent::RenameMailbox { name, .. } => {
            mailbox_admin::rename(client, &named(job)?.name, name).await
        }
        LocalIntent::DeleteMailbox { .. } => mailbox_admin::delete(client, &named(job)?.name).await,
        LocalIntent::EmptyMailbox { .. } => {
            mailbox_admin::empty(client, ctx.quirks, &named(job)?.name).await
        }
        LocalIntent::NoEffect => Ok(()),
    }
}

/// Boîte visée par l'intention ; absente du miroir comme des cibles = échec définitif
fn named(job: &ReplayJob) -> Result<&NamedMailbox, MiraError> {
    job.mailbox.as_ref().ok_or_else(mailbox_gone)
}
