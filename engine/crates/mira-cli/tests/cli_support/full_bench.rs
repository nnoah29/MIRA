//! Banc complet : faux serveurs IMAP et SMTP, horloge manuelle et identifiants partagés entre commandes.

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::mira_vocabulary::{Security, ServerEndpoint, ServerSettings, Timestamp};
use mira_assembly::ports::{Clock, IdGenerator, ManualClock, MemorySecretStore, SequentialIds};
use mira_cli::CliDeps;
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::smtp::FakeSmtpServer;
use serde_json::Value;
use tempfile::TempDir;

use super::{CliResult, parse_json, run_with, test_deps};

pub const USER: &str = "alice@folders.test";
pub const PASSWORD: &str = "pa55-SECRET-xyz";
pub const START_MILLIS: i64 = 1_700_000_000_000;

/// Profil jetable, trousseau, horloge et identifiants partagés par toutes les commandes d'un test.
pub struct FullBench {
    pub home: TempDir,
    pub secrets: Arc<MemorySecretStore>,
    pub clock: Arc<ManualClock>,
    pub ids: Arc<SequentialIds>,
    pub store: FakeMailStore,
    pub imap: FakeImapServer,
    pub smtp: FakeSmtpServer,
}

/// Point d'accès en clair sur la boucle locale.
fn endpoint(port: u16) -> ServerEndpoint {
    ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    }
}

impl FullBench {
    /// Démarre les faux serveurs pour l'utilisateur de test, avec les boîtes Sent, Archive, Trash.
    pub async fn start() -> Self {
        let credentials = Credentials::default();
        credentials.add_password(USER, PASSWORD);
        let store = FakeMailStore::new();
        for name in ["Sent", "Archive", "Trash"] {
            store.create_mailbox(USER, name, None);
        }
        let imap = FakeImapServer::start(store.clone(), credentials.clone(), CapabilitySet::full())
            .await
            .unwrap();
        let smtp = FakeSmtpServer::start(credentials).await.unwrap();
        Self {
            home: tempfile::tempdir().unwrap(),
            secrets: Arc::new(MemorySecretStore::new()),
            clock: Arc::new(ManualClock::new(Timestamp::from_millis(START_MILLIS))),
            ids: Arc::new(SequentialIds::new()),
            store,
            imap,
            smtp,
        }
    }

    /// Dépendances du CLI : `folders.test` vise les faux serveurs, horloge et identifiants partagés.
    pub fn deps(&self, stdin: &str) -> CliDeps {
        let mut network = NetworkPolicy::default();
        let servers = ServerSettings {
            imap: endpoint(self.imap.port()),
            smtp: endpoint(self.smtp.port()),
        };
        network
            .server_overrides
            .insert("folders.test".to_string(), servers);
        let mut deps = test_deps(self.home.path(), Arc::clone(&self.secrets), network, stdin);
        deps.clock = Arc::clone(&self.clock) as Arc<dyn Clock>;
        deps.ids = Arc::clone(&self.ids) as Arc<dyn IdGenerator>;
        deps
    }

    /// Lance une commande (au plus 60 secondes) avec une entrée standard vide.
    pub async fn run(&self, arguments: &[&str]) -> CliResult {
        let mut deps = self.deps("");
        let limit = Duration::from_secs(60);
        tokio::time::timeout(limit, run_with(&mut deps, arguments))
            .await
            .unwrap()
    }

    /// Lance une commande avec `--json`, exige le code 0 et rend la réponse du contrat.
    pub async fn run_ok(&self, arguments: &[&str]) -> Value {
        let mut full: Vec<&str> = arguments.to_vec();
        full.push("--json");
        let result = self.run(&full).await;
        assert_eq!(result.code, 0, "{arguments:?} : {}", result.stderr);
        parse_json(&result)["data"].clone()
    }
}
