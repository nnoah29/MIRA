//! OAuth2 sur le banc des scénarios : faux provider `oauth.test`, navigateur simulé, jetons du trousseau.

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::mira_vocabulary::{
    AccountId, OAuthSettings, ProfileId, ProviderId, SecretKind, SecretRef,
};
use mira_assembly::ports::SecretStore;
use mira_cli::CliDeps;
use mira_testkit::oauth::FakeOAuthServer;

use super::CliResult;
use super::fake_oauth_provider::{FakeOAuthProvider, OAUTH_PROVIDER_ID};
use super::oauth_bench::run_watching;
use super::sb_world::SbWorld;
use super::shared_buffer::SharedBuffer;

pub const OAUTH_USER: &str = "dave@oauth.test";
const CLIENT_ID: &str = "mira-test-client";

/// Déclare le faux provider OAuth2 et les adresses du faux serveur d'autorisation.
pub fn register_oauth(deps: &mut CliDeps, oauth: &FakeOAuthServer) {
    deps.registry.register(Arc::new(FakeOAuthProvider));
    let provider = ProviderId::new(OAUTH_PROVIDER_ID);
    let settings = OAuthSettings {
        authorize_url: oauth.authorize_url(),
        token_url: oauth.token_url(),
        scopes: vec!["mail".to_string()],
    };
    deps.oauth
        .endpoint_overrides
        .insert(provider.clone(), settings);
    deps.oauth
        .client_ids
        .insert(provider, CLIENT_ID.to_string());
}

/// Attend l'adresse d'autorisation sur stderr, puis simule la réponse de l'utilisateur.
async fn answer_when_announced(world: &SbWorld, err: &SharedBuffer, approve: bool) {
    let url = loop {
        let text = err.text();
        if let Some(line) = text.lines().find(|line| line.starts_with("http")) {
            break line.to_string();
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    if approve {
        world.oauth.approve(&url, OAUTH_USER).await.unwrap();
    } else {
        world.oauth.deny(&url).await.unwrap();
    }
}

impl SbWorld {
    /// Ajoute le compte OAuth2 de test par `account add-oauth` ; le navigateur simulé répond à l'adresse annoncée.
    pub async fn oauth_login(&self, extra: &[&str]) -> CliResult {
        let err = SharedBuffer::default();
        let mut deps = self.deps("", true);
        let mut args = vec!["account", "add-oauth", "--address", OAUTH_USER, "--json"];
        args.extend_from_slice(extra);
        let session = async {
            let browser = answer_when_announced(self, &err, true);
            tokio::join!(run_watching(&mut deps, &args, err.clone()), browser)
        };
        let limit = Duration::from_secs(60);
        tokio::time::timeout(limit, session).await.unwrap().0
    }

    /// Jeton rangé au trousseau pour ce compte, s'il y en a un.
    pub fn stored_token(&self, account: &str, kind: SecretKind) -> Option<String> {
        let key = SecretRef {
            profile: ProfileId::from_raw("default"),
            account: AccountId::from_raw(account),
            kind,
        };
        let value = self.secrets.get(&key).unwrap();
        value.map(|secret| secret.expose().to_string())
    }
}
