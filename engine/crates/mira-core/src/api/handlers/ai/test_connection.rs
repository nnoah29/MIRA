//! Essai d'une connexion d'IA : relevé frais de la liste des modèles, aucun jeton consommé.

use mira_vocabulary::{AiConnectionId, MiraError};

use super::connection_view::{done, read_connection};
use crate::ai::models::fetch_models;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Interroge le service avec la clé de la connexion et garde la liste reçue
pub(crate) async fn handle_test_ai_connection(
    engine: &Engine,
    id: &AiConnectionId,
) -> Result<Response, MiraError> {
    let connection = read_connection(engine, id)?;
    fetch_models(engine, &connection, true).await?;
    Ok(done())
}
