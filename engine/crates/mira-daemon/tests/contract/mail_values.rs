//! Scénario principal : comptes, boîtes, messages et corps arrivent en JSON du contrat, sans secret.

use serde_json::{Value, json};

use super::driver::Driver;
use super::mail_events::check_no_event_replay;
use super::mail_fixture::PASSWORD;
use super::mail_html::{check_original_html, check_rich_message};
use super::mail_start::{add_account, send, start, wait_inbox};

/// Sujets triés des messages d'une page.
fn subjects(items: &[Value]) -> Vec<&str> {
    let mut subjects: Vec<&str> = items
        .iter()
        .map(|m| m["subject"].as_str().unwrap())
        .collect();
    subjects.sort();
    subjects
}

/// Vérifie le premier message (expéditeur, drapeaux, corps) et rend les trois messages lus.
async fn check_listing<D: Driver>(driver: &D, page: &Value) -> (Value, Value) {
    let items = page["data"]["data"]["items"].as_array().unwrap();
    assert_eq!(subjects(items), ["Bonjour", "Riche", "Second"]);
    let first = items.iter().find(|m| m["subject"] == "Bonjour").unwrap();
    assert_eq!(first["from"][0]["address"], "bob@example.org");
    assert_eq!(first["flags"]["seen"], false);
    let body = send(driver, &json!({"domain": "message", "request": {"op": "get_message_body", "message": first["id"]}})).await;
    assert!(
        body["data"]["data"]["text"]
            .as_str()
            .unwrap()
            .contains("Texte du premier.")
    );
    assert!(!page.to_string().contains(PASSWORD));
    let rich = items.iter().find(|m| m["subject"] == "Riche").unwrap();
    (rich.clone(), first.clone())
}

/// Comptes, boîtes, messages et corps arrivent en JSON du contrat, sans secret.
pub async fn the_engine_serves_real_values_as_json<D: Driver>() {
    let started = start::<D>().await;
    let driver = &started.driver;
    let account = add_account(driver).await;
    let sync = json!({"domain": "sync", "request": {"op": "start_sync", "scope": {"type": "all"}}});
    let sync = send(driver, &sync).await;
    assert_eq!(sync["ok"], true, "{sync}");
    let (_, page) = wait_inbox(driver, &account).await;
    let (rich, plain) = check_listing(driver, &page).await;
    check_rich_message(driver, &rich["id"]).await;
    check_original_html(driver, &rich, &plain).await;
    check_no_event_replay(driver).await;
    assert!(driver.close().await.contains("\"ok\":true"));
}
