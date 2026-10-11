//! Parcours OAuth des tests : flux démarré, navigateur simulé qui approuve, réauthentification.

use mira_core::api::contract::OAuthStart;
use mira_core::api::{AuthRequest, Engine, EngineConfig, Request, Response};
use mira_testkit::oauth::FakeOAuthServer;
use mira_vocabulary::{Account, AccountId, OAuthSettings, ProviderId};

use super::fixture::Fixture;
use super::oauth_provider::OAUTH_PROVIDER_ID;
use super::oauth_requests::{add_oauth_request, complete_request};

pub const OAUTH_USER: &str = "dave@oauth.test";
pub const CLIENT_ID: &str = "mira-test-client";

/// Fait pointer la configuration OAuth du faux provider vers le faux serveur d'autorisation
pub fn point_oauth_at(config: &mut EngineConfig, server: &FakeOAuthServer) {
    let provider = ProviderId::new(OAUTH_PROVIDER_ID);
    let settings = OAuthSettings {
        authorize_url: server.authorize_url(),
        token_url: server.token_url(),
        scopes: vec!["mail".to_string()],
    };
    config
        .oauth
        .endpoint_overrides
        .insert(provider.clone(), settings);
    config
        .oauth
        .client_ids
        .insert(provider, CLIENT_ID.to_string());
}

/// Démarre un flux OAuth pour l'adresse et rend son identifiant et son adresse d'autorisation
pub async fn start_oauth(engine: &Engine, address: &str) -> OAuthStart {
    let request = Request::Auth(AuthRequest::StartOAuth {
        address: address.to_string(),
        provider: None,
    });
    match engine.execute(request).await.unwrap() {
        Response::OAuthStarted(started) => started,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Démarre un flux, simule l'approbation de l'utilisateur et termine le flux ; rend son identifiant
pub async fn authorize(f: &Fixture, address: &str) -> String {
    let started = start_oauth(&f.engine, address).await;
    finish_with_approval(f, address, started).await
}

/// Autorise puis ajoute un compte OAuth2 pour l'utilisateur de test
pub async fn add_oauth_account(f: &Fixture, key: &str) -> Account {
    let flow = authorize(f, OAUTH_USER).await;
    match f
        .engine
        .execute(add_oauth_request(&flow, key))
        .await
        .unwrap()
    {
        Response::Account(account) => account,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Réauthentifie un compte OAuth2 de bout en bout et rend l'identifiant du flux utilisé
pub async fn reauthenticate(f: &Fixture, account: &AccountId) -> String {
    let request = Request::Auth(AuthRequest::Reauthenticate {
        account: account.clone(),
    });
    let Response::OAuthStarted(started) = f.engine.execute(request).await.unwrap() else {
        panic!("flux attendu");
    };
    finish_with_approval(f, OAUTH_USER, started).await
}

/// Approuve la demande dans le navigateur simulé pendant que `CompleteOAuth` attend la redirection
async fn finish_with_approval(f: &Fixture, user: &str, started: OAuthStart) -> String {
    let complete = complete_request(&started.flow);
    let (approved, completed) = tokio::join!(
        f.oauth.approve(&started.authorize_url, user),
        f.engine.execute(complete)
    );
    approved.unwrap();
    assert_eq!(completed.unwrap(), Response::Done);
    started.flow
}
