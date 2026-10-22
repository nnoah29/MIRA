#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règles de bout en bout par le pont : création, liste, suppression en JSON du contrat.

use std::sync::Arc;

use mira_assembly::accounts::NetworkPolicy;
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
use serde_json::{Value, json};

/// Configuration de test : tout sous un dossier jetable, le domaine `bridge.test` vise les faux serveurs.
fn config(home: &std::path::Path, port: u16, smtp_port: u16) -> EngineConfig {
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
        secrets: Arc::new(MemorySecretStore::new()),
        clock: Arc::new(ManualClock::new(Timestamp::from_millis(1_704_100_000_000))),
        ids: Arc::new(SequentialIds::new()),
        oauth: OAuthClientConfig::default(),
        network,
    }
}

/// Envoie une requête déjà en JSON.
async fn send(request: &Value) -> Value {
    serde_json::from_str(&runtime::execute(&request.to_string()).await).unwrap()
}

/// Requête de règle du contrat, enveloppée dans son domaine.
fn rule_request(request: Value) -> Value {
    json!({"domain": "rule", "request": request})
}

/// Règle valide : objet contient « facture » puis marquer lu, sur tous les comptes.
fn invoice_rule() -> Value {
    json!({
        "id": "client-id", "name": "Factures", "enabled": true,
        "scope": {"type": "all_accounts"},
        "alternatives": [{"conditions": [{"negated": false, "test": {
            "type": "subject", "matcher": {"op": "contains", "values": ["facture"]}}}]}],
        "actions": [{"type": "mark_read", "read": true}],
        "schema_version": 1
    })
}

/// Créer, lister puis supprimer une règle passe par le pont avec des enveloppes valides.
#[tokio::test(flavor = "multi_thread")]
async fn rule_create_list_delete_through_bridge() {
    let home = tempfile::tempdir().unwrap();
    let opened: Value = serde_json::from_str(&runtime::open(config(home.path(), 1, 1))).unwrap();
    assert_eq!(opened["ok"], true, "{opened}");
    let create = rule_request(
        json!({"op": "create_rule", "rule": invoice_rule(), "idempotency_key": "k-create"}),
    );
    let created = send(&create).await;
    assert_eq!(created["ok"], true, "{created}");
    let listed = send(&rule_request(json!({"op": "list_rules"}))).await;
    let rules = listed["data"]["data"]["data"].as_array().unwrap();
    assert_eq!(rules.len(), 1, "{listed}");
    assert_eq!(rules[0]["status"]["type"], "valid", "{listed}");
    assert_eq!(rules[0]["rule"]["name"], "Factures", "{listed}");
    let id = rules[0]["rule"]["id"].clone();
    let delete =
        rule_request(json!({"op": "delete_rule", "rule": id, "idempotency_key": "k-delete"}));
    assert_eq!(send(&delete).await["ok"], true);
    let after = send(&rule_request(json!({"op": "list_rules"}))).await;
    assert_eq!(
        after["data"]["data"]["data"].as_array().unwrap().len(),
        0,
        "{after}"
    );
    assert!(runtime::close().await.contains("\"ok\":true"));
}
