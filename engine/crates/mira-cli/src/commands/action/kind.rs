//! Nature d'une action en lot saisie en ligne de commande et sa traduction en `ActionKind`.

use clap::ValueEnum;
use mira_assembly::mira_vocabulary::{ActionKind, MailboxId, MiraError};

use crate::argument_error::invalid_argument;

/// Natures d'action applicables à des messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum KindName {
    /// Marque lu.
    MarkRead,
    /// Marque non lu.
    MarkUnread,
    /// Pose le drapeau de suivi.
    Flag,
    /// Retire le drapeau de suivi.
    Unflag,
    /// Déplace vers la boîte --to.
    Move,
    /// Ajoute à la boîte --to (libellé).
    AddToMailbox,
    /// Retire de la boîte --to (libellé).
    RemoveFromMailbox,
    /// Archive.
    Archive,
    /// Met à la corbeille.
    Trash,
    /// Supprime définitivement.
    DeletePermanently,
    /// Signale comme indésirable.
    Spam,
    /// Retire le signalement d'indésirable.
    NotSpam,
}

/// Dit si la nature d'action exige une boîte donnée par `--to`.
fn takes_mailbox(name: KindName) -> bool {
    matches!(
        name,
        KindName::Move | KindName::AddToMailbox | KindName::RemoveFromMailbox
    )
}

/// Boîte donnée par `--to`, obligatoire pour les natures qui en ont besoin.
fn destination(name: KindName, to: Option<&str>) -> Result<Option<MailboxId>, MiraError> {
    match (takes_mailbox(name), to) {
        (true, Some(mailbox)) => Ok(Some(MailboxId::from_raw(mailbox))),
        (true, None) => Err(invalid_argument("cette action exige --to <boîte>")),
        (false, Some(_)) => Err(invalid_argument("--to ne s'applique pas à cette action")),
        (false, None) => Ok(None),
    }
}

/// Traduit la nature saisie et la boîte éventuelle en action du contrat.
pub fn action_kind(name: KindName, to: Option<&str>) -> Result<ActionKind, MiraError> {
    let mailbox = destination(name, to)?;
    Ok(match (name, mailbox) {
        (KindName::MarkRead, _) => ActionKind::MarkRead { read: true },
        (KindName::MarkUnread, _) => ActionKind::MarkRead { read: false },
        (KindName::Flag, _) => ActionKind::MarkFlagged { flagged: true },
        (KindName::Unflag, _) => ActionKind::MarkFlagged { flagged: false },
        (KindName::Move, Some(to)) => ActionKind::Move { to },
        (KindName::AddToMailbox, Some(mailbox)) => ActionKind::AddToMailbox { mailbox },
        (KindName::RemoveFromMailbox, Some(mailbox)) => ActionKind::RemoveFromMailbox { mailbox },
        (KindName::Archive, _) => ActionKind::Archive,
        (KindName::Trash, _) => ActionKind::Trash,
        (KindName::DeletePermanently, _) => ActionKind::DeletePermanently,
        (KindName::Spam, _) => ActionKind::MarkSpam { spam: true },
        (KindName::NotSpam, _) => ActionKind::MarkSpam { spam: false },
        (KindName::Move | KindName::AddToMailbox | KindName::RemoveFromMailbox, None) => {
            return Err(invalid_argument("cette action exige --to <boîte>"));
        }
    })
}
