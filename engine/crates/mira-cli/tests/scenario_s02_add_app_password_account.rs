#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S2 par le CLI : compte Yahoo (mot de passe d'application seul), secret transmis une fois sur stdin.

#[allow(dead_code)]
mod cli_support;

use cli_support::parse_json;
use cli_support::sa_s2_yahoo::{
    ADD_APP_PASSWORD as ADD, APP_PASSWORD, YAHOO_USER as USER, yahoo_world,
};
use mira_assembly::mira_vocabulary::SecretKind;

/// La détection ne propose que le mot de passe d'application ; la préparation rend les serveurs qui seront contactés
#[tokio::test]
async fn scenario_s2_yahoo_detection_offers_only_the_app_password_via_cli() {
    let world = yahoo_world().await;
    let detected = world.run_ok(&["provider", "detect", USER]).await;
    assert_eq!(detected["provider"]["id"], "yahoo");
    assert_eq!(
        detected["provider"]["auth_methods"],
        serde_json::json!([{"type": "app_password"}])
    );
    assert_eq!(detected["servers"]["imap"]["host"], "imap.mail.yahoo.com");
    let prepared = world
        .run_ok(&["account", "prepare", "--address", USER])
        .await;
    assert_eq!(prepared["data"]["provider"]["id"], "yahoo");
    assert!(prepared["data"]["servers"]["imap"]["port"].is_u64());
}

/// Le compte est ajouté, connecté, avec une identité par défaut ; le secret est au trousseau sous la clé profil + compte (D-10, inv. 11)
#[tokio::test]
async fn scenario_s2_app_password_account_is_added_with_the_secret_in_the_keychain_via_cli() {
    let world = yahoo_world().await;
    let added = world.run_stdin(&format!("{APP_PASSWORD}\n"), &ADD).await;
    assert_eq!(added.code, 0, "{}", added.stderr);
    let account = &parse_json(&added)["data"]["data"];
    assert_eq!(
        (account["auth"].as_str(), account["provider"].as_str()),
        (Some("app_password"), Some("yahoo"))
    );
    assert_eq!(account["state"]["type"], "connected");
    let id = account["id"].as_str().unwrap();
    assert_eq!(
        world
            .stored_secret("default", id, SecretKind::Password)
            .as_deref(),
        Some(APP_PASSWORD)
    );
    assert_eq!(world.bench.secrets.entry_count(), 1);
    let identities = world.run_ok(&["identity", "list", id]).await;
    assert_eq!(identities["data"].as_array().unwrap().len(), 1);
    assert_eq!(identities["data"][0]["is_default"], true);
}

/// Le secret n'est jamais rendu : ni à l'ajout, ni dans la liste, ni dans les événements, ni dans la base (inv. 15)
#[tokio::test]
async fn scenario_s2_app_password_is_never_returned_or_written_in_clear_via_cli() {
    let world = yahoo_world().await;
    world.run_stdin(&format!("{APP_PASSWORD}\n"), &ADD).await;
    world.run_ok(&["account", "list"]).await;
    world.run_ok(&["events"]).await;
    world.run_ok(&["sync", "status"]).await;
    world.assert_no_secret_anywhere(&[APP_PASSWORD]);
}

/// Un mot de passe ne passe pas en argument (code 10, jamais répété) et une authentification non proposée est refusée avant tout réseau
#[tokio::test]
async fn scenario_s2_secret_is_refused_as_argument_and_wrong_method_before_network_via_cli() {
    let world = yahoo_world().await;
    let argument = format!("--password={APP_PASSWORD}");
    let by_argument = world
        .run(&["account", "add", "--address", USER, &argument])
        .await;
    assert_eq!(by_argument.code, 10, "{}", by_argument.stderr);
    let plain = [
        "account",
        "add",
        "--address",
        USER,
        "--password-stdin",
        "--json",
    ];
    let wrong_method = world.run_stdin(&format!("{APP_PASSWORD}\n"), &plain).await;
    assert_eq!(wrong_method.code, 10, "{}", wrong_method.stderr);
    assert_eq!(world.bench.imap.connection_count(), 0);
    assert_eq!(world.bench.secrets.entry_count(), 0);
    world.assert_no_secret_anywhere(&[APP_PASSWORD]);
}

/// Mot de passe d'application refusé par le serveur : code 15, rien d'enregistré, ni compte, ni secret, ni événement d'ajout
#[tokio::test]
async fn scenario_s2_rejected_app_password_registers_nothing_via_cli() {
    let world = yahoo_world().await;
    let rejected = world.run_stdin("mauvais-mot-de-passe\n", &ADD).await;
    assert_eq!(rejected.code, 15, "{}", rejected.stderr);
    assert_eq!(
        parse_json(&rejected)["error"]["code"],
        "authentication.rejected"
    );
    assert_eq!(world.bench.secrets.entry_count(), 0);
    assert!(
        world.run_ok(&["account", "list"]).await["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let events = world.events_after(0).await;
    assert_eq!(
        cli_support::sa_events::count_of(&events, "account_added"),
        0
    );
    world.assert_no_secret_anywhere(&["mauvais-mot-de-passe"]);
}

/// Même clé : un seul compte ; plus tard, sans stdin, la synchro réutilise le secret du trousseau
#[tokio::test]
async fn scenario_s2_same_key_adds_one_account_and_the_keychain_serves_later_commands_via_cli() {
    let world = yahoo_world().await;
    let stdin = format!("{APP_PASSWORD}\n");
    let line = [&ADD[..7], &["--key", "yahoo-1", "--json"]].concat();
    let first = world.run_stdin(&stdin, &line).await;
    let again = world.run_stdin(&stdin, &line).await;
    assert_eq!(
        cli_support::account_id_of(&first),
        cli_support::account_id_of(&again)
    );
    assert_eq!(
        world.run_ok(&["account", "list"]).await["data"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        world.first_pass().await["data"]["status"]["type"],
        "finished"
    );
}
