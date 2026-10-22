#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat du domaine « libellés locaux » : exemples sérialisés, aller-retour JSON, présence dans l'instantané versionné.

mod contract_examples;
#[allow(dead_code)]
#[path = "provider_fakes/mod.rs"]
mod provider_fakes;
mod support;
mod support_mirror;

use std::collections::BTreeSet;

use contract_examples::local_label_domain;
use contract_examples::response_names::group_of;
use contract_examples::snapshot::snapshot_path;
use mira_core::api::{Request, Response};
use serde_json::Value;

/// Les huit opérations du domaine ont un exemple nommé d'après leur `op`, qui survit à un aller-retour JSON
#[test]
fn every_local_label_operation_has_an_example_that_round_trips() {
    let mut distinct = BTreeSet::new();
    for (name, request) in local_label_domain::requests() {
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["domain"], "local_label");
        let tag = value["request"]["op"].as_str().unwrap().to_string();
        assert_eq!(name, format!("local_label.{tag}"));
        let back: Request = serde_json::from_value(value).unwrap();
        assert_eq!(back, request, "requête {name}");
        distinct.insert(tag);
    }
    assert_eq!(distinct.len(), 8);
}

/// Chaque exemple de réponse appartient au domaine libellés locaux et survit à un aller-retour JSON
#[test]
fn local_label_responses_round_trip_through_json() {
    for (name, response) in local_label_domain::responses() {
        assert_eq!(group_of(&response), "local_label");
        let text = serde_json::to_string(&response).unwrap();
        let back: Response = serde_json::from_str(&text).unwrap();
        assert_eq!(back, response, "réponse {name}");
    }
}

/// L'instantané versionné contient chaque exemple du domaine libellés locaux, à l'identique
#[test]
fn snapshot_file_holds_every_local_label_example() {
    let text = std::fs::read_to_string(snapshot_path()).unwrap();
    let stored: Value = serde_json::from_str(&text).unwrap();
    for (name, request) in local_label_domain::requests() {
        let expected = serde_json::to_value(&request).unwrap();
        assert_eq!(stored["requests"][&name], expected, "requête {name}");
    }
    for (name, response) in local_label_domain::responses() {
        let expected = serde_json::to_value(&response).unwrap();
        assert_eq!(stored["responses"][&name], expected, "réponse {name}");
    }
}
