//! Écouteur de redirection en boucle locale : accepte la requête `/callback` puis se ferme.

use std::time::Duration;

use mira_vocabulary::error::codes::{authentication, invalid_input, network};
use mira_vocabulary::{ErrorCategory, MiraError};
use tokio::net::TcpListener;
use tokio::task::JoinSet;
use tokio::time::timeout;

use super::callback::AuthorizationCode;
use super::connection::serve_connection;
use super::pending::PendingFlow;
use super::phase::FlowPhase;

/// Attend la redirection du navigateur, vérifie `state` et ferme l'écouteur
pub async fn wait_for_redirect(
    flow: &mut PendingFlow,
    limit: Duration,
) -> Result<AuthorizationCode, MiraError> {
    let listener = match (flow.phase, flow.listener.take()) {
        (FlowPhase::AwaitingRedirect, Some(listener)) => listener,
        _ => return Err(wrong_phase()),
    };
    let served = timeout(limit, serve_until_callback(&listener, &flow.state)).await;
    let result = served.unwrap_or_else(|_| {
        Err(MiraError::new(
            ErrorCategory::Network,
            network::TIMEOUT,
            "redirection non reçue à temps",
        ))
    });
    flow.phase = match &result {
        Ok(_) => FlowPhase::CodeReceived,
        Err(error) if error.code == authentication::OAUTH_CANCELLED.0 => FlowPhase::Cancelled,
        Err(_) => FlowPhase::Failed,
    };
    result
}

/// Erreur d'usage : le flux n'attend plus de redirection
pub(super) fn wrong_phase() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "flux d'autorisation dans un état incompatible",
    )
}

/// Sert chaque connexion dans sa propre tâche jusqu'à une redirection portant le bon `state`
async fn serve_until_callback(
    listener: &TcpListener,
    state: &str,
) -> Result<AuthorizationCode, MiraError> {
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _) = accepted.map_err(|_| listener_lost())?;
                connections.spawn(serve_connection(stream, state.to_string()));
            }
            Some(finished) = connections.join_next() => {
                if let Ok(Some(result)) = finished {
                    return result;
                }
            }
        }
    }
}

/// Erreur d'un écouteur local qui ne répond plus
fn listener_lost() -> MiraError {
    MiraError::new(
        ErrorCategory::Network,
        network::CONNECTION_LOST,
        "écouteur local interrompu",
    )
}
