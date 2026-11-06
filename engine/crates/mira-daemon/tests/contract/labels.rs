//! Libellés locaux de bout en bout : création, liste, renommage, reprise, suppression en JSON du contrat.

use serde_json::{Value, json};

use super::driver::Driver;
use super::mail_fixture::config;
use super::mail_start::send;

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
async fn listed_names<D: Driver>(driver: &D) -> Vec<String> {
    let reply = send(driver, &label_request(json!({"op": "list_local_labels"}))).await;
    let labels = answer(&reply).as_array().unwrap();
    labels
        .iter()
        .map(|l| l["name"].as_str().unwrap().to_string())
        .collect()
}

/// Créer, renommer, reprendre un libellé de l'app (clé inconnue ignorée) puis tout supprimer.
pub async fn local_label_lifecycle<D: Driver>() {
    let home = tempfile::tempdir().unwrap();
    let driver = D::start(Some(config(home.path(), 1, 1))).await;
    let create = json!({"op": "create_local_label", "name": "À payer", "idempotency_key": "k-1"});
    let created = send(&driver, &label_request(create)).await;
    let id = answer(&created)["id"].clone();
    let rename = json!({"op": "rename_local_label", "label": id, "name": "Urgent"});
    assert_eq!(send(&driver, &label_request(rename)).await["ok"], true);
    let labels = json!([{"name": "Banque", "color": 4283215872u32, "conversations": ["inconnue"]}]);
    let import = json!({"op": "import_local_labels", "labels": labels, "idempotency_key": "k-2"});
    let imported = send(&driver, &label_request(import)).await;
    let report = json!({"labels": 1, "assignments": 0, "ignored": 1});
    assert_eq!(answer(&imported), &report, "{imported}");
    assert_eq!(listed_names(&driver).await, ["Banque", "Urgent"]);
    let poses = send(
        &driver,
        &label_request(json!({"op": "list_local_label_assignments"})),
    )
    .await;
    assert!(answer(&poses).as_array().unwrap().is_empty(), "{poses}");
    let delete = json!({"op": "delete_local_label", "label": id});
    assert_eq!(send(&driver, &label_request(delete)).await["ok"], true);
    assert_eq!(listed_names(&driver).await, ["Banque"]);
    assert!(driver.close().await.contains("\"ok\":true"));
}
