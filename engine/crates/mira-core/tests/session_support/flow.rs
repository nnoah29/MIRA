//! Parcours d'une connexion de session jusqu'au code reçu, avec la redirection que ferait le service.

use std::time::Duration;

use mira_core::auth::session::AiSession;
use mira_core::auth::{AuthorizationCode, PendingFlow, start_session_flow, wait_for_redirect};
use mira_vocabulary::{AiSessionLogin, MiraError};
use rand::SeedableRng;
use rand::rngs::StdRng;

use super::browser::redirect;

pub const HOST: &str = "urn:uuid:11111111-2222-3333-4444-555555555555";
pub const ISSUER: &str = "https://auth.provider.test";

/// Fiche de connexion de test dont le point de jeton est le faux serveur
pub fn login(token_url: &str) -> AiSessionLogin {
    AiSessionLogin {
        authorize_url: format!("{ISSUER}/api/accounts/authorize"),
        token_url: token_url.to_string(),
        resource: "https://api.provider.test/v1".to_string(),
        scopes: ["openid", "profile", "scope.required"]
            .map(String::from)
            .to_vec(),
        required_scope: "scope.required".to_string(),
        agent_name: "MIRA".to_string(),
    }
}

/// Démarre le flux
pub async fn start(login: &AiSessionLogin, previous: Option<&AiSession>) -> PendingFlow {
    let mut rng = StdRng::seed_from_u64(7);
    start_session_flow(login, HOST, previous, &mut rng)
        .await
        .unwrap()
}

/// Le navigateur rappelle la redirection avec cette requête (sans `state` : celui du flux est ajouté)
pub async fn receive(
    flow: &mut PendingFlow,
    state: &str,
    extra: &str,
) -> Result<AuthorizationCode, MiraError> {
    let uri = flow.redirect_uri().to_string();
    let query = format!("state={state}&{extra}");
    let hit = tokio::spawn(async move { redirect(&uri, &query).await });
    let code = wait_for_redirect(flow, Duration::from_secs(5)).await;
    let _ = hit.await;
    code
}
