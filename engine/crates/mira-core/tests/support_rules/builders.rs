//! Construction de conditions et de règles minimales.

use mira_vocabulary::{
    AccountId, Condition, ConditionGroup, ConditionTest, Rule, RuleAction, RuleId, RuleScope,
    TextMatcher, TextOp,
};

/// Condition positive
pub fn cond(test: ConditionTest) -> Condition {
    Condition {
        negated: false,
        test,
    }
}

/// Condition inversée
pub fn not(test: ConditionTest) -> Condition {
    Condition {
        negated: true,
        test,
    }
}

/// Filtre texte à une valeur
pub fn text(op: TextOp, value: &str) -> TextMatcher {
    TextMatcher {
        op,
        values: vec![value.to_string()],
    }
}

/// Règle activée sur tous les comptes, avec des groupes OU de conditions ET
pub fn rule(id: &str, groups: Vec<Vec<Condition>>, actions: Vec<RuleAction>) -> Rule {
    Rule {
        id: RuleId::from_raw(id),
        name: id.to_string(),
        enabled: true,
        scope: RuleScope::AllAccounts,
        alternatives: groups
            .into_iter()
            .map(|conditions| ConditionGroup { conditions })
            .collect(),
        actions,
        schema_version: 1,
    }
}

/// Règle limitée à une liste de comptes
pub fn scoped(id: &str, accounts: &[&str], condition: Condition) -> Rule {
    let mut result = rule(id, vec![vec![condition]], vec![RuleAction::Archive]);
    result.scope = RuleScope::Accounts(accounts.iter().map(|a| AccountId::from_raw(*a)).collect());
    result
}
