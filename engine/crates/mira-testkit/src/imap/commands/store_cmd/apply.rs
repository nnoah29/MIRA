//! Application d'un STORE sur le magasin.

use super::spec::StoreSpec;
use crate::imap::parser::uidset::resolve_set;
use crate::imap::response::error::ImapError;
use crate::imap::store::data::StoreData;
use crate::imap::store::rows::MessageRow;

/// Boîte et ensemble de messages visés par un STORE.
#[derive(Debug)]
pub(crate) struct StoreTarget {
    pub user: String,
    pub mailbox: String,
    pub set: String,
    pub by_uid: bool,
}

/// Résultat d'un STORE : messages modifiés, refusés par UNCHANGEDSINCE, HIGHESTMODSEQ final.
#[derive(Debug)]
pub(crate) struct StoreOutcome {
    pub rows: Vec<MessageRow>,
    pub modified: Vec<u32>,
    pub highest: u64,
}

/// Indique si le message dépasse la limite UNCHANGEDSINCE
fn is_too_recent(data: &StoreData, target: &StoreTarget, spec: &StoreSpec, uid: u32) -> bool {
    let Some(limit) = spec.unchanged_since else {
        return false;
    };
    data.mailbox(&target.user, &target.mailbox)
        .is_some_and(|found| {
            found
                .entries
                .iter()
                .any(|e| e.uid == uid && e.modseq > limit)
        })
}

/// Modifie un message ; faux s'il est refusé par UNCHANGEDSINCE
fn apply_one(data: &mut StoreData, target: &StoreTarget, spec: &StoreSpec, uid: u32) -> bool {
    if is_too_recent(data, target, spec, uid) {
        return false;
    }
    if spec.labels {
        data.change_labels(&target.user, &target.mailbox, uid, spec.mode, &spec.values);
    } else if let Some(found) = data.mailbox_mut(&target.user, &target.mailbox) {
        found.change_flags(uid, spec.mode, &spec.values);
    }
    true
}

/// Applique l'opération à tous les messages de l'ensemble
pub(crate) fn apply_store(
    data: &mut StoreData,
    target: &StoreTarget,
    spec: &StoreSpec,
) -> Result<StoreOutcome, ImapError> {
    let ids = data
        .mailbox(&target.user, &target.mailbox)
        .map(|found| found.uids())
        .ok_or_else(|| ImapError::no("[NONEXISTENT] Mailbox vanished"))?;
    let chosen = resolve_set(&target.set, &ids, target.by_uid)?;
    let (applied, modified): (Vec<u32>, Vec<u32>) = chosen
        .into_iter()
        .partition(|uid| apply_one(data, target, spec, *uid));
    let highest = data
        .mailbox(&target.user, &target.mailbox)
        .map_or(0, |found| found.highest_modseq);
    let rows = data.rows(&target.user, &target.mailbox, &applied, spec.labels);
    Ok(StoreOutcome {
        rows,
        modified,
        highest,
    })
}
