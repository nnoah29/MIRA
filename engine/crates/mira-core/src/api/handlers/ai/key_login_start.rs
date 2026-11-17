//! Début d'une connexion par navigateur : écouteur local ouvert, adresse d'autorisation rendue.

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{AiConnectionId, AiKeyLogin, ErrorCategory, MiraError};
use rand::SeedableRng;
use rand::rngs::StdRng;

use super::connection_view::read_connection;
use super::session_login_start::start_session_login;
use crate::ai::access::find_provider;
use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;
use crate::auth::start_key_flow;

const KEY_LABEL: &str = "MIRA";

/// Ouvre le flux pour la connexion, l'enregistre et rend son identifiant et l'adresse à ouvrir
pub(crate) async fn handle_start_ai_key_login(
    engine: &Engine,
    id: &AiConnectionId,
) -> Result<Response, MiraError> {
    let connection = read_connection(engine, id)?;
    let descriptor = find_provider(engine, &connection)?.descriptor();
    if let Some(session_login) = &descriptor.session_login {
        return start_session_login(engine, &connection, session_login).await;
    }
    let login = key_login_of(engine, &connection)?;
    let mut rng = StdRng::from_os_rng();
    let pending = start_key_flow(&login, KEY_LABEL, &mut rng).await?;
    let authorize_url = pending.authorize_url().to_string();
    let flow = engine.config.ids.next_id();
    engine
        .ai
        .key_logins
        .register(flow.clone(), id.clone(), pending);
    Ok(Response::Ai(AiResponse::KeyLoginStarted {
        flow,
        authorize_url,
    }))
}

/// Adresses de connexion du fournisseur, sinon `incompatible_state.ai_key_login_unsupported`
fn key_login_of(
    engine: &Engine,
    connection: &mira_vocabulary::AiConnection,
) -> Result<AiKeyLogin, MiraError> {
    find_provider(engine, connection)?
        .descriptor()
        .key_login
        .ok_or_else(|| {
            MiraError::new(
                ErrorCategory::IncompatibleState,
                incompatible_state::AI_KEY_LOGIN_UNSUPPORTED,
                "ce service d'IA ne propose pas la connexion par navigateur",
            )
        })
}
