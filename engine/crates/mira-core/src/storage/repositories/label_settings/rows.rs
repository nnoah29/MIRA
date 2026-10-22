//! Conversion des lignes de `label_settings`.

use mira_vocabulary::{CounterMode, LabelKind, LabelRef, LabelSettings};
use rusqlite::Row;

pub(super) const COLUMNS: &str = "hidden, hide_from_inbox, notifications, counter, color, position";

/// Lit les réglages d'une ligne, dans l'ordre de `COLUMNS` à partir de `first`
pub(super) fn settings_from_row(row: &Row<'_>, first: usize) -> rusqlite::Result<LabelSettings> {
    Ok(LabelSettings {
        hidden: row.get(first)?,
        hide_from_inbox: row.get(first + 1)?,
        notifications: row.get(first + 2)?,
        counter: CounterMode::from_db(&row.get::<_, String>(first + 3)?),
        color: row.get(first + 4)?,
        position: row.get(first + 5)?,
    })
}

/// Libellé désigné par une ligne, ou rien si la nature stockée est inconnue
pub(super) fn label_of(kind: &str, id: String) -> Option<LabelRef> {
    let kind = match kind {
        "server" => LabelKind::Server,
        "local" => LabelKind::Local,
        _ => return None,
    };
    Some(LabelRef { kind, id })
}
