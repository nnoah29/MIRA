//! Banc des scénarios S8 à S14 : un faux serveur IMAP et SMTP pour plusieurs utilisateurs, profil partagé.

use std::sync::Arc;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::mira_vocabulary::{Security, ServerEndpoint, ServerSettings, Timestamp};
use mira_assembly::ports::{Clock, IdGenerator, ManualClock, MemorySecretStore, SequentialIds};
use mira_cli::CliDeps;
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::oauth::FakeOAuthServer;
use mira_testkit::smtp::FakeSmtpServer;
use tempfile::TempDir;

use super::sb_oauth::{OAUTH_USER, register_oauth};
use super::sync_bench::quick_sync;
use super::test_deps;

pub const SB_PASSWORD: &str = "sb-SECRET-pw-7731";
pub const START_MILLIS: i64 = 1_700_000_000_000;

/// Profil jetable, trousseau, horloge et faux serveurs partagés par toutes les commandes d'un scénario.
pub struct SbWorld {
    pub home: TempDir,
    pub secrets: Arc<MemorySecretStore>,
    pub clock: Arc<ManualClock>,
    pub ids: Arc<SequentialIds>,
    pub credentials: Credentials,
    pub store: FakeMailStore,
    pub imap: FakeImapServer,
    pub smtp: FakeSmtpServer,
    pub oauth: FakeOAuthServer,
}

/// Point d'accès en clair sur la boucle locale.
fn endpoint(port: u16) -> ServerEndpoint {
    ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    }
}

impl SbWorld {
    /// Démarre les faux serveurs ; chaque utilisateur reçoit `SB_PASSWORD` et les boîtes Sent, Archive, Trash.
    pub async fn start(users: &[&str], capabilities: CapabilitySet) -> Self {
        let credentials = Credentials::default();
        let store = FakeMailStore::new();
        for user in users {
            credentials.add_password(user, SB_PASSWORD);
        }
        for user in users.iter().chain(&[OAUTH_USER]) {
            for name in ["Sent", "Archive", "Trash"] {
                store.create_mailbox(user, name, None);
            }
        }
        let imap = FakeImapServer::start(store.clone(), credentials.clone(), capabilities)
            .await
            .unwrap();
        let smtp = FakeSmtpServer::start(credentials.clone()).await.unwrap();
        let oauth = FakeOAuthServer::start(credentials.clone()).await.unwrap();
        Self {
            home: tempfile::tempdir().unwrap(),
            secrets: Arc::new(MemorySecretStore::new()),
            clock: Arc::new(ManualClock::new(Timestamp::from_millis(START_MILLIS))),
            ids: Arc::new(SequentialIds::new()),
            credentials,
            store,
            imap,
            smtp,
            oauth,
        }
    }

    /// Dépendances du CLI : les domaines de test visent les faux serveurs, horloge manuelle partagée.
    pub fn deps(&self, stdin: &str, background: bool) -> CliDeps {
        let mut network = NetworkPolicy::default();
        let servers = ServerSettings {
            imap: endpoint(self.imap.port()),
            smtp: endpoint(self.smtp.port()),
        };
        for domain in ["folders.test", "gmail.com", "oauth.test"] {
            let known = servers.clone();
            network.server_overrides.insert(domain.to_string(), known);
        }
        if background {
            network.background_sync = quick_sync();
        }
        let mut deps = test_deps(self.home.path(), Arc::clone(&self.secrets), network, stdin);
        deps.clock = Arc::clone(&self.clock) as Arc<dyn Clock>;
        deps.ids = Arc::clone(&self.ids) as Arc<dyn IdGenerator>;
        register_oauth(&mut deps, &self.oauth);
        deps
    }

    /// Nombre de secrets rangés au trousseau du profil.
    pub fn secret_count(&self) -> usize {
        self.secrets.entry_count()
    }
}
