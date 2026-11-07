//! Règles de bout en bout : création, liste, suppression en JSON du contrat.

use serde_json::{Value, json};

use super::driver::Driver;
use super::mail_fixture::config;
use super::mail_start::send;

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

/// Créer, lister puis supprimer une règle passe par le chemin avec des enveloppes valides.
pub async fn rule_create_list_delete<D: Driver>() {
    let home = tempfile::tempdir().unwrap();
    let driver = D::start(Some(config(home.path(), 1, 1))).await;
    let create = rule_request(
        json!({"op": "create_rule", "rule": invoice_rule(), "idempotency_key": "k-create"}),
    );
    let created = send(&driver, &create).await;
    assert_eq!(created["ok"], true, "{created}");
    let listed = send(&driver, &rule_request(json!({"op": "list_rules"}))).await;
    let rules = listed["data"]["data"]["data"].as_array().unwrap();
    assert_eq!(rules.len(), 1, "{listed}");
    assert_eq!(rules[0]["status"]["type"], "valid", "{listed}");
    assert_eq!(rules[0]["rule"]["name"], "Factures", "{listed}");
    let id = rules[0]["rule"]["id"].clone();
    let delete =
        rule_request(json!({"op": "delete_rule", "rule": id, "idempotency_key": "k-delete"}));
    assert_eq!(send(&driver, &delete).await["ok"], true);
    let after = send(&driver, &rule_request(json!({"op": "list_rules"}))).await;
    assert_eq!(
        after["data"]["data"]["data"].as_array().unwrap().len(),
        0,
        "{after}"
    );
    assert!(driver.close().await.contains("\"ok\":true"));
}
