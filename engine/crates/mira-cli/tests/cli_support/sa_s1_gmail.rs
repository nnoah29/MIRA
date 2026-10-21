//! Décor du scénario S1 : vrai provider Gmail redirigé vers le faux IMAP et le faux serveur d'autorisation.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use mira_assembly::mira_vocabulary::{OAuthSettings, ProviderId};
use mira_assembly::ports::{Clock, MemorySecretStore, SystemClock};
use mira_cli::CliDeps;
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::oauth::FakeOAuthServer;
use mira_testkit::smtp::FakeSmtpServer;
use tempfile::TempDir;

use super::sync_bench::quick_sync;
use super::{CliResult, run_with, test_deps};

pub const GMAIL_USER: &str = "dave@gmail.com";
pub const CLIENT_ID: &str = "mira-gmail-test-client";

/// Profil jetable, faux IMAP (extensions Gmail), faux serveur d'autorisation et journal des sorties.
pub struct GmailWorld {
    pub home: TempDir,
    pub secrets: Arc<MemorySecretStore>,
    pub store: FakeMailStore,
    pub imap: FakeImapServer,
    pub smtp: FakeSmtpServer,
    pub oauth: FakeOAuthServer,
    pub profile: &'static str,
    pub transcript: Mutex<Vec<String>>,
}

impl GmailWorld {
    /// Démarre le faux IMAP à extensions Gmail et le faux serveur d'autorisation, pour le profil donné.
    pub async fn start(profile: &'static str) -> Self {
        let credentials = Credentials::default();
        let store = FakeMailStore::new();
        let imap =
            FakeImapServer::start(store.clone(), credentials.clone(), CapabilitySet::gmail())
                .await
                .unwrap();
        let smtp = FakeSmtpServer::start(credentials.clone()).await.unwrap();
        let oauth = FakeOAuthServer::start(credentials).await.unwrap();
        let home = tempfile::tempdir().unwrap();
        let secrets = Arc::new(MemorySecretStore::new());
        let transcript = Mutex::new(Vec::new());
        Self {
            home,
            secrets,
            store,
            imap,
            smtp,
            oauth,
            profile,
            transcript,
        }
    }

    /// Dépendances du CLI : `gmail.com` vise le faux IMAP, les adresses OAuth de Gmail visent le faux serveur.
    pub fn deps(&self) -> CliDeps {
        let mut network = mira_assembly::accounts::NetworkPolicy::default();
        let servers = super::server::local_servers(self.imap.port(), self.smtp.port());
        network
            .server_overrides
            .insert("gmail.com".to_string(), servers);
        let mut deps = test_deps(self.home.path(), Arc::clone(&self.secrets), network, "");
        deps.clock = Arc::new(SystemClock) as Arc<dyn Clock>;
        deps.network.background_sync = quick_sync();
        let provider = ProviderId::new("gmail");
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

    /// Lance une commande (au plus 60 secondes) sur le profil du monde et garde ses sorties dans le journal.
    pub async fn run(&self, arguments: &[&str]) -> CliResult {
        let mut deps = self.deps();
        let line = [arguments, &["--profile", self.profile]].concat();
        let limit = Duration::from_secs(60);
        let result = tokio::time::timeout(limit, run_with(&mut deps, &line))
            .await
            .unwrap();
        let mut log = self.transcript.lock().unwrap();
        log.extend([result.stdout.clone(), result.stderr.clone()]);
        result
    }
}
