//! Démarrage d'une connexion par navigateur qui rend une clé d'API : PKCE, état et écouteur de boucle locale.

use mira_vocabulary::{AiKeyLogin, MiraError};
use rand::RngCore;

use super::pending::PendingFlow;
use super::phase::FlowPhase;
use super::start::bind_loopback;
use crate::auth::form::build_query;
use crate::auth::loopback::require_secure_url;
use crate::auth::pkce::{challenge_for, generate_verifier, random_token};

/// Prépare le flux : écouteur sur `127.0.0.1:0` et adresse d'autorisation avec défi S256, état et libellé de clé
pub async fn start_key_flow(
    login: &AiKeyLogin,
    key_label: &str,
    rng: &mut (dyn RngCore + Send),
) -> Result<PendingFlow, MiraError> {
    require_secure_url(&login.authorize_url)?;
    require_secure_url(&login.exchange_url)?;
    let verifier = generate_verifier(rng);
    let state = random_token(rng, 24);
    let listener = bind_loopback().await?;
    let port = listener.local_addr().map_or(0, |address| address.port());
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let challenge = challenge_for(verifier.expose());
    let params = [redirect_uri.as_str(), &challenge, &state, key_label];
    Ok(PendingFlow {
        authorize_url: key_authorization_url(&login.authorize_url, params),
        redirect_uri,
        state,
        verifier,
        client_id: String::new(),
        client_secret: None,
        token_url: login.exchange_url.clone(),
        listener: Some(listener),
        phase: FlowPhase::AwaitingRedirect,
    })
}

/// Adresse d'autorisation ; `params` = redirection, défi PKCE, état, libellé de la clé
fn key_authorization_url(base: &str, params: [&str; 4]) -> String {
    let [callback_url, challenge, state, key_label] = params;
    let pairs = [
        ("callback_url", callback_url),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
        ("state", state),
        ("key_label", key_label),
    ];
    let separator = if base.contains('?') { '&' } else { '?' };
    format!("{base}{separator}{}", build_query(&pairs))
}
