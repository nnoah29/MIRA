//! Oubli de souvenirs sans embedder : toujours possible, même mémoire désactivée, et sans trace dans le journal WAL.

use mira_vocabulary::MiraError;

use crate::storage::Database;

/// Supprime un souvenir puis tronque le journal ; inconnu : `not_found.ai_memory`
pub(crate) fn forget_one(db: &Database, id: &str) -> Result<(), MiraError> {
    db.write(|tx| tx.delete_ai_memory(id))?;
    db.erase_traces()
}

/// Supprime tous les souvenirs, tronque le journal et rend leur nombre
pub(crate) fn forget_everything(db: &Database) -> Result<u64, MiraError> {
    let removed = db.write(|tx| tx.delete_all_ai_memories())?;
    db.erase_traces()?;
    Ok(removed)
}
