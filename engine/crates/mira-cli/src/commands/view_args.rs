//! Choix de la vue à lister : une boîte (`--mailbox`) ou une vue standard (`--view`).

use clap::{Args, ValueEnum};
use mira_assembly::mira_vocabulary::{MailboxId, MiraError, StandardView, ViewRef};

use crate::argument_error::invalid_argument;

/// Vues standard construites sur tous les comptes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StandardViewName {
    /// Tous les messages de tous les comptes.
    Unified,
    /// Messages non lus.
    Unread,
    /// Messages suivis (drapeau).
    Flagged,
}

/// Vue visée : exactement l'une des deux options.
#[derive(Debug, Args)]
pub struct ViewArgs {
    /// Identifiant de la boîte à lister.
    #[arg(long, conflicts_with = "view")]
    pub mailbox: Option<String>,
    /// Vue standard sur tous les comptes.
    #[arg(long, value_enum)]
    pub view: Option<StandardViewName>,
}

impl ViewArgs {
    /// Dit si l'une des deux options a été donnée.
    pub fn is_given(&self) -> bool {
        self.mailbox.is_some() || self.view.is_some()
    }

    /// Traduit les options en référence de vue ; l'une des deux est obligatoire.
    pub fn view_ref(&self) -> Result<ViewRef, MiraError> {
        match (&self.mailbox, self.view) {
            (Some(mailbox), _) => Ok(ViewRef::Mailbox {
                mailbox: MailboxId::from_raw(mailbox.as_str()),
            }),
            (None, Some(name)) => Ok(ViewRef::Standard {
                view: standard_view(name),
            }),
            (None, None) => Err(invalid_argument("indiquer --mailbox ou --view")),
        }
    }
}

/// Vue standard du contrat correspondant au nom saisi.
fn standard_view(name: StandardViewName) -> StandardView {
    match name {
        StandardViewName::Unified => StandardView::Unified,
        StandardViewName::Unread => StandardView::Unread,
        StandardViewName::Flagged => StandardView::Flagged,
    }
}
