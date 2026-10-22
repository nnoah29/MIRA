//! Lecture des règles avec leur statut, calculé d'après les comptes et boîtes qui existent maintenant.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MiraError, Rule, RuleId};

use crate::api::contract::RuleView;
use crate::rule_context::StoredRuleWorld;
use crate::rules::validate::rule_status;
use crate::storage::Tx;

/// Associe à la règle son statut dans ce monde
pub(super) fn view_of(rule: Rule, world: &StoredRuleWorld) -> RuleView {
    let status = rule_status(&rule, world);
    RuleView { rule, status }
}

/// Toutes les règles, dans l'ordre de leur position, avec leur statut
pub(super) fn all_views(tx: &Tx<'_>) -> Result<Vec<RuleView>, MiraError> {
    let world = StoredRuleWorld::load(tx)?;
    let rules = tx.list_rules()?;
    Ok(rules
        .into_iter()
        .map(|rule| view_of(rule, &world))
        .collect())
}

/// Relit une règle ou échoue avec `not_found.rule`
pub(super) fn require_rule(tx: &Tx<'_>, id: &RuleId) -> Result<Rule, MiraError> {
    tx.get_rule(id)?
        .ok_or_else(|| MiraError::new(ErrorCategory::NotFound, not_found::RULE, "règle inconnue"))
}

/// Relit une règle avec son statut, ou échoue avec `not_found.rule`
pub(super) fn require_view(tx: &Tx<'_>, id: &RuleId) -> Result<RuleView, MiraError> {
    let rule = require_rule(tx, id)?;
    let world = StoredRuleWorld::load(tx)?;
    Ok(view_of(rule, &world))
}
