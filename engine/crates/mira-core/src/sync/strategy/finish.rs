//! Fin de synchronisation d'une boîte : rejeu des actions locales, compteurs, puis repère (jamais avant les données).

use mira_vocabulary::{EventKind, Mailbox, MiraError, SyncMarker};

use crate::actions::reapply_pending_local_effects;
use crate::imap::SelectedMailbox;
use crate::sync::session::SyncSession;
use crate::system_labels::refresh_account_labels;

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
    let now = session.ctx.clock.now().millis;
    let after = session.ctx.db.write(|tx| {
        reapply_pending_local_effects(tx, &account)?;
        tx.refresh_mailbox_counters(&mailbox.id)?;
        tx.set_mailbox_marker(&mailbox.id, Some(marker))?;
        let labels = refresh_account_labels(tx, &account, now)?;
        Ok((tx.get_mailbox(&mailbox.id)?, labels))
    })?;
    let (after, labels_changed) = after;
    if labels_changed {
        session.ctx.emit(EventKind::SystemLabelsChanged);
    }
    announce_counters(session, mailbox, after);
    Ok(())
}

/// Annonce les nouveaux compteurs de la boîte s'ils ont changé depuis le début de la passe
fn announce_counters(session: &SyncSession, before: &Mailbox, after: Option<Mailbox>) {
    if let Some(fresh) = after
        && (fresh.total, fresh.unread) != (before.total, before.unread)
    {
        session.ctx.emit(EventKind::MailboxCountersUpdated {
            account: session.ctx.account.id.clone(),
            mailbox: fresh.id,
            total: fresh.total,
            unread: fresh.unread,
        });
    }
}
