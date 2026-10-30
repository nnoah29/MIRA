//! Construction de l'instantané courant du contrat et chemin du fichier versionné.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mira_core::api::{Request, Response};
use serde_json::Value;

use super::{
    Named, ai_domain, all_responses, label_settings_domain, local_label_domain, request_name,
    request_texts, rule_domain, settings_domain, sync_domain, system_label_domain, template_domain,
    write_requests, write_responses,
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

/// Exemples de requêtes nommés par domaine, hors ceux décrits par texte JSON
fn named_requests() -> Vec<Named<Request>> {
    write_requests()
        .into_iter()
        .chain(sync_domain::requests())
        .chain(rule_domain::requests())
        .chain(local_label_domain::requests())
        .chain(system_label_domain::requests())
        .chain(label_settings_domain::requests())
        .chain(settings_domain::requests())
        .chain(template_domain::requests())
        .chain(ai_domain::requests())
        .collect()
}

/// Exemples de réponses nommés par domaine, hors ceux de l'inventaire général
fn named_responses() -> Vec<Named<Response>> {
    write_responses()
        .into_iter()
        .chain(sync_domain::responses())
        .chain(rule_domain::responses())
        .chain(local_label_domain::responses())
        .chain(system_label_domain::responses())
        .chain(label_settings_domain::responses())
        .chain(settings_domain::responses())
        .chain(template_domain::responses())
        .chain(ai_domain::responses())
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
    for (name, request) in named_requests() {
        requests.insert(name, serde_json::to_value(&request).unwrap());
    }
    for (name, response) in named_responses() {
        responses.insert(name, serde_json::to_value(&response).unwrap());
    }
    serde_json::json!({ "contract": "1.2", "requests": requests, "responses": responses })
}
