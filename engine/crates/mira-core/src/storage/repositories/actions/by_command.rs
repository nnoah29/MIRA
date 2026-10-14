//! Recherche d'une action par clé d'idempotence et commande, sans connaître le compte.

use mira_vocabulary::{ActionKind, IdempotencyKey, MiraError, PendingAction};
use rusqlite::{OptionalExtension, params};

use super::rows::{COLUMNS, RawAction};
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::json_columns::to_json;

impl Tx<'_> {
    /// Action déjà enregistrée sous cette clé pour cette commande exacte, même si son objet a disparu
    pub fn find_action_by_key_and_kind(
        &self,
        key: &IdempotencyKey,
        kind: &ActionKind,
    ) -> Result<Option<PendingAction>, MiraError> {
        let raw = self
            .sql()
            .query_row(
                &format!(
                    "SELECT {COLUMNS} FROM actions WHERE idempotency_key = ?1 AND kind_json = ?2"
                ),
                params![key.0, to_json(kind)?],
                RawAction::from_row,
            )
            .optional()
            .map_err(storage_error)?;
        raw.map(RawAction::into_action).transpose()
    }
}
