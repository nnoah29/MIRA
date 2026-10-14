#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S9 par le CLI : envoi avec annulation pendant le délai, identité choisie, copie dans Envoyés (D-26, inv. 26).

#[allow(dead_code)]
mod cli_support;

use cli_support::sb_events::{events_of_type, first_seq_of_type};
use cli_support::sb_leaks::assert_outputs_clean;
use cli_support::sb_world::{SB_PASSWORD, SbWorld};
use mira_testkit::imap::CapabilitySet;

const USER: &str = "alice@folders.test";
const GMAIL_USER: &str = "alice@gmail.com";

/// Monde dont le compte est ajouté et synchronisé (boîtes connues) ; rend le monde et l'identifiant du compte.
async fn world_with_account(user: &str, capabilities: CapabilitySet) -> (SbWorld, String) {
    let world = SbWorld::start(&[user], capabilities).await;
    let sent_box = "[Gmail]/Sent Mail";
    world.store.create_mailbox(user, sent_box, Some("\\Sent"));
    let account = world.add_account(user).await;
    world.run_sync_ok(&["sync", "start"]).await;
    (world, account)
}

/// Brouillon prêt à partir (destinataire, objet, corps) avec l'identité donnée, ou celle du compte.
async fn ready_draft(world: &SbWorld, account: &str, identity: Option<&str>) -> String {
    let mut create = vec!["draft", "create", account];
    create.extend(identity.iter().flat_map(|id| ["--identity", *id]));
    let created = world.run_ok(&create).await;
    let draft = created["data"]["data"]["id"].as_str().unwrap().to_string();
    let mut update = vec!["draft", "update", &draft, "--to", "bob@example.org"];
    update.extend(["--subject", "Bonjour Bob", "--body", "Texte"]);
    world.run_ok(&update).await;
    draft
}

/// Annulation pendant le délai : rien ne part, le brouillon redevient éditable ; renvoi : parti, copie dans Envoyés
#[tokio::test]
async fn scenario_s9_cancel_during_delay_then_send_via_cli() {
    let (world, account) = world_with_account(USER, CapabilitySet::full()).await;
    let draft = ready_draft(&world, &account, None).await;
    let first = world.run_ok(&["draft", "send", &draft, "--key", "a"]).await;
    let action = first["data"]["data"].as_str().unwrap().to_string();
    let scheduled = world.run_ok(&["draft", "get", &draft]).await;
    assert_eq!(scheduled["data"]["data"]["state"]["type"], "scheduled");
    let outbox = world.run_ok(&["draft", "outbox", &account]).await;
    assert_eq!(outbox["data"]["data"][0]["state"], "pending");
    world.run_ok(&["draft", "cancel-send", &action]).await;
    let editing = world.run_ok(&["draft", "get", &draft]).await;
    assert_eq!(editing["data"]["data"]["state"]["type"], "editing");
    world.clock.advance_millis(11_000);
    let emptied = world.run_ok(&["draft", "outbox", &account]).await;
    assert_eq!(emptied["data"]["data"], serde_json::json!([]));
    assert!(world.smtp.received().is_empty());
    let pump = world.pump_clock();
    let line = ["draft", "send", &draft, "--key", "b", "--wait"];
    let second = world.run(&line).await;
    pump.abort();
    assert_eq!(second.code, 0, "{}", second.stderr);
    assert_eq!(world.smtp.received().len(), 1);
    assert_eq!(world.store.messages(USER, "Sent").len(), 1);
    assert_eq!(world.run(&["draft", "get", &draft]).await.code, 11);
}

/// Événements d'envoi dans l'ordre, boîte d'envoi vide, aucun secret dans les sorties ni dans le profil
#[tokio::test]
async fn scenario_s9_events_outbox_and_secrecy_via_cli() {
    let (world, account) = world_with_account(USER, CapabilitySet::full()).await;
    let draft = ready_draft(&world, &account, None).await;
    let pump = world.pump_clock();
    let sent = world
        .run(&["draft", "send", &draft, "--wait", "--json"])
        .await;
    pump.abort();
    assert_eq!(sent.code, 0, "{}", sent.stderr);
    let events = world.events().await;
    let scheduled = first_seq_of_type(&events, "send_scheduled").unwrap();
    let departed = first_seq_of_type(&events, "send_departed").unwrap();
    assert!(scheduled < departed);
    assert_eq!(events_of_type(&events, "send_departed").len(), 1);
    let outbox = world.run(&["draft", "outbox", &account, "--json"]).await;
    let listed = cli_support::parse_json(&outbox);
    let listed = listed["data"].clone();
    assert_eq!(listed["data"]["data"], serde_json::json!([]));
    assert_outputs_clean(&[&sent, &outbox], SB_PASSWORD);
    assert!(!world.profile_contains(SB_PASSWORD));
}

/// L'identité choisie à la création signe l'envoi : expéditeur SMTP et en-tête From
#[tokio::test]
async fn scenario_s9_send_uses_the_chosen_identity_via_cli() {
    let (world, account) = world_with_account(USER, CapabilitySet::full()).await;
    let mut add = vec!["identity", "add", &account];
    add.extend(["--address", "alias@folders.test", "--display-name", "Alias"]);
    let added = world.run_ok(&add).await;
    let identity = added["data"]["id"].as_str().unwrap().to_string();
    let draft = ready_draft(&world, &account, Some(&identity)).await;
    let pump = world.pump_clock();
    let sent = world.run(&["draft", "send", &draft, "--wait"]).await;
    pump.abort();
    assert_eq!(sent.code, 0, "{}", sent.stderr);
    let mail = &world.smtp.received()[0];
    assert_eq!(mail.from, "alias@folders.test");
    let text = String::from_utf8_lossy(&mail.data).into_owned();
    assert!(text.contains("<alias@folders.test>"), "{text}");
}

/// Un service qui garde lui-même la copie n'en reçoit pas une seconde : Envoyés reste vide
#[tokio::test]
async fn scenario_s9_server_saved_copy_is_not_duplicated_via_cli() {
    let capabilities = CapabilitySet::full().with_label_extensions();
    let (world, account) = world_with_account(GMAIL_USER, capabilities).await;
    let boxes = world.run_ok(&["mailbox", "list", &account]).await;
    let roles: Vec<&str> = boxes["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["role"].as_str())
        .collect();
    assert!(roles.contains(&"sent"), "{roles:?}");
    let draft = ready_draft(&world, &account, None).await;
    let pump = world.pump_clock();
    let sent = world.run(&["draft", "send", &draft, "--wait"]).await;
    pump.abort();
    assert_eq!(sent.code, 0, "{}", sent.stderr);
    assert_eq!(world.smtp.received().len(), 1);
    assert!(
        world
            .store
            .messages(GMAIL_USER, "[Gmail]/Sent Mail")
            .is_empty()
    );
    assert!(world.store.messages(GMAIL_USER, "Sent").is_empty());
}
