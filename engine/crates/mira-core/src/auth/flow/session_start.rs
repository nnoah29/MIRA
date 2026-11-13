//! Démarrage d'une connexion qui rend une session : inscription ou reconnexion, PKCE, état, nonce et écouteur local.

use mira_vocabulary::{AiSessionLogin, MiraError};
use rand::RngCore;
use tokio::net::TcpListener;

use super::pending::PendingFlow;
use super::phase::FlowPhase;
use super::start::bind_loopback;
use crate::auth::form::build_query;
use crate::auth::loopback::require_secure_url;
use crate::auth::pkce::{challenge_for, generate_verifier, random_token};
use crate::auth::session::AiSession;

const REGISTRATION_CLIENT: &str = "dynamic_agent_client";
const PREFERRED_PORT: u16 = 1455;
const CALLBACK_PATH: &str = "/auth/callback";

/// Prépare le flux ; sans session connue c'est une inscription (client dynamique), sinon une reconnexion
pub async fn start_session_flow(
    login: &AiSessionLogin,
    host_id: &str,
    previous: Option<&AiSession>,
    rng: &mut (dyn RngCore + Send),
) -> Result<PendingFlow, MiraError> {
    require_secure_url(&login.authorize_url)?;
    require_secure_url(&login.token_url)?;
    let verifier = generate_verifier(rng);
    let (state, nonce) = (random_token(rng, 24), random_token(rng, 24));
    let listener = listener_for(previous).await?;
    let port = listener.local_addr().map_or(0, |address| address.port());
    let redirect_uri = format!("http://127.0.0.1:{port}{CALLBACK_PATH}");
    let challenge = challenge_for(verifier.expose());
    let params = [redirect_uri.as_str(), &challenge, &state, &nonce, host_id];
    Ok(PendingFlow {
        authorize_url: authorization_url(login, previous, params),
        redirect_uri,
        state,
        verifier,
        client_id: previous.map_or_else(String::new, |session| session.client_id.clone()),
        client_secret: None,
        token_url: login.token_url.clone(),
        listener: Some(listener),
        phase: FlowPhase::AwaitingRedirect,
        nonce: Some(nonce),
        callback_path: CALLBACK_PATH,
    })
}

/// Écouteur local : l'inscription préfère le port 1455 et se replie sur un port libre, la reconnexion prend un port libre
async fn listener_for(previous: Option<&AiSession>) -> Result<TcpListener, MiraError> {
    if previous.is_none()
        && let Ok(listener) = TcpListener::bind(("127.0.0.1", PREFERRED_PORT)).await
    {
        return Ok(listener);
    }
    bind_loopback().await
}

/// Adresse d'autorisation ; `params` = redirection, défi PKCE, état, nonce, identifiant d'hôte
fn authorization_url(
    login: &AiSessionLogin,
    previous: Option<&AiSession>,
    params: [&str; 5],
) -> String {
    let [redirect_uri, challenge, state, nonce, host_id] = params;
    let scope = login.scopes.join(" ");
    let client = previous.map_or(REGISTRATION_CLIENT, |session| session.client_id.as_str());
    let mut pairs = vec![
        ("response_type", "code"),
        ("client_id", client),
        ("redirect_uri", redirect_uri),
        ("scope", scope.as_str()),
        ("resource", login.resource.as_str()),
        ("state", state),
        ("nonce", nonce),
        ("code_challenge_method", "S256"),
        ("code_challenge", challenge),
        ("ext_agent_host_id", host_id),
    ];
    pairs.extend(identity_hint(login, previous));
    let separator = if login.authorize_url.contains('?') {
        '&'
    } else {
        '?'
    };
    format!("{}{separator}{}", login.authorize_url, build_query(&pairs))
}

/// Indice d'identité : le courriel connu en reconnexion, le nom de l'application à l'inscription
fn identity_hint<'a>(
    login: &'a AiSessionLogin,
    previous: Option<&'a AiSession>,
) -> Option<(&'static str, &'a str)> {
    match previous {
        None => Some(("agent_name_hint", login.agent_name.as_str())),
        Some(session) => session.email.as_deref().map(|email| ("login_hint", email)),
    }
}
