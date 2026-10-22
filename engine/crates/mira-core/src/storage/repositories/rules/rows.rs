//! Conversion entre une ligne de la table `rules` et une `Rule`.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{
    AccountId, ConditionGroup, ErrorCategory, MiraError, Rule, RuleAction, RuleId, RuleScope,
};
use serde::{Deserialize, Serialize};

use crate::storage::json_columns::{from_json, to_json};

pub(super) const COLUMNS: &str = "id, name, enabled, scope_all, schema_version, definition";

/// Contenu de la colonne `definition` : la portée n'y figure jamais.
#[derive(Serialize, Deserialize)]
pub(super) struct StoredDefinition {
    pub(super) alternatives: Vec<ConditionGroup>,
    pub(super) actions: Vec<RuleAction>,
}

/// Ligne brute de la table `rules`, dans l'ordre de `COLUMNS`.
pub(super) struct RawRule {
    pub(super) id: String,
    name: String,
    enabled: i64,
    scope_all: i64,
    schema_version: i64,
    definition: String,
}

impl RawRule {
    /// Lit une ligne sélectionnée avec `COLUMNS`
    pub(super) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            enabled: row.get(2)?,
            scope_all: row.get(3)?,
            schema_version: row.get(4)?,
            definition: row.get(5)?,
        })
    }

    /// Reconstruit la règle du domaine ; la portée vient de `scope_all` et des comptes rattachés
    pub(super) fn into_rule(self, accounts: Vec<AccountId>) -> Result<Rule, MiraError> {
        let definition: StoredDefinition = from_json(&self.definition)?;
        let scope = if self.scope_all != 0 {
            RuleScope::AllAccounts
        } else {
            RuleScope::Accounts(accounts)
        };
        Ok(Rule {
            id: RuleId::from_raw(self.id),
            name: self.name,
            enabled: self.enabled != 0,
            scope,
            alternatives: definition.alternatives,
            actions: definition.actions,
            schema_version: u32::try_from(self.schema_version).unwrap_or(0),
        })
    }
}

/// Sérialise alternatives et actions pour la colonne `definition`
pub(super) fn definition_json(rule: &Rule) -> Result<String, MiraError> {
    to_json(&StoredDefinition {
        alternatives: rule.alternatives.clone(),
        actions: rule.actions.clone(),
    })
}

/// Erreur « règle inconnue »
pub(super) fn rule_not_found() -> MiraError {
    MiraError::new(ErrorCategory::NotFound, not_found::RULE, "règle inconnue")
}
