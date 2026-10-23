//! Domaine « règles » (spec R1 §7) : un fichier par opération, écritures idempotentes et contrôlées.

mod apply_existing;
mod apply_existing_submit;
mod catalog;
mod create;
mod delete;
mod get;
mod idempotency;
mod list;
mod preview;
mod preview_scan;
mod reading;
mod reorder;
mod set_enabled;
mod update;
mod validate;
mod writing;

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, RuleRequest, RuleRequest as R};
use crate::api::engine::Engine;

/// Route une requête du domaine règles
pub(crate) async fn route(engine: &Engine, request: RuleRequest) -> Result<Response, MiraError> {
    match request {
        R::ListRules => list::handle_list_rules(engine),
        R::GetRule { rule } => get::handle_get_rule(engine, &rule),
        R::CreateRule {
            rule,
            idempotency_key: key,
        } => create::handle_create_rule(engine, rule, key),
        R::UpdateRule {
            rule,
            idempotency_key: key,
        } => update::handle_update_rule(engine, rule, key),
        R::DeleteRule {
            rule,
            idempotency_key: key,
        } => delete::handle_delete_rule(engine, &rule, key),
        R::SetRuleEnabled {
            rule,
            enabled,
            idempotency_key: key,
        } => set_enabled::handle_set_rule_enabled(engine, &rule, enabled, key),
        R::ReorderRules {
            order,
            idempotency_key: key,
        } => reorder::handle_reorder_rules(engine, &order, key),
        R::ValidateRule { rule } => validate::handle_validate_rule(engine, &rule),
        R::PreviewRule { rule, limit } => preview::handle_preview_rule(engine, rule, limit).await,
        R::ApplyRuleToExisting {
            rule,
            idempotency_key: key,
        } => apply_existing::handle_apply_rule_to_existing(engine, &rule, key).await,
        R::ListRuleCatalog => catalog::handle_list_rule_catalog(),
    }
}
