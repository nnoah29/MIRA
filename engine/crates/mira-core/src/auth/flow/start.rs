//! Démarrage d'un flux : PKCE, état aléatoire, écouteur de boucle locale et URL d'autorisation.

use mira_vocabulary::error::codes::network;
use mira_vocabulary::{ErrorCategory, MiraError, OAuthSettings};
use rand::RngCore;
use tokio::net::TcpListener;

use super::pending::PendingFlow;
use super::phase::FlowPhase;
use crate::auth::form::build_query;
use crate::auth::loopback::require_secure_url;
use crate::auth::pkce::{challenge_for, generate_verifier, random_token};

/// Prépare un flux : ouvre l'écouteur sur `127.0.0.1:0` et construit l'URL d'autorisation
pub async fn start_flow(
    settings: &OAuthSettings,
    client_id: &str,
    login_hint: &str,
    rng: &mut (dyn RngCore + Send),
) -> Result<PendingFlow, MiraError> {
    require_secure_url(&settings.authorize_url)?;
    require_secure_url(&settings.token_url)?;
    let verifier = generate_verifier(rng);
    let state = random_token(rng, 24);
    let listener = bind_loopback().await?;
    let port = listener.local_addr().map_or(0, |address| address.port());
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let challenge = challenge_for(verifier.expose());
    let params = [client_id, login_hint, &redirect_uri, &state, &challenge];
    Ok(PendingFlow {
        authorize_url: authorization_url(settings, params),
        redirect_uri,
        state,
        verifier,
        client_id: client_id.to_string(),
        token_url: settings.token_url.clone(),
        listener: Some(listener),
        phase: FlowPhase::AwaitingRedirect,
    })
}

/// Ouvre l'écouteur de redirection, uniquement sur l'interface de boucle locale
async fn bind_loopback() -> Result<TcpListener, MiraError> {
    TcpListener::bind("127.0.0.1:0").await.map_err(|_| {
        MiraError::new(
            ErrorCategory::Network,
            network::OFFLINE,
            "écouteur de redirection local impossible",
        )
    })
}

/// URL d'autorisation ; `params` = client, indice de connexion, redirection, état, défi PKCE
fn authorization_url(settings: &OAuthSettings, params: [&str; 5]) -> String {
    let [client_id, login_hint, redirect_uri, state, challenge] = params;
    let scope = settings.scopes.join(" ");
    let mut pairs = vec![
        ("response_type", "code"),
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("state", state),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
    ];
    if !scope.is_empty() {
        pairs.push(("scope", &scope));
    }
    if !login_hint.is_empty() {
        pairs.push(("login_hint", login_hint));
    }
    let base = &settings.authorize_url;
    let separator = if base.contains('?') { '&' } else { '?' };
    format!("{base}{separator}{}", build_query(&pairs))
}
