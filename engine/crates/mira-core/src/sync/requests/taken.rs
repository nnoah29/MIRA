//! Demandes retirées de la file pour être servies par une seule passe.

use mira_vocabulary::{MailboxId, MiraError, OperationId};

use super::queue::Ticket;
use super::scope::RequestScope;

/// Demandes fusionnées : portées, accusés à envoyer et opération que portera la passe.
#[derive(Debug, Default)]
pub struct TakenRequests {
    pub scopes: Vec<RequestScope>,
    pub tickets: Vec<Ticket>,
    pub operation: Option<OperationId>,
}

impl TakenRequests {
    /// Aucune demande n'était en attente.
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty() && self.tickets.is_empty()
    }

    /// Le compte entier est demandé.
    pub fn wants_account(&self) -> bool {
        self.scopes.contains(&RequestScope::Account)
    }

    /// Boîtes à resynchroniser de force.
    pub fn resyncs(&self) -> Vec<MailboxId> {
        let wanted = |scope: &RequestScope| match scope {
            RequestScope::Resync(mailbox) => Some(mailbox.clone()),
            RequestScope::Account | RequestScope::Mailbox(_) => None,
        };
        self.scopes.iter().filter_map(wanted).collect()
    }

    /// Boîtes à synchroniser seules.
    pub fn mailboxes(&self) -> Vec<MailboxId> {
        let wanted = |scope: &RequestScope| match scope {
            RequestScope::Mailbox(mailbox) => Some(mailbox.clone()),
            RequestScope::Account | RequestScope::Resync(_) => None,
        };
        self.scopes.iter().filter_map(wanted).collect()
    }

    /// Répond à tous les demandeurs avec l'issue de la passe.
    pub fn settle(self, outcome: &Result<(), MiraError>) {
        for ticket in self.tickets {
            let _ = ticket.send(outcome.clone());
        }
    }
}
