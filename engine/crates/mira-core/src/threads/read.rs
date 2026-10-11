//! Lectures locales des fils : liste d'une boîte et contenu d'un fil.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{
    ErrorCategory, MailboxId, MessageSummary, MiraError, Page, PageRequest, Thread, ThreadId,
};

use crate::storage::Database;

/// Liste une page de fils d'une boîte, du plus récemment actif au plus ancien
pub fn list_threads(
    db: &Database,
    mailbox: &MailboxId,
    page: &PageRequest,
) -> Result<Page<Thread>, MiraError> {
    db.read(|tx| tx.list_threads_in_mailbox(mailbox, page))
}

/// Relit un fil et ses messages, du plus ancien au plus récent
pub fn get_thread(
    db: &Database,
    id: &ThreadId,
) -> Result<(Thread, Vec<MessageSummary>), MiraError> {
    db.read(|tx| {
        let thread = tx.get_thread(id)?.ok_or_else(thread_not_found)?;
        Ok((thread, tx.messages_of_thread(id)?))
    })
}

/// Erreur « fil inconnu »
fn thread_not_found() -> MiraError {
    MiraError::new(ErrorCategory::NotFound, not_found::THREAD, "fil inconnu")
}
