//! Écritures sur les règles : insertion, mise à jour, suppression, activation.

use mira_vocabulary::{MiraError, Rule, RuleId, RuleScope};
use rusqlite::params;

use super::rows::{definition_json, rule_not_found};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Insère une règle en dernière position
    pub fn insert_rule(&self, rule: &Rule) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT INTO rules (id, name, enabled, position, scope_all, schema_version, definition) \
                 VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(position), -1) + 1 FROM rules), ?4, ?5, ?6)",
                params![
                    rule.id.as_str(),
                    rule.name,
                    rule.enabled,
                    rule.scope == RuleScope::AllAccounts,
                    rule.schema_version,
                    definition_json(rule)?,
                ],
            )
            .map_err(storage_error)?;
        self.replace_scope_rows(rule)
    }

    /// Met à jour une règle existante sans changer sa position
    pub fn update_rule(&self, rule: &Rule) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE rules SET name = ?2, enabled = ?3, scope_all = ?4, schema_version = ?5, \
                 definition = ?6, updated_at = CAST(strftime('%s', 'now') AS INTEGER) * 1000 WHERE id = ?1",
                params![
                    rule.id.as_str(),
                    rule.name,
                    rule.enabled,
                    rule.scope == RuleScope::AllAccounts,
                    rule.schema_version,
                    definition_json(rule)?,
                ],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(rule_not_found());
        }
        self.replace_scope_rows(rule)
    }

    /// Supprime une règle et ses rattachements de comptes
    pub fn delete_rule(&self, id: &RuleId) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute("DELETE FROM rules WHERE id = ?1", [id.as_str()])
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(rule_not_found());
        }
        Ok(())
    }

    /// Active ou désactive une règle
    pub fn set_enabled(&self, id: &RuleId, enabled: bool) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE rules SET enabled = ?2 WHERE id = ?1",
                params![id.as_str(), enabled],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(rule_not_found());
        }
        Ok(())
    }

    /// Remplace les comptes rattachés selon la portée de la règle
    fn replace_scope_rows(&self, rule: &Rule) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM rule_accounts WHERE rule_id = ?1",
                [rule.id.as_str()],
            )
            .map_err(storage_error)?;
        if let RuleScope::Accounts(accounts) = &rule.scope {
            for account in accounts {
                self.sql()
                    .execute(
                        "INSERT OR IGNORE INTO rule_accounts (rule_id, account_id) VALUES (?1, ?2)",
                        [rule.id.as_str(), account.as_str()],
                    )
                    .map_err(storage_error)?;
            }
        }
        Ok(())
    }
}
