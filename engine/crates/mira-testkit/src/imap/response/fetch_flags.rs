//! Ligne FETCH annonçant les drapeaux d'un message.

use super::flags::flag_list;
use crate::imap::store::rows::MessageRow;

/// Texte `seq FETCH (UID u FLAGS (...) [MODSEQ (m)])`, sans le préfixe `* `
pub(crate) fn flags_update(row: &MessageRow, with_modseq: bool) -> String {
    let modseq = if with_modseq {
        format!(" MODSEQ ({})", row.modseq)
    } else {
        String::new()
    };
    format!(
        "{} FETCH (UID {} FLAGS {}{modseq})",
        row.seq,
        row.uid,
        flag_list(&row.flags)
    )
}
