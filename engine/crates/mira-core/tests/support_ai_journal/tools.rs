//! Aides des tests du journal : compte d'appels d'un outil et complétion suivie de la relecture de la ligne.
#![allow(dead_code)]

use mira_core::storage::Database;
use mira_vocabulary::{AiJournalEntry, AiJournalFinish, AiToolCount, OperationId};

/// Compte d'appels d'un outil.
pub fn count(name: &str, count: u32) -> AiToolCount {
    AiToolCount {
        name: name.to_string(),
        count,
    }
}

/// Complète la ligne `op-1` et rend la ligne relue.
pub fn finished_with(db: &Database, finish: AiJournalFinish) -> AiJournalEntry {
    let operation = OperationId::from_raw("op-1");
    db.write(|tx| tx.finish_ai_journal(&operation, &finish))
        .unwrap();
    db.read(|tx| tx.list_ai_journal(1, None)).unwrap().remove(0)
}
