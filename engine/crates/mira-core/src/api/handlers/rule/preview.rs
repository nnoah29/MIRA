//! Aperçu d'une règle sur les messages existants, sans l'enregistrer ni rien modifier.

use mira_vocabulary::{MiraError, Rule};

use super::preview_scan::scan_candidates;
use crate::api::contract::{Response, RuleResponse};
use crate::api::engine::Engine;

/// Nombre maximal d'exemples rendus par un aperçu.
pub const MAX_PREVIEW_SAMPLES: u32 = 50;

/// Évalue la règle, même désactivée, sur les messages récents de sa portée ; `limit` plafonne les exemples
pub(crate) fn handle_preview_rule(
    engine: &Engine,
    rule: Rule,
    limit: u32,
) -> Result<Response, MiraError> {
    let probe = Rule {
        enabled: true,
        ..rule
    };
    let now = engine.config.clock.now();
    let samples = limit.min(MAX_PREVIEW_SAMPLES);
    let preview = engine
        .database
        .read(|tx| scan_candidates(tx, &probe, now, samples))?;
    Ok(Response::Rule(RuleResponse::Preview(preview)))
}
