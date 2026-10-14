#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S1 par le CLI : compte Gmail par OAuth2 (PKCE, redirection locale), jetons au trousseau, première synchro.

#[allow(dead_code)]
mod cli_support;

use cli_support::parse_json;
use cli_support::sa_events::{count_of, first_position};
use cli_support::sa_mail::raw_mail;
use cli_support::sa_s1_gmail::{GMAIL_USER, GmailWorld};
use cli_support::sa_s1_login::created_id;
use mira_assembly::mira_vocabulary::SecretKind;

/// Gmail est détecté, OAuth2 vient en tête puis le mot de passe d'application (D-06)
#[tokio::test]
async fn scenario_s1_gmail_detection_offers_oauth_first_via_cli() {
    let world = GmailWorld::start("default").await;
    let detected = world
        .run(&["provider", "detect", GMAIL_USER, "--json"])
        .await;
    let data = &parse_json(&detected)["data"];
    assert_eq!(data["provider"]["id"], "gmail");
    let methods = data["provider"]["auth_methods"].as_array().unwrap();
    assert_eq!(
        (methods[0]["type"].as_str(), methods[1]["type"].as_str()),
        (Some("oauth2"), Some("app_password"))
    );
    assert_eq!(data["servers"]["imap"]["host"], "imap.gmail.com");
}

/// Le flux annonce son adresse sur stderr, les jetons vont au trousseau sous profil + compte (D-10), le compte est connecté
#[tokio::test]
async fn scenario_s1_oauth_flow_adds_the_account_with_tokens_under_the_profile_key_via_cli() {
    let world = GmailWorld::start("work").await;
    let (result, url) = world.login(&[], true).await;
    assert_eq!(result.code, 0, "{}", result.stderr);
    assert!(
        url.contains("code_challenge=") && url.contains("state="),
        "{url}"
    );
    let account = &parse_json(&result)["data"]["data"];
    assert_eq!(
        (account["auth"].as_str(), account["provider"].as_str()),
        (Some("oauth2"), Some("gmail"))
    );
    assert_eq!(
        (
            account["state"]["type"].as_str(),
            account["profile"].as_str()
        ),
        (Some("connected"), Some("work"))
    );
    let id = created_id(&result);
    assert!(world.token("work", &id, SecretKind::AccessToken).is_some());
    assert!(world.token("work", &id, SecretKind::RefreshToken).is_some());
    assert!(
        world
            .token("default", &id, SecretKind::AccessToken)
            .is_none()
    );
    assert_eq!(world.secrets.entry_count(), 2);
}

/// L'événement d'ajout précède la première synchro, qui lit le courrier avec les jetons (S4)
#[tokio::test]
async fn scenario_s1_first_sync_follows_the_added_account_via_cli() {
    let world = GmailWorld::start("default").await;
    let raw = raw_mail(GMAIL_USER, "Bonjour Gmail", "");
    world
        .store
        .deliver_gmail(GMAIL_USER, &["INBOX"], &raw, 1001, 2001);
    let (result, _url) = world.login(&["--sync-wait", "--timeout", "30"], true).await;
    assert_eq!(result.code, 0, "{}", result.stderr);
    let events = world.run(&["events", "--limit", "1000", "--json"]).await;
    let list = parse_json(&events)["data"]["data"]
        .as_array()
        .unwrap()
        .clone();
    let added = first_position(&list, "account_added").unwrap();
    assert!(added < first_position(&list, "sync_started").unwrap());
    assert!(count_of(&list, "sync_finished") >= 1);
    let account = created_id(&result);
    let boxes = world.run(&["mailbox", "list", &account, "--json"]).await;
    let boxes = parse_json(&boxes)["data"]["data"]
        .as_array()
        .unwrap()
        .clone();
    let inbox = boxes.iter().find(|item| item["name"] == "INBOX").unwrap();
    assert_eq!(inbox["total"], 1);
}

/// Aucun jeton ni secret d'autorisation dans les sorties, les événements, la base ou les journaux (inv. 15)
#[tokio::test]
async fn scenario_s1_tokens_never_leak_via_cli() {
    let world = GmailWorld::start("default").await;
    let (result, _url) = world.login(&["--sync-wait", "--timeout", "30"], true).await;
    let account = created_id(&result);
    world.run(&["account", "list", "--json"]).await;
    world.run(&["events", "--json"]).await;
    world.run(&["sync", "status", "--json"]).await;
    for kind in [SecretKind::AccessToken, SecretKind::RefreshToken] {
        let token = world.token("default", &account, kind).unwrap();
        assert!(!world.leaked(&token));
    }
}

/// Autorisation refusée (15) ou serveur injoignable après autorisation (13) : ni compte, ni jeton, ni événement d'ajout
#[tokio::test]
async fn scenario_s1_denied_or_unreachable_registers_nothing_via_cli() {
    let world = GmailWorld::start("default").await;
    let (denied, _url) = world.login(&[], false).await;
    assert_eq!(denied.code, 15, "{}", denied.stderr);
    world.imap.shutdown();
    let (unreachable, _url) = world.login(&[], true).await;
    assert_eq!(
        unreachable.code, 13,
        "{} {}",
        unreachable.stdout, unreachable.stderr
    );
    assert_eq!(world.secrets.entry_count(), 0);
    let listed = world.run(&["account", "list", "--json"]).await;
    assert!(
        parse_json(&listed)["data"]["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let events = world.run(&["events", "--json"]).await;
    let list = parse_json(&events)["data"]["data"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(count_of(&list, "account_added"), 0);
}

/// Après redémarrage (autre invocation), le compte est connecté et la synchro utilise les jetons rangés sans navigateur
#[tokio::test]
async fn scenario_s1_account_survives_restart_and_syncs_with_stored_tokens_via_cli() {
    let world = GmailWorld::start("default").await;
    let (result, _url) = world.login(&[], true).await;
    let account = created_id(&result);
    let listed = world.run(&["account", "list", "--json"]).await;
    assert_eq!(
        parse_json(&listed)["data"]["data"][0]["id"],
        account.as_str()
    );
    assert_eq!(parse_json(&listed)["data"]["data"][0]["auth"], "oauth2");
    let synced = world.run(&["sync", "start", "--json"]).await;
    assert_eq!(synced.code, 0, "{}", synced.stderr);
    assert_eq!(
        parse_json(&synced)["data"]["data"]["status"]["type"],
        "finished"
    );
    assert_eq!(world.secrets.entry_count(), 2);
}
