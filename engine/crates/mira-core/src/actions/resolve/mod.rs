//! Vérifications préalables et résolution d'une action en intention locale (D-17, boîtes système, cibles du compte).

mod checks;
mod mailbox_admin;
mod messages;
mod naming;

use mira_vocabulary::{ActionKind, DraftId, MailboxRole, MiraError, ProviderQuirks};

use super::errors::not_found::draft_not_found;
use super::model::{LocalIntent, SubmitRequest};
use crate::ports::IdGenerator;
use crate::storage::Tx;
use checks::{require_no_targets, require_targets};

/// Refuse l'impossible puis dit quel effet local appliquer ; les messages ciblés doivent être du compte
pub(crate) fn resolve_intent(
    tx: &Tx<'_>,
    request: &SubmitRequest,
    quirks: &ProviderQuirks,
    ids: &dyn IdGenerator,
) -> Result<LocalIntent, MiraError> {
    match &request.kind {
        ActionKind::MarkRead { .. }
        | ActionKind::MarkFlagged { .. }
        | ActionKind::DeletePermanently
        | ActionKind::Move { .. }
        | ActionKind::AddToMailbox { .. }
        | ActionKind::RemoveFromMailbox { .. }
        | ActionKind::Archive
        | ActionKind::Trash
        | ActionKind::MarkSpam { .. } => {
            require_targets(tx, &request.account, &request.targets)?;
            resolve_message_action(tx, request, quirks)
        }
        ActionKind::CreateMailbox { .. }
        | ActionKind::RenameMailbox { .. }
        | ActionKind::DeleteMailbox { .. }
        | ActionKind::EmptyMailbox { .. } => {
            require_no_targets(&request.targets)?;
            resolve_mailbox_admin(tx, request, ids)
        }
        ActionKind::SaveDraft { draft } => require_draft(tx, request, draft),
        ActionKind::Send { .. } => Ok(LocalIntent::NoEffect),
    }
}

/// Résout les actions portant sur des messages (les cibles sont déjà vérifiées)
fn resolve_message_action(
    tx: &Tx<'_>,
    request: &SubmitRequest,
    quirks: &ProviderQuirks,
) -> Result<LocalIntent, MiraError> {
    let account = &request.account;
    match &request.kind {
        ActionKind::MarkRead { read } => Ok(LocalIntent::SetSeen { seen: *read }),
        ActionKind::MarkFlagged { flagged } => Ok(LocalIntent::SetFlagged { flagged: *flagged }),
        ActionKind::DeletePermanently => Ok(LocalIntent::MarkDeleted),
        ActionKind::Move { to } => messages::move_to(tx, account, to),
        ActionKind::AddToMailbox { mailbox } => messages::add_to(tx, account, mailbox, quirks),
        ActionKind::RemoveFromMailbox { mailbox } => {
            messages::remove_from(tx, account, mailbox, quirks)
        }
        ActionKind::Archive => messages::archive(tx, account, quirks),
        ActionKind::Trash => messages::move_to_role(tx, account, MailboxRole::Trash),
        ActionKind::MarkSpam { spam } => messages::move_to_role(tx, account, spam_role(*spam)),
        ActionKind::CreateMailbox { .. }
        | ActionKind::RenameMailbox { .. }
        | ActionKind::DeleteMailbox { .. }
        | ActionKind::EmptyMailbox { .. }
        | ActionKind::SaveDraft { .. }
        | ActionKind::Send { .. } => Ok(LocalIntent::NoEffect),
    }
}

/// Boîte de destination : le courrier indésirable pour « spam », la réception pour « non spam »
fn spam_role(spam: bool) -> MailboxRole {
    if spam {
        MailboxRole::Junk
    } else {
        MailboxRole::Inbox
    }
}

/// Résout les actions qui créent, renomment, suppriment ou vident une boîte
fn resolve_mailbox_admin(
    tx: &Tx<'_>,
    request: &SubmitRequest,
    ids: &dyn IdGenerator,
) -> Result<LocalIntent, MiraError> {
    let account = &request.account;
    match &request.kind {
        ActionKind::CreateMailbox { name, parent } => {
            mailbox_admin::create(tx, account, name, parent.as_ref(), ids)
        }
        ActionKind::RenameMailbox { mailbox, name } => {
            mailbox_admin::rename(tx, account, mailbox, name)
        }
        ActionKind::DeleteMailbox { mailbox } => mailbox_admin::delete(tx, account, mailbox),
        ActionKind::EmptyMailbox { mailbox } => mailbox_admin::empty(tx, account, mailbox),
        ActionKind::MarkRead { .. }
        | ActionKind::MarkFlagged { .. }
        | ActionKind::Move { .. }
        | ActionKind::AddToMailbox { .. }
        | ActionKind::RemoveFromMailbox { .. }
        | ActionKind::Archive
        | ActionKind::Trash
        | ActionKind::DeletePermanently
        | ActionKind::MarkSpam { .. }
        | ActionKind::SaveDraft { .. }
        | ActionKind::Send { .. } => Ok(LocalIntent::NoEffect),
    }
}

/// Exige un brouillon du compte pour enregistrer sa copie sur le serveur
fn require_draft(
    tx: &Tx<'_>,
    request: &SubmitRequest,
    draft: &DraftId,
) -> Result<LocalIntent, MiraError> {
    match tx.get_draft(draft)? {
        Some(found) if found.account == request.account => Ok(LocalIntent::NoEffect),
        _ => Err(draft_not_found(draft)),
    }
}
