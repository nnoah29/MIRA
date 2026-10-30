//! Lignes de `ai_journal` : colonnes lues dans l'ordre et conversion vers `AiJournalEntry`.

use mira_vocabulary::{
    AiConnectionId, AiJournalEntry, AiProviderId, MiraError, OperationId, Timestamp,
};
use rusqlite::Row;

use crate::storage::json_columns::from_json;

pub(super) const COLUMNS: &str = "id, at, operation, connection, provider, model, local, kinds, parts, chars, masked, input_tokens, output_tokens, outcome";

/// Lit une ligne dans l'ordre de `COLUMNS` ; une liste de types illisible est une base corrompue
pub(super) fn entry_from_row(row: &Row<'_>) -> rusqlite::Result<Result<AiJournalEntry, MiraError>> {
    let kinds: String = row.get(7)?;
    let entry = AiJournalEntry {
        id: row.get(0)?,
        at: Timestamp::from_millis(row.get(1)?),
        operation: OperationId::from_raw(row.get::<_, String>(2)?),
        connection: AiConnectionId::from_raw(row.get::<_, String>(3)?),
        provider: AiProviderId(row.get(4)?),
        model: row.get(5)?,
        local: row.get::<_, i64>(6)? != 0,
        kinds: Vec::new(),
        parts: row.get(8)?,
        chars: row.get(9)?,
        masked: row.get(10)?,
        input_tokens: row.get(11)?,
        output_tokens: row.get(12)?,
        outcome: row.get(13)?,
    };
    Ok(from_json(&kinds).map(|kinds| AiJournalEntry { kinds, ..entry }))
}
