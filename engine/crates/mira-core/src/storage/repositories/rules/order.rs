//! Réordonnancement des règles.

use std::collections::HashSet;

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError, RuleId};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Réordonne les règles ; la liste doit contenir chaque règle existante exactement une fois
    pub fn reorder_rules(&self, ordered: &[RuleId]) -> Result<(), MiraError> {
        let existing: HashSet<String> = self
            .list_rules()?
            .into_iter()
            .map(|rule| rule.id.as_str().to_string())
            .collect();
        let given: HashSet<String> = ordered.iter().map(|id| id.as_str().to_string()).collect();
        if given.len() != ordered.len() || given != existing {
            return Err(MiraError::new(
                ErrorCategory::InvalidInput,
                invalid_input::PARAMETER,
                "l'ordre doit citer chaque règle exactement une fois",
            ));
        }
        for (position, id) in (0_i64..).zip(ordered.iter()) {
            self.sql()
                .execute(
                    "UPDATE rules SET position = ?2 WHERE id = ?1",
                    rusqlite::params![id.as_str(), position],
                )
                .map_err(storage_error)?;
        }
        Ok(())
    }
}
