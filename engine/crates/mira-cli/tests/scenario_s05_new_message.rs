#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S5 par le CLI : un nouveau message est lu, rattaché à son fil et annoncé une seule fois.
//! Le CLI vit le temps d'une commande : l'écoute IDLE n'est observable que pendant `sync start`.

#[allow(dead_code)]
mod cli_support;

use cli_support::sa_s4_first_sync::PASSWORD;
use cli_support::sa_s5_arrival::{arrivals, start_synced_world, sync_while_mail_arrives};
use cli_support::sa_s7_moves::message_ids_in;

/// Un courrier arrivé après la première synchro est annoncé une fois, avec le compte, la boîte et l'identifiant de la liste
#[tokio::test]
async fn scenario_s5_new_message_is_announced_once_with_its_ids_via_cli() {
    let (world, account, inbox, seen) = start_synced_world().await;
    world.deliver("INBOX", "Nouveau", "");
    world.first_pass().await;
    let announced = arrivals(&world.events_after(seen).await);
    assert_eq!(announced.len(), 1);
    let kind = &announced[0]["kind"];
    assert_eq!(kind["account"], account.as_str());
    assert_eq!(kind["mailbox"], inbox.as_str());
    let newest = message_ids_in(&world, &inbox).await.remove(0);
    assert_eq!(kind["message"], newest.as_str());
    assert_eq!(world.subjects_in(&inbox).await[0], "Nouveau");
}

/// Le nouveau message est non lu, compté, et son corps est préchargé : lisible serveur coupé
#[tokio::test]
async fn scenario_s5_new_message_is_unread_counted_and_readable_offline_via_cli() {
    let (mut world, account, inbox, _seen) = start_synced_world().await;
    world.deliver("INBOX", "Nouveau", "");
    world.first_pass().await;
    let counters = world.mailbox_view(&account, "INBOX").await;
    assert_eq!(
        (counters["total"].as_u64(), counters["unread"].as_u64()),
        (Some(4), Some(4))
    );
    world.go_offline();
    let newest = message_ids_in(&world, &inbox).await.remove(0);
    let got = world.run_ok(&["message", "get", &newest]).await;
    assert_eq!(got["data"]["flags"]["seen"], false);
    let body = world.run_ok(&["message", "body", &newest]).await;
    assert!(
        body["data"]["text"]
            .as_str()
            .unwrap()
            .contains("Corps de Nouveau")
    );
}

/// Une réponse rejoint le fil du message d'origine (D-15) ; les autres fils restent isolés
#[tokio::test]
async fn scenario_s5_reply_joins_the_existing_thread_via_cli() {
    let (world, _account, inbox, _seen) = start_synced_world().await;
    let headers = "In-Reply-To: <Alpha@example.org>\r\nReferences: <Alpha@example.org>\r\n";
    world.deliver("INBOX", "Re Alpha", headers);
    world.first_pass().await;
    let threads = world.run_ok(&["thread", "list", &inbox]).await;
    let items = threads["data"]["items"].as_array().unwrap();
    let counts: Vec<u64> = items
        .iter()
        .map(|t| t["message_count"].as_u64().unwrap())
        .collect();
    assert_eq!(items.len(), 3);
    assert_eq!(counts.iter().sum::<u64>(), 4);
    let alpha = items.iter().find(|t| t["message_count"] == 2).unwrap();
    let opened = world
        .run_ok(&["thread", "get", alpha["id"].as_str().unwrap()])
        .await;
    let shown = opened.to_string();
    assert!(
        shown.contains("Alpha") && shown.contains("Re Alpha"),
        "{shown}"
    );
}

/// Reprendre `events --after` au dernier numéro vu ne rend jamais deux fois la même arrivée (inv. 32)
#[tokio::test]
async fn scenario_s5_resuming_events_never_repeats_an_arrival_via_cli() {
    let (world, _account, _inbox, seen) = start_synced_world().await;
    world.deliver("INBOX", "Nouveau", "");
    world.first_pass().await;
    let first = world.events_after(seen).await;
    assert_eq!(arrivals(&first).len(), 1);
    let last = first.last().unwrap()["seq"].as_u64().unwrap();
    world.first_pass().await;
    let later = world.events_after(last).await;
    assert!(arrivals(&later).is_empty());
    assert!(arrivals(&world.events_after(0).await).len() == 1);
}

/// L'événement ne porte ni sujet, ni corps, ni expéditeur, ni secret (inv. 15, 32)
#[tokio::test]
async fn scenario_s5_arrival_event_carries_no_content_or_secret_via_cli() {
    let (world, _account, _inbox, seen) = start_synced_world().await;
    world.deliver("INBOX", "Confidentiel", "");
    world.first_pass().await;
    let events = world.events_after(seen).await;
    let shown = serde_json::to_string(&arrivals(&events)).unwrap();
    for forbidden in ["Confidentiel", "Corps de", "bob@example.org", PASSWORD] {
        assert!(!shown.contains(forbidden), "{forbidden}");
    }
    world.assert_no_secret_anywhere(&[PASSWORD]);
}

/// Des courriers posés à des instants variés pendant `sync start` (relevé, IDLE ou arrêt) sont chacun annoncés une fois, sans perte
#[tokio::test]
async fn scenario_s5_mail_arriving_during_a_command_is_announced_exactly_once_via_cli() {
    let (world, _account, inbox, seen) = start_synced_world().await;
    let delays = [0, 2, 5, 10, 20, 40];
    for (index, delay) in delays.into_iter().enumerate() {
        let subject = format!("Direct {index}");
        assert_eq!(sync_while_mail_arrives(&world, &subject, delay).await, 0);
    }
    world.first_pass().await;
    assert_eq!(
        arrivals(&world.events_after(seen).await).len(),
        delays.len()
    );
    let mut ids = message_ids_in(&world, &inbox).await;
    assert_eq!(ids.len(), 3 + delays.len());
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3 + delays.len());
}
