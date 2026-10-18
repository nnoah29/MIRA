#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Instantané du contrat v1 : un exemple sérialisé par variante de `Request` et de `Response`, comparé à un fichier.
//! `MIRA_UPDATE_SNAPSHOT=1 cargo test` réécrit le fichier après une évolution voulue du contrat.

mod contract_examples;
#[allow(dead_code)]
#[path = "provider_fakes/mod.rs"]
mod provider_fakes;
mod support;
mod support_mirror;

use contract_examples::response_names::{TYPES, group_of};
use contract_examples::snapshot::{current_snapshot, parsed_requests, snapshot_path};
use contract_examples::{
    all_responses, expected_request_names, request_name, write_requests, write_responses,
};
use mira_core::api::{Request, Response};
use serde_json::Value;

/// Le contrat sérialisé est identique à l'instantané versionné du dépôt
#[test]
fn contract_snapshot_requests_and_responses() {
    let current = current_snapshot();
    if std::env::var_os("MIRA_UPDATE_SNAPSHOT").is_some() {
        let text = serde_json::to_string_pretty(&current).unwrap() + "\n";
        std::fs::write(snapshot_path(), text).unwrap();
    }
    let stored: Value =
        serde_json::from_str(&std::fs::read_to_string(snapshot_path()).unwrap()).unwrap();
    assert_eq!(
        current, stored,
        "le contrat a changé : relire puis régénérer l'instantané"
    );
}

/// Chaque variante de requête a exactement un exemple, et son nom concorde avec sa forme sérialisée
#[test]
fn every_request_variant_has_exactly_one_example() {
    let mut names: Vec<String> = Vec::new();
    for request in parsed_requests() {
        let value = serde_json::to_value(&request).unwrap();
        let serialized = format!(
            "{}.{}",
            value["domain"].as_str().unwrap(),
            value["request"]["op"].as_str().unwrap()
        );
        assert_eq!(request_name(&request), serialized);
        names.push(serialized);
    }
    names.sort();
    let mut expected: Vec<String> = expected_request_names()
        .iter()
        .map(|n| n.to_string())
        .collect();
    expected.sort();
    assert_eq!(names, expected);
}

/// Chaque variante de réponse a exactement un exemple, qui se relit à l'identique
#[test]
fn every_response_variant_has_exactly_one_example() {
    let mut types: Vec<String> = Vec::new();
    for response in all_responses() {
        let text = serde_json::to_string(&response).unwrap();
        let back: Response = serde_json::from_str(&text).unwrap();
        assert_eq!(back, response);
        assert!(!group_of(&response).is_empty());
        types.push(
            serde_json::to_value(&response).unwrap()["type"]
                .as_str()
                .unwrap()
                .to_string(),
        );
    }
    types.sort();
    let mut expected: Vec<String> = TYPES.iter().map(|t| t.to_string()).collect();
    expected.sort();
    assert_eq!(types, expected);
}

/// Les exemples de requêtes se relisent à l'identique après sérialisation
#[test]
fn request_examples_round_trip() {
    for request in parsed_requests() {
        let text = serde_json::to_string(&request).unwrap();
        let back: Request = serde_json::from_str(&text).unwrap();
        assert_eq!(back, request);
    }
}

/// Chaque exemple d'écriture survit à un aller-retour JSON sans perte
#[test]
fn write_examples_round_trip_through_json() {
    for (name, request) in write_requests() {
        let text = serde_json::to_string(&request).unwrap();
        let back: Request = serde_json::from_str(&text).unwrap();
        assert_eq!(back, request, "requête {name}");
    }
    for (name, response) in write_responses() {
        let text = serde_json::to_string(&response).unwrap();
        let back: Response = serde_json::from_str(&text).unwrap();
        assert_eq!(back, response, "réponse {name}");
    }
}

/// Chaque opération d'écriture a au moins un exemple, nommé d'après son étiquette JSON `op`
#[test]
fn every_write_operation_has_a_request_example() {
    let mut distinct = std::collections::BTreeSet::new();
    for (name, request) in write_requests() {
        let value = serde_json::to_value(&request).unwrap();
        let tag = value["request"]["op"].as_str().unwrap().to_string();
        let domain = value["domain"].as_str().unwrap().to_string();
        assert!(name.starts_with(&format!("{domain}.{tag}")), "{name}");
        distinct.insert(format!("{domain}.{tag}"));
    }
    assert_eq!(distinct.len(), 6 + 12 + 3);
}
