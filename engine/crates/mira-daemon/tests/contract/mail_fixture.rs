//! Moteur sur de faux serveurs IMAP et SMTP de la boucle locale, avec trois messages dans la boîte de réception.

use std::sync::Arc;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::ai::AiProviderRegistry;
use mira_assembly::api::EngineConfig;
use mira_assembly::auth::OAuthClientConfig;
use mira_assembly::build_registry;
use mira_assembly::mira_vocabulary::{
    ProfileId, Security, ServerEndpoint, ServerSettings, Timestamp,
};
use mira_assembly::ports::{ManualClock, MemorySecretStore, SequentialIds};
use mira_assembly::profile::ProfileEnvironment;
use mira_assembly::sync::{BackgroundSync, SupervisorConfig};
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::smtp::FakeSmtpServer;

pub use super::mail_samples::{PASSWORD, USER};
use super::mail_samples::{html_mail, raw_mail};

/// Politique réseau : le domaine `bridge.test` vise les faux serveurs, synchronisation en arrière-plan active.
fn network_to(port: u16, smtp_port: u16) -> NetworkPolicy {
    let endpoint = |port| ServerEndpoint {
        host: "127.0.0.1".into(),
        port,
        security: Security::Plaintext,
    };
    let servers = ServerSettings {
        imap: endpoint(port),
        smtp: endpoint(smtp_port),
    };
    let mut network = NetworkPolicy::default();
    network
        .server_overrides
        .insert("bridge.test".into(), servers);
    network.background_sync = BackgroundSync::On(SupervisorConfig::default());
    network
}

/// Configuration de test : tout sous un dossier jetable, le domaine `bridge.test` vise les faux serveurs.
pub fn config(home: &std::path::Path, port: u16, smtp_port: u16) -> EngineConfig {
    config_with_clock(home, port, smtp_port, test_clock())
}

/// Horloge de test arrêtée au 2024-01-01 11:00 UTC.
pub fn test_clock() -> Arc<ManualClock> {
    Arc::new(ManualClock::new(Timestamp::from_millis(1_704_100_000_000)))
}

/// Comme `config`, avec une horloge que le test peut faire avancer.
pub fn config_with_clock(
    home: &std::path::Path,
    port: u16,
    smtp_port: u16,
    clock: Arc<ManualClock>,
) -> EngineConfig {
    EngineConfig {
        profile: ProfileId::from_raw("default"),
        environment: ProfileEnvironment {
            mira_home: Some(home.to_path_buf()),
            xdg_data_home: None,
            home: None,
        },
        registry: build_registry(),
        ai_registry: AiProviderRegistry::new(),
        secrets: Arc::new(MemorySecretStore::new()),
        clock,
        ids: Arc::new(SequentialIds::new()),
        oauth: OAuthClientConfig::default(),
        network: network_to(port, smtp_port),
    }
}

/// Démarre les faux serveurs IMAP (trois messages dans `INBOX`) et SMTP pour ce compte.
pub async fn servers() -> (FakeImapServer, FakeSmtpServer) {
    let credentials = Credentials::default();
    credentials.add_password(USER, PASSWORD);
    let store = FakeMailStore::new();
    let first = raw_mail("Bonjour", "Texte du premier.");
    store.deliver(USER, "INBOX", &first, &[]);
    store.deliver(USER, "INBOX", &html_mail(), &[]);
    store.deliver(
        USER,
        "INBOX",
        &raw_mail("Second", "Texte du second."),
        &["\\Seen"],
    );
    let imap = FakeImapServer::start(store, credentials.clone(), CapabilitySet::full())
        .await
        .unwrap();
    let smtp = FakeSmtpServer::start(credentials).await.unwrap();
    (imap, smtp)
}
