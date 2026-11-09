//! Connexion de session de bout en bout contre le faux point de jeton.

use mira_core::auth::session::AiSession;
use mira_core::auth::{PendingFlow, complete_session_flow};
use mira_vocabulary::{AiSessionLogin, MiraError};

use super::browser::query_of;
use super::flow::{HOST, login, receive, start};
use super::id_token::{Claims, make};
use super::token_server::TokenServer;

pub const NOW: i64 = 1_000_000;

/// Corps de succès : l'identité porte ces réclamations, la portée accordée est `scope`
pub fn grant(nonce: &str, client: &str, iss: &str, scope: &str) -> String {
    let claims = Claims {
        iss,
        aud: client,
        nonce,
        exp: NOW + 3600,
        iat: NOW,
    };
    let id = make(&claims);
    format!(
        r#"{{"access_token":"AT-1","refresh_token":"RT-1","id_token":"{id}","token_type":"Bearer","expires_in":3600,"scope":"{scope}"}}"#
    )
}

/// Parcourt la connexion : redirection avec `extra`, puis échange avec la réponse du serveur
pub async fn connect(
    server: &TokenServer,
    previous: Option<&AiSession>,
    extra: &str,
    reply: &dyn Fn(&str) -> String,
) -> Result<AiSession, MiraError> {
    let login: AiSessionLogin = login(&server.url);
    let mut flow: PendingFlow = start(&login, previous).await;
    let query = query_of(flow.authorize_url());
    server.push(200, &reply(&query["nonce"]));
    let code = receive(&mut flow, &query["state"], extra).await?;
    complete_session_flow(&mut flow, &login, &code, previous, HOST, NOW).await
}
