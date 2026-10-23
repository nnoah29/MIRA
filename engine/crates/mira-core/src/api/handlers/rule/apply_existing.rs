//! Application d'une règle aux messages déjà présents : mêmes actions et mêmes libellés que pour un message arrivé.

use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{ErrorCategory, IdempotencyKey, MiraError, Rule, RuleId};

use super::apply_existing_submit::{apply_labels, submit_actions};
use super::preview_scan::load_contexts;
use super::reading::require_view;
use crate::actions::errors::input::invalid_parameter;
use crate::api::contract::{Response, RulePreview, RuleResponse};
use crate::api::engine::Engine;
use crate::rules::context::MessageContext;
use crate::rules::evaluate::evaluate;
use crate::rules::validate::RuleStatus;

/// Nombre maximal de messages examinés quand une règle est appliquée à l'existant (les plus récents).
pub const APPLY_SCAN_LIMIT: u32 = 5000;

/// Applique la règle aux messages présents du miroir : actions mises en file, libellés locaux posés, rien d'envoyé avant le délai d'annulation
pub(crate) async fn handle_apply_rule_to_existing(
    engine: &Engine,
    id: &RuleId,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let rule = applicable_rule(engine, id)?;
    let now = engine.config.clock.now();
    let contexts = engine
        .database
        .read(|tx| load_contexts(tx, &rule.scope, now, APPLY_SCAN_LIMIT))?;
    let scanned = u64::try_from(contexts.len()).unwrap_or(u64::MAX);
    let probe = rule.clone();
    let matched = tokio::task::spawn_blocking(move || matching(&probe, contexts))
        .await
        .map_err(|_| interrupted())?;
    if let Some(first) = matched.first() {
        let outcome = evaluate(std::slice::from_ref(&rule), first);
        submit_actions(engine, &outcome, &key, &matched)?;
        apply_labels(engine, &outcome, &matched)?;
    }
    Ok(report(scanned, &matched))
}

/// La règle, si elle existe, est valide et activée ; sinon refus sans rien écrire
fn applicable_rule(engine: &Engine, id: &RuleId) -> Result<Rule, MiraError> {
    let view = engine.database.read(|tx| require_view(tx, id))?;
    if view.status != RuleStatus::Valid || !view.rule.enabled {
        return Err(invalid_parameter("règle invalide ou désactivée"));
    }
    Ok(view.rule)
}

/// Contextes des messages que la règle déclenche, du plus récent au plus ancien
fn matching(rule: &Rule, contexts: Vec<MessageContext>) -> Vec<MessageContext> {
    contexts
        .into_iter()
        .filter(|context| {
            !evaluate(std::slice::from_ref(rule), context)
                .matched
                .is_empty()
        })
        .collect()
}

/// Bilan rendu au client : messages examinés, touchés, et quelques exemples
fn report(scanned: u64, matched: &[MessageContext]) -> Response {
    Response::Rule(RuleResponse::Preview(RulePreview {
        matched: u64::try_from(matched.len()).unwrap_or(u64::MAX),
        scanned,
        samples: matched
            .iter()
            .take(5)
            .map(|c| c.summary.id.clone())
            .collect(),
    }))
}

/// Erreur interne quand le calcul en arrière-plan est interrompu
fn interrupted() -> MiraError {
    MiraError::new(
        ErrorCategory::Internal,
        internal::BUG,
        "application de règle interrompue",
    )
}
