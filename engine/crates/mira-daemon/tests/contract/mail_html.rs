//! HTML assaini, image incorporée enregistrable et HTML d'origine sans assainissement.

use std::time::Duration;

use serde_json::{Value, json};

use super::driver::Driver;
use super::mail_start::send;

/// HTML assaini : cid gardé, image distante retirée et signalée.
async fn check_safe_html<D: Driver>(driver: &D, id: &Value) {
    let html = send(
        driver,
        &json!({"domain": "message", "request": {"op": "get_safe_html", "message": id}}),
    )
    .await;
    let safe = &html["data"]["data"];
    assert!(safe["html"].as_str().unwrap().contains("cid:logo@x"));
    assert!(!safe["html"].as_str().unwrap().contains("tracker.example"));
    assert_eq!(
        safe["blocked_resources"][0],
        "https://tracker.example/p.png"
    );
}

/// L'image incorporée figure dans la liste des pièces jointes et s'enregistre sur disque.
async fn check_embedded_image<D: Driver>(driver: &D, id: &Value) {
    let list = send(
        driver,
        &json!({"domain": "message", "request": {"op": "list_attachments", "message": id}}),
    )
    .await;
    let logo = &list["data"]["data"][0];
    assert_eq!(logo["content_id"], "logo@x");
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("logo.png");
    let saved = send(
        driver,
        &json!({"domain": "message", "request": {"op": "save_attachment",
        "attachment": logo["id"], "destination": target.to_str().unwrap()}}),
    )
    .await;
    assert_eq!(saved["ok"], true, "{saved}");
    for _ in 0..100 {
        if target.exists() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("image incorporée jamais enregistrée");
}

/// HTML assaini (cid gardé, image distante retirée et signalée) et image incorporée enregistrable.
pub async fn check_rich_message<D: Driver>(driver: &D, id: &Value) {
    check_safe_html(driver, id).await;
    check_embedded_image(driver, id).await;
}

/// Le HTML d'origine garde tout (aucun assainissement) et un texte seul n'en donne pas.
pub async fn check_original_html<D: Driver>(driver: &D, rich: &Value, plain: &Value) {
    let original = driver.original_html(rich["id"].as_str().unwrap()).await;
    let original: Value = serde_json::from_str(&original).unwrap();
    let html = original["data"]["html"].as_str().unwrap();
    assert!(html.contains("<table>") && html.contains("cid:logo@x"));
    assert!(html.contains("https://tracker.example/p.png"));
    let none = driver.original_html(plain["id"].as_str().unwrap()).await;
    let none: Value = serde_json::from_str(&none).unwrap();
    assert!(none["data"]["html"].is_null());
}
