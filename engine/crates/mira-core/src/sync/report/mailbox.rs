//! Boîtes en cours de synchronisation dans le rapport.

use mira_vocabulary::MailboxId;

use super::handle::ReportHandle;

impl ReportHandle {
    /// Note qu'une boîte est en cours de synchronisation.
    pub fn mailbox_started(&self, mailbox: &MailboxId) {
        self.update(|state| state.mailboxes.push(mailbox.clone()));
    }

    /// Note que la synchronisation de la boîte est terminée.
    pub fn mailbox_ended(&self, mailbox: &MailboxId) {
        self.update(|state| state.mailboxes.retain(|current| current != mailbox));
    }
}
