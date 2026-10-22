//! Parcours des règles triées : portée, déclenchement, actions et arrêt.

use super::action_translation::apply_actions;
use super::context::MessageContext;
use super::group_eval::rule_fires;
use super::outcome::RuleOutcome;
use mira_vocabulary::{AccountId, Rule, RuleScope};

/// Évalue les règles, supposées triées par position, sur un message ; chaque règle voit les libellés posés par les précédentes.
pub fn evaluate(rules: &[Rule], ctx: &MessageContext) -> RuleOutcome {
    let mut outcome = RuleOutcome::default();
    let mut working = ctx.clone();
    for rule in rules {
        if !rule.enabled || !scope_covers(&rule.scope, &working.summary.account) {
            continue;
        }
        if !rule_fires(rule, &working) {
            continue;
        }
        outcome.matched.push(rule.id.clone());
        let stop = apply_actions(&rule.actions, &mut outcome, &mut working.thread_labels);
        if stop {
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
