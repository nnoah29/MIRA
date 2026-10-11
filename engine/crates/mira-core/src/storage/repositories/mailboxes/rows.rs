//! Conversion entre une ligne de la table `mailboxes` et une `Mailbox`.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{
    AccountId, ErrorCategory, ErrorContext, Mailbox, MailboxId, MiraError, SyncMarker,
};
use rusqlite::types::Value;

use super::role::{role_from_name, role_name};

pub(super) const COLUMNS: &str = "id, account_id, name, display_name, delimiter, parent_id, role, sync_enabled, uid_validity, uid_next, highest_modseq, total, unread";

type RawMarker = (Option<i64>, Option<i64>, Option<i64>);

/// Ligne brute de la table `mailboxes`, dans l'ordre de `COLUMNS`.
pub(super) struct RawMailbox {
    id: String,
    account: String,
    name: String,
    display_name: String,
    delimiter: Option<String>,
    parent: Option<String>,
    role: String,
    sync_enabled: bool,
    marker: RawMarker,
    counters: (i64, i64),
}

impl RawMailbox {
    /// Lit une ligne sélectionnée avec `COLUMNS`
    pub(super) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            account: row.get(1)?,
            name: row.get(2)?,
            display_name: row.get(3)?,
            delimiter: row.get(4)?,
            parent: row.get(5)?,
            role: row.get(6)?,
            sync_enabled: row.get(7)?,
            marker: (row.get(8)?, row.get(9)?, row.get(10)?),
            counters: (row.get(11)?, row.get(12)?),
        })
    }

    /// Reconstruit la boîte du domaine depuis la ligne
    pub(super) fn into_mailbox(self) -> Result<Mailbox, MiraError> {
        Ok(Mailbox {
            id: MailboxId::from_raw(self.id),
            account: AccountId::from_raw(self.account),
            name: self.name,
            display_name: self.display_name,
            delimiter: self.delimiter,
            parent: self.parent.map(MailboxId::from_raw),
            role: role_from_name(&self.role)?,
            sync_enabled: self.sync_enabled,
            marker: marker_of(self.marker),
            total: u64::try_from(self.counters.0).unwrap_or(0),
            unread: u64::try_from(self.counters.1).unwrap_or(0),
        })
    }
}

/// Reconstruit le repère de synchronisation si validité et prochain UID sont connus
fn marker_of(raw: RawMarker) -> Option<SyncMarker> {
    match raw {
        (Some(validity), Some(next), modseq) => Some(SyncMarker {
            uid_validity: u32::try_from(validity).unwrap_or(0),
            uid_next: u32::try_from(next).unwrap_or(0),
            highest_modseq: modseq.and_then(|value| u64::try_from(value).ok()),
        }),
        _ => None,
    }
}

/// Valeurs d'insertion d'une boîte, dans l'ordre de `COLUMNS`
pub(super) fn mailbox_values(mailbox: &Mailbox) -> Vec<Value> {
    let marker = mailbox.marker;
    let modseq = marker.and_then(|m| m.highest_modseq.and_then(|v| i64::try_from(v).ok()));
    vec![
        Value::from(mailbox.id.as_str().to_string()),
        Value::from(mailbox.account.as_str().to_string()),
        Value::from(mailbox.name.clone()),
        Value::from(mailbox.display_name.clone()),
        Value::from(mailbox.delimiter.clone()),
        Value::from(mailbox.parent.as_ref().map(|id| id.as_str().to_string())),
        Value::from(role_name(mailbox.role).to_string()),
        Value::from(i64::from(mailbox.sync_enabled)),
        Value::from(marker.map(|m| i64::from(m.uid_validity))),
        Value::from(marker.map(|m| i64::from(m.uid_next))),
        Value::from(modseq),
        Value::from(i64::try_from(mailbox.total).unwrap_or(i64::MAX)),
        Value::from(i64::try_from(mailbox.unread).unwrap_or(i64::MAX)),
    ]
}

/// Erreur « boîte inconnue » portant l'identifiant visé
pub(crate) fn mailbox_not_found(id: &MailboxId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::MAILBOX,
        "boîte inconnue",
    )
    .with_context(ErrorContext {
        mailbox: Some(id.clone()),
        ..ErrorContext::default()
    })
}
