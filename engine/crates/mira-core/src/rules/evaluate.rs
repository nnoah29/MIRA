//! Parcours des règles triées : portée, déclenchement, actions et arrêt.

use super::action_translation::apply_actions;
use super::context::MessageContext;
use super::group_eval::rule_fires;
use super::outcome::RuleOutcome;
use mira_vocabulary::{AccountId, Rule, RuleScope};

/// Évalue les règles, supposées triées par position, sur un message.
pub fn evaluate(rules: &[Rule], ctx: &MessageContext) -> RuleOutcome {
    let mut outcome = RuleOutcome::default();
    for rule in rules {
        if !rule.enabled || !scope_covers(&rule.scope, &ctx.summary.account) {
            continue;
        }
        if !rule_fires(rule, ctx) {
            continue;
        }
        outcome.matched.push(rule.id.clone());
        if apply_actions(&rule.actions, &mut outcome) {
            break;
        }
    }
    outcome
}

/// La portée contient le compte ; une liste vide ne contient aucun compte.
fn scope_covers(scope: &RuleScope, account: &AccountId) -> bool {
    match scope {
        RuleScope::AllAccounts => true,
        RuleScope::Accounts(accounts) => accounts.contains(account),
    }
}
