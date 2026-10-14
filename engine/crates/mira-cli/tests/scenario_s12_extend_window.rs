#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S12 par le CLI : remonter au-delà de la plage, fils complétés, recherche étendue au serveur (D-22, inv. 27).

#[allow(dead_code)]
mod cli_support;

use cli_support::json_views::page_ids;
use cli_support::sb_events::events_of_type;
use cli_support::sb_world::SbWorld;
use mira_testkit::imap::CapabilitySet;

const USER: &str = "alice@folders.test";

/// Compte synchronisé sur la plage de 90 jours : « Reponse » (1 j) est local, « Racine » (100 j) et « Vieux » (200 j) non.
async fn aged_world() -> (SbWorld, String, String) {
    let world = SbWorld::start(&[USER], CapabilitySet::full()).await;
    world.deliver_aged(USER, "Racine", 100, None);
    world.deliver_aged(USER, "Reponse", 1, Some("Racine"));
    world.deliver_aged(USER, "Vieux", 200, None);
    world.use_real_time();
    let account = world.add_account(USER).await;
    world.run_sync_ok(&["sync", "start"]).await;
    let inbox = world.mailbox_named(&account, "INBOX").await;
    (world, account, inbox)
}

/// Remonter de 30 jours : les en-têtes de « Racine » arrivent, son fil se complète, la progression est publiée
#[tokio::test]
async fn scenario_s12_extend_window_downloads_headers_and_completes_threads_via_cli() {
    let (world, _account, inbox) = aged_world().await;
    assert_eq!(world.subjects_of(&inbox).await, ["Reponse"]);
    let before = world.run_ok(&["thread", "list", &inbox]).await;
    assert_eq!(before["data"]["items"][0]["message_count"], 1);
    let thread = before["data"]["items"][0]["id"].clone();
    let args = ["sync", "extend-window", "--mailbox", &inbox, "--days", "30"];
    let data = world.run_sync_ok(&args).await;
    assert_eq!(data["data"]["kind"], "extend_window");
    assert_eq!(data["data"]["status"]["type"], "finished");
    assert_eq!(world.subjects_of(&inbox).await, ["Racine", "Reponse"]);
    let after = world.run_ok(&["thread", "list", &inbox]).await;
    assert_eq!(page_ids(&after).len(), 1);
    assert_eq!(after["data"]["items"][0]["id"], thread);
    assert_eq!(after["data"]["items"][0]["message_count"], 2);
    let events = world.events().await;
    let operation = data["data"]["id"].as_str().unwrap();
    let progress = events_of_type(&events, "operation_progress");
    assert!(progress.iter().any(|step| step["operation"] == operation));
    let finished = events_of_type(&events, "operation_finished");
    assert!(finished.iter().any(|end| end["operation"] == operation));
}

/// Une seconde remontée atteint le courrier plus ancien sans changer l'identité des messages déjà là
#[tokio::test]
async fn scenario_s12_second_extension_reaches_older_mail_via_cli() {
    let (world, _account, inbox) = aged_world().await;
    let first = ["sync", "extend-window", "--mailbox", &inbox, "--days", "30"];
    world.run_sync_ok(&first).await;
    let racine = world.message_id_of(&inbox, "Racine").await;
    let reponse = world.message_id_of(&inbox, "Reponse").await;
    assert_eq!(world.subjects_of(&inbox).await.len(), 2);
    let second = [
        "sync",
        "extend-window",
        "--mailbox",
        &inbox,
        "--days",
        "100",
    ];
    let data = world.run_sync_ok(&second).await;
    assert_eq!(data["data"]["status"]["type"], "finished");
    let subjects = world.subjects_of(&inbox).await;
    assert_eq!(subjects, ["Racine", "Reponse", "Vieux"]);
    assert_eq!(world.message_id_of(&inbox, "Racine").await, racine);
    assert_eq!(world.message_id_of(&inbox, "Reponse").await, reponse);
}

/// Un mot absent de la plage locale : la recherche locale ne dit rien et n'ouvre aucune connexion, la recherche serveur le trouve
#[tokio::test]
async fn scenario_s12_server_search_finds_mail_outside_the_local_range_via_cli() {
    let (world, account, inbox) = aged_world().await;
    let connections = world.imap.connection_count();
    let local = world.run_ok(&["search", "local", "Vieux"]).await;
    assert!(page_ids(&local).is_empty());
    assert_eq!(world.imap.connection_count(), connections);
    let json = world
        .run(&["search", "server", &account, "Vieux", "--json"])
        .await;
    assert_eq!(json.code, 0, "{}", json.stderr);
    let found = cli_support::parse_json(&json);
    assert_eq!(found["data"]["type"], "server_search");
    let hits = found["data"]["data"]["hits"].as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["mailbox"], inbox.as_str());
    assert!(hits[0]["message"].is_null());
    assert!(world.imap.connection_count() > connections);
    assert_eq!(world.subjects_of(&inbox).await, ["Reponse"]);
}

/// Erreurs de la remontée : boîte inconnue (11), nombre de jours invalide (10)
#[tokio::test]
async fn scenario_s12_extend_window_errors_use_the_contract_exit_codes_via_cli() {
    let (world, _account, inbox) = aged_world().await;
    let unknown = [
        "sync",
        "extend-window",
        "--mailbox",
        "absente",
        "--days",
        "30",
    ];
    let refused = world.run_sync(&unknown).await;
    assert_eq!(refused.code, 11, "{}", refused.stderr);
    let invalid = [
        "sync",
        "extend-window",
        "--mailbox",
        &inbox,
        "--days",
        "abc",
    ];
    assert_eq!(world.run_sync(&invalid).await.code, 10);
    assert_eq!(world.subjects_of(&inbox).await, ["Reponse"]);
}
