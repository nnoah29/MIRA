//! Règles de test.

use mira_core::storage::Database;
use mira_vocabulary::{
    AccountId, Condition, ConditionGroup, ConditionTest, Rule, RuleAction, RuleId, RuleScope,
};

/// Règle de test : marque comme lu les messages envoyés à moi, portée donnée
pub fn sample_rule(id: &str, scope: RuleScope) -> Rule {
    Rule {
        id: RuleId::from_raw(id),
        name: format!("Règle {id}"),
        enabled: true,
        scope,
        alternatives: vec![ConditionGroup {
            conditions: vec![Condition {
                negated: false,
                test: ConditionTest::MeInTo,
            }],
        }],
        actions: vec![RuleAction::MarkRead { read: true }],
        schema_version: 1,
    }
}

/// Portée limitée aux comptes donnés
pub fn scope_of(accounts: &[&str]) -> RuleScope {
    RuleScope::Accounts(accounts.iter().map(|a| AccountId::from_raw(*a)).collect())
}

/// Identifiant de règle de test
pub fn id(raw: &str) -> RuleId {
    RuleId::from_raw(raw)
}

/// Identifiants des règles stockées, dans l'ordre de la liste
pub fn rule_ids(db: &Database) -> Vec<String> {
    let rules = db.read(|tx| tx.list_rules()).unwrap();
    rules.iter().map(|r| r.id.as_str().to_string()).collect()
}
