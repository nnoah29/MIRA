//! Règle complète : portée, alternatives de conditions et actions.

use serde::{Deserialize, Serialize};

use super::action::RuleAction;
use super::condition::ConditionGroup;
use crate::ids::kinds::{AccountId, RuleId};

/// Version courante du format de définition d'une règle.
pub const CURRENT_RULE_SCHEMA_VERSION: u32 = 1;

/// Comptes auxquels s'applique une règle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "accounts", rename_all = "snake_case")]
pub enum RuleScope {
    AllAccounts,
    Accounts(Vec<AccountId>),
}

/// Règle automatique ; l'ordre entre règles est porté par le stockage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub enabled: bool,
    pub scope: RuleScope,
    pub alternatives: Vec<ConditionGroup>,
    pub actions: Vec<RuleAction>,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
}

/// Version de schéma attribuée aux règles qui n'en déclarent pas.
fn default_schema_version() -> u32 {
    CURRENT_RULE_SCHEMA_VERSION
}
