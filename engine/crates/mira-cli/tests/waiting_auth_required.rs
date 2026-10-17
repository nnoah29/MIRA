#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Attente `--wait` sur un compte à réauthentifier : sortie immédiate en code 15, l'action reste en file (S10, 7.7).

#[allow(dead_code)]
mod cli_support;

use cli_support::sb_world::SbWorld;
use cli_support::{CliResult, parse_json};
use mira_testkit::imap::CapabilitySet;

const USER: &str = "alice@folders.test";

/// Compte à mot de passe dont le serveur refuse désormais le mot de passe rangé ; rend le monde, le compte et deux messages.
async fn rejected_password_world() -> (SbWorld, String, String, String) {
    let world = SbWorld::start(&[USER], CapabilitySet::full()).await;
    world.deliver_subject(USER, "INBOX", "Alpha");
    world.deliver_subject(USER, "INBOX", "Beta");
    let account = world.add_account(USER).await;
    world.run_sync_ok(&["sync", "start"]).await;
    let inbox = world.mailbox_named(&account, "INBOX").await;
    let alpha = world.message_id_of(&inbox, "Alpha").await;
    let beta = world.message_id_of(&inbox, "Beta").await;
    world.credentials.add_password(USER, "nouveau-mdp-9911");
    (world, account, alpha, beta)
}

/// Pose une action « drapeau » avec `--wait`, l'horloge avançant seule, et rend le résultat.
async fn flag_and_wait(world: &SbWorld, message: &str) -> CliResult {
    let pump = world.pump_clock();
    let line = [
        "action",
        "apply",
        "flag",
        "--message",
        message,
        "--wait",
        "--timeout",
        "30",
        "--json",
    ];
    let result = world.run(&line).await;
    pump.abort();
    result
}

/// `action apply --wait` : le mot de passe est refusé pendant l'attente, sortie en 15 « réauthentifier », action en file
#[tokio::test]
async fn apply_wait_exits_with_the_authentication_code_when_the_account_needs_reauthentication() {
    let (world, account, alpha, _beta) = rejected_password_world().await;
    let started = std::time::Instant::now();
    let waited = flag_and_wait(&world, &alpha).await;
    assert_eq!(waited.code, 15, "{}", waited.stderr);
    assert!(started.elapsed().as_secs() < 20, "attente jusqu'au délai");
    let error = &parse_json(&waited)["error"];
    assert_eq!(error["category"], "authentication");
    assert_eq!(error["suggested_action"], "reauthenticate");
    let queued = world.queued_actions(&account).await;
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0]["state"], "pending");
}

/// Compte déjà à réauthentifier : `mailbox create --wait` rend 15 sans attendre le délai, l'action reste en file
#[tokio::test]
async fn mailbox_create_wait_exits_at_once_when_the_account_is_already_auth_required() {
    let (world, account, alpha, _beta) = rejected_password_world().await;
    assert_eq!(flag_and_wait(&world, &alpha).await.code, 15);
    let listed = world.run_ok(&["account", "list"]).await;
    assert_eq!(listed["data"][0]["state"]["type"], "auth_required");
    let line = [
        "mailbox",
        "create",
        &account,
        "Projets",
        "--wait",
        "--timeout",
        "30",
    ];
    let started = std::time::Instant::now();
    let created = world.run(&line).await;
    assert_eq!(created.code, 15, "{}", created.stderr);
    assert!(started.elapsed().as_secs() < 10, "attente jusqu'au délai");
    let queued = world.queued_actions(&account).await;
    assert_eq!(queued.len(), 2);
    assert!(queued.iter().all(|action| action["state"] == "pending"));
}
