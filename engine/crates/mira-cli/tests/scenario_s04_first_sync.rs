#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S4 par le CLI : première synchronisation, réception d'abord, plage de 90 jours, repères, progression.

#[allow(dead_code)]
mod cli_support;

use cli_support::sa_events::{count_of, first_position, progress_of, seqs_strictly_increase};
use cli_support::sa_s4_first_sync::{PASSWORD, start_first_sync_world};

/// Avant la synchro le miroir est vide ; après, boîtes et rôles viennent des attributs du serveur
#[tokio::test]
async fn scenario_s4_first_sync_fills_mailboxes_with_roles_via_cli() {
    let (world, account) = start_first_sync_world().await;
    let before = world.run_ok(&["mailbox", "list", &account]).await;
    assert!(before["data"].as_array().unwrap().is_empty());
    let finished = world.first_pass().await;
    assert_eq!(finished["data"]["status"]["type"], "finished");
    let after = world.run_ok(&["mailbox", "list", &account]).await;
    let roles: Vec<(&str, &str)> = after["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| (b["name"].as_str().unwrap(), b["role"].as_str().unwrap()))
        .collect();
    let expected = [("Archive", "archive"), ("INBOX", "inbox")];
    assert_eq!(roles[..2], expected);
    assert_eq!(roles[2..], [("Sent", "sent"), ("Trash", "trash")]);
}

/// Seuls les 90 derniers jours de la réception sont téléchargés (D-21) ; les corps récents sont préchargés
#[tokio::test]
async fn scenario_s4_first_sync_keeps_the_ninety_day_window_via_cli() {
    let (world, account) = start_first_sync_world().await;
    world.first_pass().await;
    let inbox = world.mailbox_id(&account, "INBOX").await;
    let mut subjects = world.subjects_in(&inbox).await;
    subjects.sort();
    assert_eq!(subjects, ["Alpha", "Beta", "Gamma"]);
    let counted = world.mailbox_view(&account, "INBOX").await;
    assert_eq!(
        (counted["total"].as_u64(), counted["unread"].as_u64()),
        (Some(3), Some(3))
    );
    let accounts = world.run_ok(&["account", "list"]).await;
    assert_eq!(accounts["data"][0]["sync"]["window_days"], 90);
    let first = cli_support::sa_s7_moves::message_ids_in(&world, &inbox)
        .await
        .remove(0);
    let body = world.run_ok(&["message", "body", &first]).await;
    assert!(
        body["data"]["text"]
            .as_str()
            .unwrap()
            .starts_with("Corps de")
    );
}

/// Événements : réception d'abord, progression des en-têtes puis des corps, fin unique, jamais « nouveau message » (inv. 32)
#[tokio::test]
async fn scenario_s4_first_sync_emits_ordered_progress_events_via_cli() {
    let (world, _account) = start_first_sync_world().await;
    let seen = world.last_event_seq().await;
    world.first_pass().await;
    let events = world.events_after(seen).await;
    let headers = progress_of(&events, "headers");
    assert_eq!(headers.first(), Some(&(0, 3)));
    assert!(headers.contains(&(3, 3)) && headers.contains(&(1, 1)));
    assert!(!progress_of(&events, "mailboxes").is_empty());
    assert_eq!(progress_of(&events, "bodies").last(), Some(&(3, 3)));
    assert_eq!(count_of(&events, "body_available"), 3);
    assert_eq!(count_of(&events, "new_message_received"), 0);
    let started = first_position(&events, "sync_started").unwrap();
    assert!(started < first_position(&events, "sync_finished").unwrap());
    assert!(seqs_strictly_increase(&events));
}

/// Les repères des boîtes valent ceux du serveur une fois les données écrites (D-22, inv. 20)
#[tokio::test]
async fn scenario_s4_mailbox_markers_match_the_server_via_cli() {
    let (world, account) = start_first_sync_world().await;
    world.first_pass().await;
    for name in ["INBOX", "Sent", "Archive", "Trash"] {
        let marker = world.mailbox_view(&account, name).await["marker"].clone();
        let store = &world.bench.store;
        let validity = store.uidvalidity(&world.user, name).unwrap();
        let modseq = store.highest_modseq(&world.user, name).unwrap();
        assert_eq!(marker["uid_validity"], validity, "{name}");
        assert_eq!(marker["highest_modseq"], modseq, "{name}");
    }
}

/// Une seconde passe est incrémentale : mêmes messages et mêmes identifiants, aucun en-tête relu, aucun événement de message
#[tokio::test]
async fn scenario_s4_second_pass_is_incremental_and_changes_nothing_via_cli() {
    let (world, account) = start_first_sync_world().await;
    world.first_pass().await;
    let inbox = world.mailbox_id(&account, "INBOX").await;
    let ids = cli_support::sa_s7_moves::message_ids_in(&world, &inbox).await;
    let seen = world.last_event_seq().await;
    let logged = world.bench.store.command_log().len();
    world.first_pass().await;
    assert_eq!(
        cli_support::sa_s7_moves::message_ids_in(&world, &inbox).await,
        ids
    );
    let log = world.bench.store.command_log();
    assert!(log[logged..].iter().any(|line| line.contains("QRESYNC")));
    assert!(
        !log[logged..]
            .iter()
            .any(|line| line.contains("BODY.PEEK[HEADER]"))
    );
    let events = world.events_after(seen).await;
    assert_eq!(
        count_of(&events, "new_message_received") + count_of(&events, "message_updated"),
        0
    );
}

/// `account add --sync-wait` lance la première synchro après l'ajout ; ni les sorties ni le profil ne contiennent le secret (inv. 15)
#[tokio::test]
async fn scenario_s4_add_account_with_sync_wait_runs_the_first_sync_via_cli() {
    let world = cli_support::sa_world::SaWorld::start(
        "alice@folders.test",
        PASSWORD,
        mira_testkit::imap::CapabilitySet::full(),
    )
    .await;
    world.deliver("INBOX", "Alpha", "");
    let account = world.add_account_waiting_first_sync().await;
    let inbox = world.mailbox_id(&account, "INBOX").await;
    assert_eq!(world.subjects_in(&inbox).await, ["Alpha"]);
    let events = world.events_after(0).await;
    let added = first_position(&events, "account_added").unwrap();
    assert!(added < first_position(&events, "sync_started").unwrap());
    assert!(first_position(&events, "sync_finished").is_some());
    world.assert_no_secret_anywhere(&[PASSWORD]);
}
