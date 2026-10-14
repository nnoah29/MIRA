//! Construction de l'instantané courant du contrat et chemin du fichier versionné.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mira_core::api::Request;
use serde_json::Value;

use super::{
    all_responses, request_name, request_texts, sync_domain, write_requests, write_responses,
};

/// Chemin du fichier d'instantané versionné
pub fn snapshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/contract_v1.json")
}

/// Exemples de requêtes désérialisés depuis leur texte
pub fn parsed_requests() -> Vec<Request> {
    let texts = request_texts();
    texts
        .iter()
        .map(|text| serde_json::from_str(text).unwrap())
        .collect()
}

/// Instantané courant : les exemples sérialisés, classés par nom de variante
pub fn current_snapshot() -> Value {
    let mut requests = BTreeMap::new();
    for request in parsed_requests() {
        let value = serde_json::to_value(&request).unwrap();
        requests.insert(request_name(&request).to_string(), value);
    }
    let mut responses = BTreeMap::new();
    for response in all_responses() {
        let value = serde_json::to_value(&response).unwrap();
        responses.insert(value["type"].as_str().unwrap().to_string(), value);
    }
    for (name, request) in write_requests().into_iter().chain(sync_domain::requests()) {
        requests.insert(name, serde_json::to_value(&request).unwrap());
    }
    for (name, response) in write_responses()
        .into_iter()
        .chain(sync_domain::responses())
    {
        responses.insert(name, serde_json::to_value(&response).unwrap());
    }
    serde_json::json!({ "contract": "1.0", "requests": requests, "responses": responses })
}
