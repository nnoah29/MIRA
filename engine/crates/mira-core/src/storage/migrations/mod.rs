//! Migrations versionnées du schéma, suivies par `PRAGMA user_version`.

use rusqlite::Connection;

use super::errors::storage_error;
use mira_vocabulary::MiraError;

const MIGRATIONS: &[&str] = &[
    include_str!("v001_initial.sql"),
    include_str!("v002_account_profile_and_default_identity.sql"),
    include_str!("v003_account_token_meta.sql"),
    include_str!("v004_reading_indexes.sql"),
    include_str!("v005_account_idempotency.sql"),
];

/// Applique, dans une transaction, les migrations pas encore appliquées
pub(crate) fn apply_migrations(connection: &mut Connection) -> Result<(), MiraError> {
    let current: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(storage_error)?;
    let transaction = connection.transaction().map_err(storage_error)?;
    for (version, script) in (1_i64..)
        .zip(MIGRATIONS.iter())
        .skip(usize::try_from(current).unwrap_or(0))
    {
        transaction.execute_batch(script).map_err(storage_error)?;
        transaction
            .pragma_update(None, "user_version", version)
            .map_err(storage_error)?;
    }
    transaction.commit().map_err(storage_error)
}

/// Version de schéma attendue par ce moteur
pub fn schema_version() -> usize {
    MIGRATIONS.len()
}
