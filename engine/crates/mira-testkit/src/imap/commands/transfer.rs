//! Copie ou déplacement de messages entre boîtes, commun à COPY et MOVE.

use crate::imap::parser::uidset::resolve_set;
use crate::imap::parser::uidset_format::format_uid_set;
use crate::imap::response::error::ImapError;
use crate::imap::store::data::{Entry, StoreData};

/// Boîtes et messages concernés par un transfert.
#[derive(Debug)]
pub(crate) struct TransferRequest {
    pub user: String,
    pub from: String,
    pub to: String,
    pub set: String,
    pub by_uid: bool,
    pub remove: bool,
}

/// UID source et destination d'un transfert réussi.
#[derive(Debug)]
pub(crate) struct Transfer {
    pub validity: u32,
    pub source: Vec<u32>,
    pub target: Vec<u32>,
}

impl Transfer {
    /// Code de réponse `COPYUID validité source destination`
    pub(crate) fn copyuid(&self) -> String {
        format!(
            "COPYUID {} {} {}",
            self.validity,
            format_uid_set(&self.source),
            format_uid_set(&self.target)
        )
    }
}

/// Messages de la boîte source désignés par l'ensemble de la requête
fn select_entries(data: &StoreData, request: &TransferRequest) -> Result<Vec<Entry>, ImapError> {
    let source_box = data
        .mailbox(&request.user, &request.from)
        .ok_or_else(|| ImapError::no("[NONEXISTENT] No such mailbox"))?;
    let chosen = resolve_set(&request.set, &source_box.uids(), request.by_uid)?;
    let entries = source_box.entries.iter();
    Ok(entries
        .filter(|entry| chosen.contains(&entry.uid))
        .cloned()
        .collect())
}

/// Copie (ou déplace) les messages de l'ensemble vers la boîte de destination
pub(crate) fn transfer(
    data: &mut StoreData,
    request: &TransferRequest,
) -> Result<Transfer, ImapError> {
    let entries = select_entries(data, request)?;
    let target_box = data
        .mailbox_mut(&request.user, &request.to)
        .ok_or_else(|| ImapError::no("[TRYCREATE] No such destination mailbox"))?;
    let target: Vec<u32> = entries
        .iter()
        .map(|e| target_box.append(e.flags.clone(), e.internal_date, e.message.clone()))
        .collect();
    let validity = target_box.uidvalidity;
    let source: Vec<u32> = entries.iter().map(|entry| entry.uid).collect();
    if let (true, Some(origin)) = (
        request.remove,
        data.mailbox_mut(&request.user, &request.from),
    ) {
        origin.remove_uids(&source);
    }
    Ok(Transfer {
        validity,
        source,
        target,
    })
}
