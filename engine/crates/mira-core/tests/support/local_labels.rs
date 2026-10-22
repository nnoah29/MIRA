//! Libellés locaux de test.

use mira_core::storage::Database;
use mira_vocabulary::{LocalLabel, LocalLabelId, ThreadId};

/// Libellé de test : identifiant, nom, couleur fixe
pub fn label(id: &str, name: &str) -> LocalLabel {
    LocalLabel {
        id: LocalLabelId::from_raw(id),
        name: name.to_string(),
        color: 0xFFE8_542B,
    }
}

/// Identifiant de libellé de test
pub fn label_id(raw: &str) -> LocalLabelId {
    LocalLabelId::from_raw(raw)
}

/// Insère un libellé dans la base
pub fn store_label(db: &Database, id: &str, name: &str) {
    db.write(|tx| tx.insert_local_label(&label(id, name)))
        .unwrap();
}

/// Libellés posés sur un fil, par identifiants triés
pub fn labels_on(db: &Database, thread: &ThreadId) -> Vec<String> {
    let mut found: Vec<String> = db
        .read(|tx| tx.labels_of_thread(thread))
        .unwrap()
        .iter()
        .map(|l| l.as_str().to_string())
        .collect();
    found.sort();
    found
}
