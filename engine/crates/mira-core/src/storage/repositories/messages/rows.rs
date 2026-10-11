//! Conversion entre une ligne de la table `messages` et un `MessageSummary`.

use mira_vocabulary::{AccountId, Flags, MailboxId, MessageId, MessageSummary, MiraError};
use mira_vocabulary::{ThreadId, Timestamp, Unsubscribe};

use super::content_state::content_state_from_name;
use crate::storage::json_columns::from_json;

pub(crate) const COLUMNS: &str = "m.id, m.account_id, m.thread_id, m.from_json, m.to_json, m.cc_json, m.bcc_json, m.reply_to_json, m.subject, m.date, m.internal_date, m.message_id_header, m.in_reply_to, m.references_json, m.seen, m.flagged, m.answered, m.forwarded, m.size, m.preview, m.has_attachments, m.unsubscribe_json, m.content_state, m.sort_date";

/// Ligne brute de la table `messages`, dans l'ordre de `COLUMNS`.
pub(super) struct RawMessage {
    pub(super) id: String,
    account: String,
    thread: Option<String>,
    addresses: [String; 5],
    subject: String,
    dates: (Option<i64>, Option<i64>),
    headers: (Option<String>, Option<String>, String),
    flags: [bool; 4],
    size: i64,
    preview: String,
    has_attachments: bool,
    unsubscribe_json: Option<String>,
    content_state: String,
    pub(super) sort_date: i64,
}

impl RawMessage {
    /// Lit une ligne sélectionnée avec `COLUMNS`
    pub(super) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            account: row.get(1)?,
            thread: row.get(2)?,
            addresses: [
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ],
            subject: row.get(8)?,
            dates: (row.get(9)?, row.get(10)?),
            headers: (row.get(11)?, row.get(12)?, row.get(13)?),
            flags: [row.get(14)?, row.get(15)?, row.get(16)?, row.get(17)?],
            size: row.get(18)?,
            preview: row.get(19)?,
            has_attachments: row.get(20)?,
            unsubscribe_json: row.get(21)?,
            content_state: row.get(22)?,
            sort_date: row.get(23)?,
        })
    }

    /// Reconstruit le résumé du domaine depuis la ligne et les boîtes où il apparaît
    pub(super) fn into_summary(
        self,
        mailboxes: Vec<MailboxId>,
    ) -> Result<MessageSummary, MiraError> {
        let [from, to, cc, bcc, reply_to] = &self.addresses;
        Ok(MessageSummary {
            id: MessageId::from_raw(self.id),
            account: AccountId::from_raw(self.account),
            thread: self.thread.map(ThreadId::from_raw),
            from: from_json(from)?,
            to: from_json(to)?,
            cc: from_json(cc)?,
            bcc: from_json(bcc)?,
            reply_to: from_json(reply_to)?,
            subject: self.subject,
            date: self.dates.0.map(Timestamp::from_millis),
            internal_date: self.dates.1.map(Timestamp::from_millis),
            message_id_header: self.headers.0,
            in_reply_to: self.headers.1,
            references: from_json(&self.headers.2)?,
            flags: flags_of(self.flags),
            size: u64::try_from(self.size).unwrap_or(0),
            preview: self.preview,
            has_attachments: self.has_attachments,
            unsubscribe: unsubscribe_of(self.unsubscribe_json.as_deref())?,
            content: content_state_from_name(&self.content_state)?,
            mailboxes,
        })
    }
}

/// Assemble les drapeaux lus (lu, suivi, répondu, transféré)
fn flags_of(values: [bool; 4]) -> Flags {
    Flags {
        seen: values[0],
        flagged: values[1],
        answered: values[2],
        forwarded: values[3],
    }
}

/// Relit les moyens de désabonnement stockés en JSON, s'il y en a
fn unsubscribe_of(json: Option<&str>) -> Result<Option<Unsubscribe>, MiraError> {
    json.map(from_json).transpose()
}
