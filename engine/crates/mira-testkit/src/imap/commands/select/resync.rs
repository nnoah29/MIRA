//! Resynchronisation QRESYNC à l'ouverture d'une boîte.

use super::params::QresyncParams;
use crate::imap::parser::uidset::resolve_set;
use crate::imap::parser::uidset_format::format_uid_set;
use crate::imap::response::fetch_flags::flags_update;
use crate::imap::response::reply::Reply;
use crate::imap::store::view::MailboxView;

/// Annonce les UID disparus et les messages modifiés depuis le MODSEQ du client
pub(crate) fn announce_resync(reply: &mut Reply, view: &MailboxView, params: &QresyncParams) {
    if params.validity != view.validity {
        return;
    }
    let mut vanished = view.vanished.clone();
    vanished.sort_unstable();
    if let Some(known) = &params.known {
        vanished = resolve_set(known, &vanished, true).unwrap_or_default();
    }
    if !vanished.is_empty() {
        reply.untagged(&format!("VANISHED (EARLIER) {}", format_uid_set(&vanished)));
    }
    for row in &view.changed {
        reply.untagged(&flags_update(row, true));
    }
}
