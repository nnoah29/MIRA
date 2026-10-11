//! Simulation du navigateur et de l'utilisateur : approbation ou refus.

use std::io;

use super::authorize::AuthorizeRequest;
use super::redirect::deliver;
use super::server::FakeOAuthServer;
use super::state::PendingCode;

impl FakeOAuthServer {
    /// Approuve la requête d'autorisation pour cet utilisateur et redirige avec `code` et `state`
    pub async fn approve(&self, authorize_url: &str, user: &str) -> io::Result<()> {
        let request = AuthorizeRequest::parse(authorize_url)?;
        let pending = PendingCode {
            user: user.to_string(),
            client_id: request.client_id.clone(),
            redirect_uri: request.redirect_uri.clone(),
            challenge: request.code_challenge.clone(),
        };
        let code = self.context.shared.with(|state| state.store_code(pending));
        let params = request.callback_params("code", &code);
        deliver(&request.redirect_uri, &params).await
    }

    /// Refuse la requête d'autorisation et redirige avec `error=access_denied` et `state`
    pub async fn deny(&self, authorize_url: &str) -> io::Result<()> {
        let request = AuthorizeRequest::parse(authorize_url)?;
        let params = request.callback_params("error", "access_denied");
        deliver(&request.redirect_uri, &params).await
    }
}
