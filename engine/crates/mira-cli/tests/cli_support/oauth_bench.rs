//! Banc OAuth : faux serveurs IMAP et d'autorisation, faux provider `oauth.test`, navigateur simulé.

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::mira_vocabulary::{
    AccountId, OAuthSettings, ProfileId, ProviderId, SecretKind, SecretRef,
};
use mira_assembly::ports::{Clock, MemorySecretStore, SecretStore, SystemClock};
use mira_cli::{CliDeps, CliIo, run};
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::oauth::FakeOAuthServer;
use mira_testkit::smtp::FakeSmtpServer;
use tempfile::TempDir;

use super::fake_oauth_provider::{FakeOAuthProvider, OAUTH_PROVIDER_ID};
use super::shared_buffer::SharedBuffer;
use super::sync_bench::quick_sync;
use super::{CliResult, test_deps};

pub const OAUTH_USER: &str = "dave@oauth.test";
pub const CLIENT_ID: &str = "mira-test-client";

/// Profil jetable, trousseau, faux serveur IMAP et faux serveur d'autorisation OAuth2.
pub struct OauthBench {
    pub home: TempDir,
    pub secrets: Arc<MemorySecretStore>,
    pub store: FakeMailStore,
    pub imap: FakeImapServer,
    pub smtp: FakeSmtpServer,
    pub oauth: FakeOAuthServer,
}

impl OauthBench {
    /// Démarre les deux faux serveurs sur la boucle locale.
    pub async fn start() -> Self {
        let credentials = Credentials::default();
        let store = FakeMailStore::new();
        let imap = FakeImapServer::start(store.clone(), credentials.clone(), CapabilitySet::full())
            .await
            .unwrap();
        let smtp = FakeSmtpServer::start(credentials.clone()).await.unwrap();
        let oauth = FakeOAuthServer::start(credentials).await.unwrap();
        Self {
            home: tempfile::tempdir().unwrap(),
            secrets: Arc::new(MemorySecretStore::new()),
            store,
            imap,
            smtp,
            oauth,
        }
    }

    /// Dépendances du CLI : faux provider, adresses OAuth du faux serveur, synchro permise.
    pub fn deps(&self) -> CliDeps {
        let mut network = NetworkPolicy::default();
        let servers = super::server::local_servers(self.imap.port(), self.smtp.port());
        network
            .server_overrides
            .insert("oauth.test".to_string(), servers);
        let mut deps = test_deps(self.home.path(), Arc::clone(&self.secrets), network, "");
        deps.registry.register(Arc::new(FakeOAuthProvider));
        deps.clock = Arc::new(SystemClock) as Arc<dyn Clock>;
        deps.network.background_sync = quick_sync();
        let provider = ProviderId::new(OAUTH_PROVIDER_ID);
        let settings = OAuthSettings {
            authorize_url: self.oauth.authorize_url(),
            token_url: self.oauth.token_url(),
            scopes: vec!["mail".to_string()],
        };
        deps.oauth
            .endpoint_overrides
            .insert(provider.clone(), settings);
        deps.oauth
            .client_ids
            .insert(provider, CLIENT_ID.to_string());
        deps
    }
}

impl OauthBench {
    /// Jeton rangé au trousseau pour ce compte, s'il y en a un.
    pub fn stored(&self, account: &str, kind: SecretKind) -> Option<String> {
        let key = SecretRef {
            profile: ProfileId::from_raw("default"),
            account: AccountId::from_raw(account),
            kind,
        };
        let value = self.secrets.get(&key).unwrap();
        value.map(|secret| secret.expose().to_string())
    }

    /// Lance `account add-oauth` pendant que le navigateur simulé répond à l'adresse annoncée.
    pub async fn login(&self, extra: &[&str], approve: bool) -> (CliResult, String) {
        let err = SharedBuffer::default();
        let mut deps = self.deps();
        let mut args = vec!["account", "add-oauth", "--address", OAUTH_USER, "--json"];
        args.extend_from_slice(extra);
        let session = async {
            let browser = answer_when_announced(&err, self, approve);
            tokio::join!(run_watching(&mut deps, &args, err.clone()), browser)
        };
        let limit = Duration::from_secs(60);
        tokio::time::timeout(limit, session).await.unwrap()
    }
}

/// Lance `mira` avec une sortie d'erreur partagée que le test peut lire pendant l'exécution.
pub async fn run_watching(deps: &mut CliDeps, arguments: &[&str], err: SharedBuffer) -> CliResult {
    let mut args = vec!["mira".to_string()];
    args.extend(arguments.iter().map(ToString::to_string));
    let mut out = Vec::new();
    let mut err_writer = err.clone();
    let code = run(args, &mut CliIo::new(&mut out, &mut err_writer), deps).await;
    CliResult {
        code,
        stdout: String::from_utf8_lossy(&out).into_owned(),
        stderr: err.text(),
    }
}

/// Attend l'adresse d'autorisation sur stderr, puis simule la réponse de l'utilisateur ; rend l'adresse.
pub async fn answer_when_announced(
    err: &SharedBuffer,
    bench: &OauthBench,
    approve: bool,
) -> String {
    let url = loop {
        let text = err.text();
        if let Some(line) = text.lines().find(|line| line.starts_with("http")) {
            break line.to_string();
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    if approve {
        bench.oauth.approve(&url, OAUTH_USER).await.unwrap();
    } else {
        bench.oauth.deny(&url).await.unwrap();
    }
    url
}
