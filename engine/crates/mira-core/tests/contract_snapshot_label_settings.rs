#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat du domaine « réglages des libellés » : exemples sérialisés, aller-retour JSON, présence dans l'instantané versionné.

mod contract_examples;
#[allow(dead_code)]
#[path = "provider_fakes/mod.rs"]
mod provider_fakes;
mod support;
mod support_mirror;

use std::collections::BTreeSet;

use contract_examples::label_settings_domain;
use contract_examples::response_names::group_of;
use contract_examples::snapshot::snapshot_path;
use mira_core::api::{Request, Response};
use serde_json::Value;

/// Les trois opérations du domaine ont un exemple nommé d'après leur `op`, qui survit à un aller-retour JSON
#[test]
fn every_label_settings_operation_has_an_example_that_round_trips() {
    let mut distinct = BTreeSet::new();
    for (name, request) in label_settings_domain::requests() {
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["domain"], "label_settings");
        let tag = value["request"]["op"].as_str().unwrap().to_string();
        assert_eq!(name, format!("label_settings.{tag}"));
        let back: Request = serde_json::from_value(value).unwrap();
        assert_eq!(back, request, "requête {name}");
        distinct.insert(tag);
    }
    assert_eq!(distinct.len(), 3);
}

/// Chaque exemple de réponse appartient au domaine réglages des libellés et survit à un aller-retour JSON
#[test]
fn label_settings_responses_round_trip_through_json() {
    for (name, response) in label_settings_domain::responses() {
        assert_eq!(group_of(&response), "label_settings");
        let text = serde_json::to_string(&response).unwrap();
        let back: Response = serde_json::from_str(&text).unwrap();
        assert_eq!(back, response, "réponse {name}");
    }
}

/// L'instantané versionné contient chaque exemple du domaine réglages des libellés, à l'identique
#[test]
fn snapshot_file_holds_every_label_settings_example() {
    let text = std::fs::read_to_string(snapshot_path()).unwrap();
    let stored: Value = serde_json::from_str(&text).unwrap();
    for (name, request) in label_settings_domain::requests() {
        let expected = serde_json::to_value(&request).unwrap();
        assert_eq!(stored["requests"][&name], expected, "requête {name}");
    }
    for (name, response) in label_settings_domain::responses() {
        let expected = serde_json::to_value(&response).unwrap();
        assert_eq!(stored["responses"][&name], expected, "réponse {name}");
    }
}
