//! Règles incorrectes, chacune refusée par une règle de validation différente (spec R1 §5).

use mira_vocabulary::{AccountId, ConditionTest, Rule, RuleAction, RuleScope, TextOp};

use super::samples::{move_rule, subject_rule};

/// Règle « objet correspond à l'expression régulière `pattern` »
pub fn regex_rule(name: &str, pattern: &str) -> Rule {
    let mut rule = subject_rule(name, pattern, RuleScope::AllAccounts);
    if let ConditionTest::Subject { matcher } = &mut rule.alternatives[0].conditions[0].test {
        matcher.op = TextOp::Regex;
    }
    rule
}

/// Sans action, regex invalide, boîte hors d'une portée à un compte, boîte inconnue, actions contradictoires
pub fn invalid_rules(account: &AccountId) -> Vec<Rule> {
    let mut no_action = subject_rule("Vide", "x", RuleScope::AllAccounts);
    no_action.actions.clear();
    let mut wide_move = move_rule(account, "box-work");
    wide_move.scope = RuleScope::AllAccounts;
    let mut contradictory = subject_rule("Conflit", "x", RuleScope::AllAccounts);
    contradictory.actions = vec![
        RuleAction::MarkRead { read: true },
        RuleAction::MarkRead { read: false },
    ];
    vec![
        no_action,
        regex_rule("Regex", "("),
        wide_move,
        move_rule(account, "box-nowhere"),
        contradictory,
    ]
}
