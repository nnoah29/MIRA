#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat d'API figé (invariants 6, 30, 31) : un exemple sérialisé par variante, comparé à `snapshots/contract_v1.json`.
//! `MIRA_UPDATE_SNAPSHOT=1` réécrit le fichier après un changement volontaire du contrat.

mod contract_examples;

use std::collections::BTreeSet;

use contract_examples::{all_requests, all_responses};
use mira_core::api::{Request, Response};
use mira_vocabulary::CONTRACT_VERSION;
use serde_json::{Map, Value, json};

/// Contrat sérialisé : version, requêtes et réponses d'exemple, clés triées
fn current_contract() -> Value {
    let requests: Map<String, Value> = all_requests()
        .into_iter()
        .map(|(name, request)| (name, serde_json::to_value(request).unwrap()))
        .collect();
    let responses: Map<String, Value> = all_responses()
        .into_iter()
        .map(|(name, response)| (name, serde_json::to_value(response).unwrap()))
        .collect();
    json!({
        "contract_version": CONTRACT_VERSION,
        "requests": requests,
        "responses": responses,
    })
}

/// Le contrat sérialisé est identique à l'instantané versionné ; toute modification incompatible casse ce test
#[test]
fn contract_snapshot_requests_and_responses() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots/contract_v1.json"
    );
    let current = current_contract();
    if std::env::var_os("MIRA_UPDATE_SNAPSHOT").is_some() {
        let text = serde_json::to_string_pretty(&current).unwrap();
        std::fs::write(path, format!("{text}\n")).unwrap();
    }
    let stored: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        stored, current,
        "le contrat a changé : relire puis régénérer l'instantané"
    );
}

/// Chaque exemple survit à un aller-retour JSON sans perte (désérialisable par un client)
#[test]
fn every_example_round_trips_through_json() {
    for (name, request) in all_requests() {
        let text = serde_json::to_string(&request).unwrap();
        let back: Request = serde_json::from_str(&text).unwrap();
        assert_eq!(back, request, "requête {name}");
    }
    for (name, response) in all_responses() {
        let text = serde_json::to_string(&response).unwrap();
        let back: Response = serde_json::from_str(&text).unwrap();
        assert_eq!(back, response, "réponse {name}");
    }
}

/// Chaque opération d'écriture a au moins un exemple, nommé d'après son étiquette JSON `op`
#[test]
fn every_write_operation_has_a_request_example() {
    let mut distinct: BTreeSet<String> = BTreeSet::new();
    for (name, request) in all_requests() {
        let value = serde_json::to_value(&request).unwrap();
        let tag = value["request"]["op"].as_str().unwrap().to_string();
        let domain = value["domain"].as_str().unwrap().to_string();
        assert!(
            name.starts_with(&format!("{domain}.{tag}")),
            "{name} / {domain}.{tag}"
        );
        distinct.insert(format!("{domain}.{tag}"));
    }
    assert_eq!(distinct.len(), 6 + 11 + 3);
}
