//! Début d'une connexion qui rend une session : reconnexion si une session existe, sinon inscription.

use mira_vocabulary::{AiConnection, AiSessionLogin, MiraError};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

use crate::ai::session::read_session;
use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;
use crate::auth::start_session_flow;

/// Ouvre le flux, l'enregistre pour la connexion et rend son identifiant et l'adresse à ouvrir
pub(super) async fn start_session_login(
    engine: &Engine,
    connection: &AiConnection,
    login: &AiSessionLogin,
) -> Result<Response, MiraError> {
    let mut rng = StdRng::from_os_rng();
    let previous = read_session(engine, &connection.id);
    let host_id = previous
        .as_ref()
        .map_or_else(|| new_host_id(&mut rng), |session| session.host_id.clone());
    let pending = start_session_flow(login, &host_id, previous.as_ref(), &mut rng).await?;
    let authorize_url = pending.authorize_url().to_string();
    let flow = engine.config.ids.next_id();
    engine
        .ai_key_logins
        .register_session(flow.clone(), connection.id.clone(), pending, host_id);
    Ok(Response::Ai(AiResponse::KeyLoginStarted {
        flow,
        authorize_url,
    }))
}

/// Identifiant d'hôte `urn:uuid:` aléatoire, sans donnée personnelle
fn new_host_id(rng: &mut StdRng) -> String {
    let mut bytes = [0_u8; 16];
    rng.fill_bytes(&mut bytes);
    let id = uuid::Builder::from_random_bytes(bytes).into_uuid();
    format!("urn:uuid:{id}")
}
