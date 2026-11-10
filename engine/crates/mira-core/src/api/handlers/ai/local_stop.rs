//! Arrêt du serveur local à la demande : la mémoire est rendue, le modèle reste installé.

use mira_vocabulary::MiraError;

use super::connection_view::done;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Arrête le serveur s'il tourne ; sans effet sinon
pub(crate) async fn handle_stop_local_runtime(engine: &Engine) -> Result<Response, MiraError> {
    engine.ai.local_runtime.stop().await;
    Ok(done())
}
