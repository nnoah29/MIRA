//! Recalcul des compteurs de la Réception après un changement de libellé, avec annonce.

use mira_vocabulary::{EventKind, MiraError};

use crate::events::EventSink;
use crate::storage::Database;

/// Recalcule les compteurs de chaque Réception (sans `force`, seulement si un libellé est masqué) et les annonce
pub fn refresh_inbox_counters(
    db: &Database,
    events: &dyn EventSink,
    force: bool,
) -> Result<(), MiraError> {
    let refreshed = db.write(|tx| tx.refresh_inbox_counters(force))?;
    for mailbox in refreshed {
        events.emit(EventKind::MailboxCountersUpdated {
            account: mailbox.account,
            mailbox: mailbox.id,
            total: mailbox.total,
            unread: mailbox.unread,
        })?;
    }
    Ok(())
}
