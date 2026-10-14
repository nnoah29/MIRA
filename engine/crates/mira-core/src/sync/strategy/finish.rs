//! Fin de synchronisation d'une boîte : rejeu des actions locales, compteurs, puis repère (jamais avant les données).

use mira_vocabulary::{EventKind, Mailbox, MiraError, SyncMarker};

use crate::actions::reapply_pending_local_effects;
use crate::imap::SelectedMailbox;
use crate::sync::session::SyncSession;

/// Rejoue les actions locales par-dessus le miroir, recalcule les compteurs, puis avance le repère.
pub fn finish_mailbox(
    session: &SyncSession,
    mailbox: &Mailbox,
    selected: &SelectedMailbox,
) -> Result<(), MiraError> {
    let marker = SyncMarker {
        uid_validity: selected.uid_validity,
        uid_next: selected.uid_next,
        highest_modseq: selected.highest_modseq,
    };
    let account = session.ctx.account.id.clone();
    let after = session.ctx.db.write(|tx| {
        reapply_pending_local_effects(tx, &account)?;
        tx.refresh_mailbox_counters(&mailbox.id)?;
        tx.set_mailbox_marker(&mailbox.id, Some(marker))?;
        tx.get_mailbox(&mailbox.id)
    })?;
    if let Some(fresh) = after
        && (fresh.total, fresh.unread) != (mailbox.total, mailbox.unread)
    {
        session.ctx.emit(EventKind::MailboxCountersUpdated {
            account,
            mailbox: fresh.id,
            total: fresh.total,
            unread: fresh.unread,
        });
    }
    Ok(())
}
