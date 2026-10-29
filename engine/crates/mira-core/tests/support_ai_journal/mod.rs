//! Aides des tests du journal des envois d'IA : lignes types, complétion, identifiants listés.
#![allow(dead_code)]

use mira_core::storage::Database;
use mira_vocabulary::{
    AiConnectionId, AiJournalEntry, AiJournalFinish, AiProviderId, ContentKind, OperationId,
    Timestamp,
};

/// Ligne `running` d'une génération distante de ce mail, à cette date
pub fn entry(id: &str, operation: &str, at: i64) -> AiJournalEntry {
    AiJournalEntry {
        id: id.to_string(),
        at: Timestamp::from_millis(at),
        operation: OperationId::from_raw(operation),
        connection: AiConnectionId::from_raw("conn-1"),
        provider: AiProviderId("test-provider".to_string()),
        model: "model-a".to_string(),
        local: false,
        kinds: vec![ContentKind::MailMessage],
        parts: 1,
        chars: 321,
        masked: 0,
        input_tokens: None,
        output_tokens: None,
        outcome: "running".to_string(),
    }
}

/// Ce que l'on apprend à la fin : issue, valeurs masquées, 12 jetons d'entrée et 5 de sortie
pub fn finish(outcome: &str, masked: u32) -> AiJournalFinish {
    AiJournalFinish {
        masked,
        input_tokens: Some(12),
        output_tokens: Some(5),
        outcome: outcome.to_string(),
    }
}

/// Insère ces lignes (identifiant, opération, date) dans l'ordre donné
pub fn insert_all(db: &Database, rows: &[(&str, &str, i64)]) {
    for (id, operation, at) in rows {
        let row = entry(id, operation, *at);
        db.write(|tx| tx.insert_ai_journal(&row)).unwrap();
    }
}

/// Identifiants des lignes, dans l'ordre rendu
pub fn ids(rows: &[AiJournalEntry]) -> Vec<&str> {
    rows.iter().map(|row| row.id.as_str()).collect()
}
