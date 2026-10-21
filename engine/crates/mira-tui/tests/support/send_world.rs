//! Monde d'envoi : faux serveurs IMAP et SMTP sur la boucle locale, compte de test, horloge manuelle gardée.

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::mira_vocabulary::{Security, ServerEndpoint, ServerSettings, Timestamp};
use mira_assembly::ports::{Clock, ManualClock};
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::smtp::FakeSmtpServer;
use mira_tui::app::open_engine;

use super::config::test_config;
use super::imap_account::{PASSWORD, USER, add_test_account};
use super::world::TestWorld;

const SEND_DELAY_PASSED: i64 = 10_001;

/// Monde de test, faux SMTP qui reçoit les envois, horloge du moteur.
pub struct SendWorld {
    pub world: TestWorld,
    pub smtp: FakeSmtpServer,
    pub clock: Arc<ManualClock>,
}

/// Réseau de test : le domaine `folders.test` vise les deux faux serveurs, en clair.
fn network_for(imap_port: u16, smtp_port: u16) -> NetworkPolicy {
    let endpoint = |port: u16| ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    };
    let settings = ServerSettings {
        imap: endpoint(imap_port),
        smtp: endpoint(smtp_port),
    };
    let mut network = NetworkPolicy::default();
    network
        .server_overrides
        .insert("folders.test".to_string(), settings);
    network
}

impl SendWorld {
    /// Démarre les faux serveurs, ouvre le moteur avec l'horloge du test et ajoute `alice@folders.test`.
    pub async fn start() -> SendWorld {
        let credentials = Credentials::default();
        credentials.add_password(USER, PASSWORD);
        let store = FakeMailStore::new();
        let caps = CapabilitySet::full();
        let imap = FakeImapServer::start(store, credentials.clone(), caps)
            .await
            .unwrap();
        let smtp = FakeSmtpServer::start(credentials).await.unwrap();
        let home = tempfile::tempdir().unwrap();
        let clock = Arc::new(ManualClock::new(Timestamp::from_millis(1_700_000_000_000)));
        let mut config = test_config(home.path(), network_for(imap.port(), smtp.port()));
        config.clock = Arc::clone(&clock) as Arc<dyn Clock>;
        let engine = open_engine(config.clone()).unwrap();
        let mut world = TestWorld {
            home,
            config,
            engine,
            account: None,
            imap: Some(imap),
            smtp: None,
        };
        world.account = add_test_account(&world.engine).await;
        SendWorld { world, smtp, clock }
    }

    /// Passe le délai d'annulation, attend un message chez le faux SMTP puis le temps d'un second envoi ; rend le nombre reçu.
    pub async fn deliver(&self) -> usize {
        self.clock.advance_millis(SEND_DELAY_PASSED);
        for _ in 0..600 {
            if !self.smtp.received().is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        self.clock.advance_millis(SEND_DELAY_PASSED);
        tokio::time::sleep(Duration::from_millis(1_000)).await;
        self.smtp.received().len()
    }
}
