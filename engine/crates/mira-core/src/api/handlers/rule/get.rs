//! Lecture d'une règle avec son statut.

use mira_vocabulary::{MiraError, RuleId};

use super::reading::require_view;
use super::writing::single;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Rend la règle et son statut, ou `not_found.rule`
pub(crate) fn handle_get_rule(engine: &Engine, id: &RuleId) -> Result<Response, MiraError> {
    let view = engine.database.read(|tx| require_view(tx, id))?;
    Ok(single(view))
}
