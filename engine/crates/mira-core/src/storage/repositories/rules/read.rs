//! Lectures sur les règles.

use mira_vocabulary::{AccountId, MiraError, Rule, RuleId};
use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, RawRule};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit une règle, ou rien si elle n'existe pas
    pub fn get_rule(&self, id: &RuleId) -> Result<Option<Rule>, MiraError> {
        let raw = self
            .sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM rules WHERE id = ?1"),
                [id.as_str()],
                RawRule::from_row,
            )
            .optional()
            .map_err(storage_error)?;
        raw.map(|raw| self.complete_rule(raw)).transpose()
    }

    /// Liste toutes les règles dans l'ordre de leur position
    pub fn list_rules(&self) -> Result<Vec<Rule>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM rules ORDER BY position, id"
            ))
            .map_err(storage_error)?;
        let raws = statement
            .query_map([], RawRule::from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        raws.into_iter()
            .map(|raw| self.complete_rule(raw))
            .collect()
    }

    /// Joint les comptes rattachés à la ligne brute pour former la règle
    fn complete_rule(&self, raw: RawRule) -> Result<Rule, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT account_id FROM rule_accounts WHERE rule_id = ?1 ORDER BY account_id")
            .map_err(storage_error)?;
        let accounts = statement
            .query_map([raw.id.as_str()], |row| row.get::<_, String>(0))
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        raw.into_rule(accounts.into_iter().map(AccountId::from_raw).collect())
    }
}
