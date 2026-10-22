//! Liste des règles, dans l'ordre d'exécution, chacune avec son statut.

use mira_vocabulary::MiraError;

use super::reading::all_views;
use crate::api::contract::{Response, RuleResponse};
use crate::api::engine::Engine;

/// Rend toutes les règles par position croissante ; une règle devenue invalide y figure avec sa raison
pub(crate) fn handle_list_rules(engine: &Engine) -> Result<Response, MiraError> {
    let views = engine.database.read(all_views)?;
    Ok(Response::Rule(RuleResponse::Rules(views)))
}
