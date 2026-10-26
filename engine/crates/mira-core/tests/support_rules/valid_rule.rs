//! Règle valide de référence pour la validation : limitée à acc-1, elle range dans mb-1.

use super::probe::{cond, rule, text};
use mira_vocabulary::{AccountId, ConditionTest, MailboxId, Rule, RuleAction, RuleScope, TextOp};

/// Règle correcte limitée à acc-1, qui range dans mb-1
pub fn valid_rule() -> Rule {
    let test = ConditionTest::Subject {
        matcher: text(TextOp::Contains, "facture"),
    };
    let actions = vec![RuleAction::MoveTo {
        mailbox: MailboxId::from_raw("mb-1"),
    }];
    let mut result = rule("r1", vec![vec![cond(test)]], actions);
    result.scope = RuleScope::Accounts(vec![AccountId::from_raw("acc-1")]);
    result
}

/// Règle valide dont l'unique action est remplacée
pub fn with_action(action: RuleAction) -> Rule {
    let mut rule = valid_rule();
    rule.actions = vec![action];
    rule
}
