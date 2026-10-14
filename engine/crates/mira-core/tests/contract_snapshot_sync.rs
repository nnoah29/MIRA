#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat du domaine « synchronisation » : exemples sérialisés, aller-retour JSON, présence dans l'instantané versionné.

mod contract_examples;
#[allow(dead_code)]
#[path = "provider_fakes/mod.rs"]
mod provider_fakes;
mod support;
mod support_mirror;

use std::collections::BTreeSet;

use contract_examples::snapshot::snapshot_path;
use contract_examples::sync_domain;
use mira_core::api::{Request, Response};
use serde_json::Value;

/// Chaque exemple de requête ou de réponse de synchronisation survit à un aller-retour JSON sans perte
#[test]
fn sync_examples_round_trip_through_json() {
    for (name, request) in sync_domain::requests() {
        let text = serde_json::to_string(&request).unwrap();
        let back: Request = serde_json::from_str(&text).unwrap();
        assert_eq!(back, request, "requête {name}");
    }
    for (name, response) in sync_domain::responses() {
        let text = serde_json::to_string(&response).unwrap();
        let back: Response = serde_json::from_str(&text).unwrap();
        assert_eq!(back, response, "réponse {name}");
    }
}

/// Les cinq opérations du domaine ont un exemple, nommé d'après leur étiquette JSON `op`
#[test]
fn every_sync_operation_has_a_request_example() {
    let mut distinct = BTreeSet::new();
    for (name, request) in sync_domain::requests() {
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["domain"], "sync");
        let tag = value["request"]["op"].as_str().unwrap().to_string();
        assert!(name.starts_with(&format!("sync.{tag}")), "{name}");
        distinct.insert(tag);
    }
    let expected = [
        "start_sync",
        "pause_sync",
        "get_sync_status",
        "extend_window",
        "resync_mailbox",
    ];
    assert_eq!(distinct, expected.iter().map(|t| t.to_string()).collect());
}

/// Les trois portées de `StartSync` sont étiquetées `all`, `account` et `mailbox`
#[test]
fn start_sync_scopes_are_tagged() {
    let scopes: Vec<String> = sync_domain::requests()
        .iter()
        .map(|(_, request)| serde_json::to_value(request).unwrap())
        .filter(|value| value["request"]["op"] == "start_sync")
        .map(|value| {
            value["request"]["scope"]["type"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(scopes, ["all", "account", "mailbox"]);
}

/// L'instantané versionné contient chaque exemple de synchronisation, à l'identique
#[test]
fn snapshot_file_holds_every_sync_example() {
    let text = std::fs::read_to_string(snapshot_path()).unwrap();
    let stored: Value = serde_json::from_str(&text).unwrap();
    for (name, request) in sync_domain::requests() {
        let expected = serde_json::to_value(&request).unwrap();
        assert_eq!(stored["requests"][&name], expected, "requête {name}");
    }
    for (name, response) in sync_domain::responses() {
        let expected = serde_json::to_value(&response).unwrap();
        assert_eq!(stored["responses"][&name], expected, "réponse {name}");
    }
}
