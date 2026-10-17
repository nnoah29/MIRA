//! Lignes non étiquetées d'une ouverture de boîte.

use crate::imap::response::flags::flag_list;
use crate::imap::response::reply::Reply;
use crate::imap::store::view::MailboxView;

const SYSTEM_FLAGS: [&str; 5] = ["\\Answered", "\\Flagged", "\\Deleted", "\\Seen", "\\Draft"];

/// Annonce drapeaux, compteurs, UIDVALIDITY, UIDNEXT et HIGHESTMODSEQ
pub(crate) fn announce_open(reply: &mut Reply, view: &MailboxView, condstore: bool) {
    let mut flags: Vec<String> = SYSTEM_FLAGS
        .iter()
        .map(|flag| (*flag).to_string())
        .collect();
    for flag in &view.flags {
        if !flags.iter().any(|known| known.eq_ignore_ascii_case(flag)) {
            flags.push(flag.clone());
        }
    }
    reply.untagged(&format!("FLAGS {}", flag_list(&flags)));
    reply.untagged(
        "OK [PERMANENTFLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft \\*)] Flags permitted",
    );
    reply.untagged(&format!("{} EXISTS", view.uids.len()));
    reply.untagged("0 RECENT");
    reply.untagged(&format!("OK [UIDVALIDITY {}] UIDs valid", view.validity));
    reply.untagged(&format!("OK [UIDNEXT {}] Predicted next UID", view.uidnext));
    if condstore {
        reply.untagged(&format!(
            "OK [HIGHESTMODSEQ {}] Highest",
            view.highest_modseq
        ));
    }
}
