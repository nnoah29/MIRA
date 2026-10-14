#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S3 par le CLI : compte « Autre », serveurs saisis, test de connexion sans rien enregistrer.

#[allow(dead_code)]
mod cli_support;

use cli_support::sa_events::count_of;
use cli_support::sa_s3_manual::{OTHER_PASSWORD, OTHER_USER, start_other_world};
use cli_support::sa_world::closed_port;
use cli_support::{account_id_of, parse_json};
use mira_assembly::mira_vocabulary::SecretKind;

/// Domaine inconnu : provider de repli, serveurs `imap.<domaine>` et `smtp.<domaine>` proposés, deux authentifications (D-05)
#[tokio::test]
async fn scenario_s3_unknown_domain_falls_back_to_the_generic_provider_via_cli() {
    let world = start_other_world().await;
    let detected = world.run_ok(&["provider", "detect", OTHER_USER]).await;
    assert_eq!(detected["provider"]["id"], "generic");
    assert_eq!(detected["servers"]["imap"]["host"], "imap.other.test");
    assert_eq!(detected["servers"]["smtp"]["host"], "smtp.other.test");
    let methods = detected["provider"]["auth_methods"].as_array().unwrap();
    assert_eq!(methods.len(), 2);
    let prepared = world
        .run_ok(&["account", "prepare", "--address", OTHER_USER])
        .await;
    assert_eq!(prepared["data"]["servers"], detected["servers"]);
}

/// Le test de connexion contacte le serveur saisi mais n'enregistre ni compte, ni secret, ni événement d'ajout
#[tokio::test]
async fn scenario_s3_connection_test_registers_nothing_via_cli() {
    let world = start_other_world().await;
    let port = world.bench.imap.port();
    let tested = world
        .account_with_servers("test", port, OTHER_PASSWORD)
        .await;
    assert_eq!(tested.code, 0, "{}", tested.stderr);
    assert_eq!(parse_json(&tested)["data"]["type"], "done");
    assert!(world.bench.imap.connection_count() >= 1);
    assert_eq!(world.bench.secrets.entry_count(), 0);
    assert!(
        world.run_ok(&["account", "list"]).await["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(count_of(&world.events_after(0).await, "account_added"), 0);
}

/// À l'ajout, les serveurs saisis sont rangés dans le compte et servent ensuite sans être redonnés (7.9)
#[tokio::test]
async fn scenario_s3_added_account_keeps_the_typed_servers_via_cli() {
    let world = start_other_world().await;
    let port = world.bench.imap.port();
    let added = world
        .account_with_servers("add", port, OTHER_PASSWORD)
        .await;
    assert_eq!(added.code, 0, "{}", added.stderr);
    let id = account_id_of(&added);
    let account = &parse_json(&added)["data"]["data"];
    assert_eq!(account["provider"], "generic");
    assert_eq!(account["servers"]["imap"]["port"], port);
    assert_eq!(account["servers"]["imap"]["security"], "plaintext");
    let listed = world.run_ok(&["account", "list"]).await;
    assert_eq!(listed["data"][0]["servers"], account["servers"]);
    assert_eq!(
        world
            .stored_secret("default", &id, SecretKind::Password)
            .as_deref(),
        Some(OTHER_PASSWORD)
    );
    assert_eq!(
        world.first_pass().await["data"]["status"]["type"],
        "finished"
    );
    world.assert_no_secret_anywhere(&[OTHER_PASSWORD]);
}

/// Un serveur inatteignable ou un seul hôte saisi sont refusés (13, 10) sans rien garder ; la saisie corrigée réussit
#[tokio::test]
async fn scenario_s3_wrong_server_is_refused_then_the_corrected_one_is_added_via_cli() {
    let world = start_other_world().await;
    let wrong = world
        .account_with_servers("add", closed_port(), OTHER_PASSWORD)
        .await;
    assert_eq!(wrong.code, 13, "{}", wrong.stderr);
    assert_eq!(world.bench.secrets.entry_count(), 0);
    let one_host = world.account_with_lone_imap_host(OTHER_PASSWORD).await;
    assert_eq!(one_host.code, 10, "{}", one_host.stderr);
    assert_eq!(world.account_count().await, 0);
    let port = world.bench.imap.port();
    let fixed = world
        .account_with_servers("add", port, OTHER_PASSWORD)
        .await;
    assert_eq!(fixed.code, 0, "{}", fixed.stderr);
    assert_eq!(world.bench.secrets.entry_count(), 1);
}

/// Mot de passe refusé sur des serveurs saisis : code 15, ni compte ni secret ; le mot de passe ne fuit nulle part
#[tokio::test]
async fn scenario_s3_rejected_password_on_typed_servers_registers_nothing_via_cli() {
    let world = start_other_world().await;
    let port = world.bench.imap.port();
    let rejected = world
        .account_with_servers("add", port, "pas-le-bon-4471")
        .await;
    assert_eq!(rejected.code, 15, "{}", rejected.stderr);
    assert_eq!(world.bench.secrets.entry_count(), 0);
    assert!(
        world.run_ok(&["account", "list"]).await["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    world.assert_no_secret_anywhere(&["pas-le-bon-4471", OTHER_PASSWORD]);
}
