#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Libellés locaux de bout en bout par le pont : création, liste, renommage, reprise, suppression en JSON du contrat.

mod support;

use mira_bridge::runtime;
use serde_json::{Value, json};
use support::{config, send};

/// Requête du domaine libellés locaux, enveloppée dans son domaine.
fn label_request(request: Value) -> Value {
    json!({"domain": "local_label", "request": request})
}

/// Données de la réponse : l'enveloppe `{ok, data}` contient `{type, data}` qui contient `{type, data}`.
fn answer(reply: &Value) -> &Value {
    assert_eq!(reply["ok"], true, "{reply}");
    &reply["data"]["data"]["data"]
}

/// Noms des libellés listés par le moteur, dans l'ordre.
async fn listed_names() -> Vec<String> {
    let reply = send(&label_request(json!({"op": "list_local_labels"}))).await;
    let labels = answer(&reply).as_array().unwrap();
    labels
        .iter()
        .map(|l| l["name"].as_str().unwrap().to_string())
        .collect()
}

/// Créer, renommer, reprendre un libellé de l'app (clé inconnue ignorée) puis tout supprimer, par le pont.
#[tokio::test(flavor = "multi_thread")]
async fn local_label_lifecycle_through_bridge() {
    let home = tempfile::tempdir().unwrap();
    let opened: Value = serde_json::from_str(&runtime::open(config(home.path(), 1, 1))).unwrap();
    assert_eq!(opened["ok"], true, "{opened}");
    let create = json!({"op": "create_local_label", "name": "À payer", "idempotency_key": "k-1"});
    let created = send(&label_request(create)).await;
    let id = answer(&created)["id"].clone();
    let rename = json!({"op": "rename_local_label", "label": id, "name": "Urgent"});
    assert_eq!(send(&label_request(rename)).await["ok"], true);
    let labels = json!([{"name": "Banque", "color": 4283215872u32, "conversations": ["inconnue"]}]);
    let import = json!({"op": "import_local_labels", "labels": labels, "idempotency_key": "k-2"});
    let imported = send(&label_request(import)).await;
    let report = json!({"labels": 1, "assignments": 0, "ignored": 1});
    assert_eq!(answer(&imported), &report, "{imported}");
    assert_eq!(listed_names().await, ["Banque", "Urgent"]);
    let poses = send(&label_request(
        json!({"op": "list_local_label_assignments"}),
    ))
    .await;
    assert!(answer(&poses).as_array().unwrap().is_empty(), "{poses}");
    let delete = json!({"op": "delete_local_label", "label": id});
    assert_eq!(send(&label_request(delete)).await["ok"], true);
    assert_eq!(listed_names().await, ["Banque"]);
    assert!(runtime::close().await.contains("\"ok\":true"));
}
