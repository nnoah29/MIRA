#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Support commun des tests du pont : configuration jetable du moteur et envoi d'une requête JSON.

pub mod chat_fixture;

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
use mira_bridge::runtime;
use serde_json::Value;

/// Configuration de test : tout sous un dossier jetable, le domaine `bridge.test` vise les faux serveurs.
pub fn config(home: &std::path::Path, port: u16, smtp_port: u16) -> EngineConfig {
    let endpoint = |port| ServerEndpoint {
        host: "127.0.0.1".into(),
        port,
        security: Security::Plaintext,
    };
    let mut network = NetworkPolicy::default();
    network.server_overrides.insert(
        "bridge.test".into(),
        ServerSettings {
            imap: endpoint(port),
            smtp: endpoint(smtp_port),
        },
    );
    network.background_sync = BackgroundSync::On(SupervisorConfig::default());
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
        clock: Arc::new(ManualClock::new(Timestamp::from_millis(1_704_100_000_000))),
        ids: Arc::new(SequentialIds::new()),
        oauth: OAuthClientConfig::default(),
        network,
    }
}

/// Envoie une requête déjà en JSON.
pub async fn send(request: &Value) -> Value {
    serde_json::from_str(&runtime::execute(&request.to_string()).await).unwrap()
}
