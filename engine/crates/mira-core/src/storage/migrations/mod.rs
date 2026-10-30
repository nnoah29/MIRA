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
    include_str!("v006_rules.sql"),
    include_str!("v007_rule_idempotency.sql"),
    include_str!("v008_local_labels.sql"),
    include_str!("v009_local_label_idempotency.sql"),
    include_str!("v010_label_settings.sql"),
    include_str!("v011_engine_settings.sql"),
    include_str!("v012_message_templates.sql"),
    include_str!("v013_rule_reply_drafts.sql"),
    include_str!("v014_rule_idempotency_result.sql"),
    include_str!("v015_system_labels.sql"),
    include_str!("v016_ai_connections.sql"),
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
