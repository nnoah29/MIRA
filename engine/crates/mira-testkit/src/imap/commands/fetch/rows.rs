//! Sélection des messages d'un FETCH.

use crate::imap::parser::uidset::resolve_set;
use crate::imap::response::error::ImapError;
use crate::imap::store::data::StoreData;
use crate::imap::store::entry_flags::{FlagMode, owned};
use crate::imap::store::rows::MessageRow;

/// Paramètres de sélection des messages d'un FETCH.
#[derive(Debug)]
pub(crate) struct FetchRequest {
    pub user: String,
    pub mailbox: String,
    pub set: String,
    pub by_uid: bool,
    pub mark_seen: bool,
    pub labels: bool,
    pub changed_since: Option<u64>,
}

/// Choisit les messages, pose `\Seen` si besoin, puis les copie
pub(crate) fn collect_rows(
    data: &mut StoreData,
    request: &FetchRequest,
) -> Result<Vec<MessageRow>, ImapError> {
    let ids = data
        .mailbox(&request.user, &request.mailbox)
        .map(|mailbox| mailbox.uids())
        .ok_or_else(|| ImapError::no("[NONEXISTENT] Mailbox vanished"))?;
    let uids = resolve_set(&request.set, &ids, request.by_uid)?;
    if request.mark_seen {
        mark_seen(data, request, &uids);
    }
    let mut rows = data.rows(&request.user, &request.mailbox, &uids, request.labels);
    if let Some(since) = request.changed_since {
        rows.retain(|row| row.modseq > since);
    }
    Ok(rows)
}

/// Ajoute `\Seen` aux messages indiqués
fn mark_seen(data: &mut StoreData, request: &FetchRequest, uids: &[u32]) {
    if let Some(mailbox) = data.mailbox_mut(&request.user, &request.mailbox) {
        for uid in uids {
            mailbox.change_flags(*uid, FlagMode::Add, &owned(&["\\Seen"]));
        }
    }
}
