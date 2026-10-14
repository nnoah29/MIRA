//! Base SQLite d'un profil : ouverture, réglages, transactions de lecture et d'écriture.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{Connection, TransactionBehavior};

use super::errors::storage_error;
use super::migrations::apply_migrations;
use super::tx::Tx;
use mira_vocabulary::MiraError;

/// Base d'un profil ; un seul écrivain à la fois (D-27, AG-05).
#[derive(Debug)]
pub struct Database {
    connection: Mutex<Connection>,
}

impl Database {
    /// Ouvre (ou crée) la base du profil et applique les migrations
    pub fn open(path: &Path) -> Result<Self, MiraError> {
        Self::prepare(Connection::open(path).map_err(storage_error)?)
    }

    /// Ouvre une base en mémoire, pour les tests
    pub fn open_in_memory() -> Result<Self, MiraError> {
        Self::prepare(Connection::open_in_memory().map_err(storage_error)?)
    }

    /// Règle la connexion et applique les migrations
    fn prepare(mut connection: Connection) -> Result<Self, MiraError> {
        connection
            .execute_batch(
                "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA synchronous=NORMAL; PRAGMA secure_delete=ON;",
            )
            .map_err(storage_error)?;
        apply_migrations(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Exécute des écritures dans une transaction validée seulement en cas de succès
    pub fn write<T>(
        &self,
        work: impl FnOnce(&Tx<'_>) -> Result<T, MiraError>,
    ) -> Result<T, MiraError> {
        let mut connection = self.lock();
        let inner = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let tx = Tx::new(inner);
        let value = work(&tx)?;
        tx.commit().map_err(storage_error)?;
        Ok(value)
    }

    /// Exécute des lectures dans une transaction cohérente, jamais validée
    pub fn read<T>(
        &self,
        work: impl FnOnce(&Tx<'_>) -> Result<T, MiraError>,
    ) -> Result<T, MiraError> {
        let mut connection = self.lock();
        let inner = connection.transaction().map_err(storage_error)?;
        work(&Tx::new(inner))
    }

    /// Prend le verrou de la connexion en ignorant un empoisonnement
    pub(super) fn lock(&self) -> MutexGuard<'_, Connection> {
        self.connection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
