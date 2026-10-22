//! Aperçu d'une règle sur les messages existants, sans l'enregistrer ni rien modifier.

use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{ErrorCategory, MiraError, Rule};

use super::preview_scan::{count_matches, load_contexts};
use crate::api::contract::{Response, RuleResponse};
use crate::api::engine::Engine;
use crate::rules::validate::validate_shape;

/// Nombre maximal d'exemples rendus par un aperçu.
pub const MAX_PREVIEW_SAMPLES: u32 = 50;

/// Contrôle la forme, lit les messages sous le verrou de la base, puis évalue hors verrou et hors fil asynchrone
pub(crate) async fn handle_preview_rule(
    engine: &Engine,
    rule: Rule,
    limit: u32,
) -> Result<Response, MiraError> {
    validate_shape(&rule)?;
    let probe = Rule {
        enabled: true,
        ..rule
    };
    let now = engine.config.clock.now();
    let contexts = engine
        .database
        .read(|tx| load_contexts(tx, &probe.scope, now))?;
    let samples = limit.min(MAX_PREVIEW_SAMPLES);
    let preview = tokio::task::spawn_blocking(move || count_matches(&probe, contexts, samples))
        .await
        .map_err(|_| {
            MiraError::new(
                ErrorCategory::Internal,
                internal::BUG,
                "aperçu de règle interrompu",
            )
        })?;
    Ok(Response::Rule(RuleResponse::Preview(preview)))
}
