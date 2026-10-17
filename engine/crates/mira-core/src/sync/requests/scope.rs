//! Portée d'une demande de synchronisation.

use mira_vocabulary::MailboxId;

/// Ce qu'une demande veut synchroniser ; les demandes identiques sont fusionnées.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestScope {
    Account,
    Mailbox(MailboxId),
    Resync(MailboxId),
}
