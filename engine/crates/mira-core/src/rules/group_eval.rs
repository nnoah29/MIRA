//! ET dans un groupe, OU entre groupes, inversion par condition.

use super::condition_eval::test_holds;
use super::context::MessageContext;
use mira_vocabulary::{Condition, ConditionGroup, Rule};

/// Une règle se déclenche si au moins un de ses groupes est vrai.
pub fn rule_fires(rule: &Rule, ctx: &MessageContext) -> bool {
    rule.alternatives
        .iter()
        .any(|group| group_holds(group, ctx))
}

/// Un groupe non vide est vrai si toutes ses conditions le sont.
fn group_holds(group: &ConditionGroup, ctx: &MessageContext) -> bool {
    !group.conditions.is_empty()
        && group
            .conditions
            .iter()
            .all(|condition| condition_holds(condition, ctx))
}

/// Évalue un test puis applique l'inversion.
fn condition_holds(condition: &Condition, ctx: &MessageContext) -> bool {
    test_holds(&condition.test, ctx) != condition.negated
}
