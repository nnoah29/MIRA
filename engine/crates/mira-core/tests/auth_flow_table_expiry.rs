#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Table des flux OAuth : un flux abandonné expire au lieu de fuir (revue 17, audit 13).

mod support_auth;

use std::time::Duration;

use mira_core::auth::{FlowEntry, FlowTarget, OAuthFlows, start_flow};
use mira_vocabulary::{AccountId, EmailAddress, ProviderId};
use support_auth::{CLIENT_ID, USER, oauth_settings, seeded_rng, started_oauth};

/// Construit l'entrée d'un flux démarré contre le faux serveur
async fn entry(seed: u64, name: &str) -> FlowEntry {
    let (server, _) = started_oauth().await;
    let mut rng = seeded_rng(seed);
    let pending = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
        .await
        .unwrap();
    let target = FlowTarget {
        account: AccountId::from_raw(name),
        address: EmailAddress::parse(USER).unwrap(),
        provider: ProviderId::new("generic"),
        reauthentication: false,
    };
    FlowEntry::new(pending, target)
}

/// Un flux en attente depuis plus que la durée de vie est oublié à l'enregistrement suivant
#[tokio::test]
async fn regression_abandoned_flow_expires_when_the_next_one_registers() {
    let flows = OAuthFlows::with_ttl(Duration::from_millis(50));
    flows.register("old".to_string(), entry(41, "acc-old").await);
    tokio::time::sleep(Duration::from_millis(120)).await;
    flows.register("new".to_string(), entry(42, "acc-new").await);
    assert!(flows.cancel("old").is_none());
    assert!(flows.cancel("new").is_some());
}

/// Un flux expiré ne peut plus être pris en charge pour l'attente de la redirection
#[tokio::test]
async fn regression_expired_flow_cannot_begin_completion() {
    let flows = OAuthFlows::with_ttl(Duration::from_millis(50));
    flows.register("old".to_string(), entry(43, "acc-old").await);
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert!(flows.begin_completion("old").is_err());
}

/// Un flux récent reste utilisable
#[tokio::test]
async fn fresh_flow_can_begin_completion() {
    let flows = OAuthFlows::with_ttl(Duration::from_secs(60));
    flows.register("fresh".to_string(), entry(44, "acc-fresh").await);
    assert!(flows.begin_completion("fresh").is_ok());
}
