//! Décor du scénario S5 : compte déjà synchronisé, courrier qui arrive ensuite, arrivées en pleine commande.

use std::time::Duration;

use serde_json::Value;

use super::sa_s4_first_sync::start_first_sync_world;
use super::sa_world::SaWorld;

/// Compte synchronisé une première fois ; rend le monde, le compte, la réception et le dernier événement vu.
pub async fn start_synced_world() -> (SaWorld, String, String, u64) {
    let (world, account) = start_first_sync_world().await;
    world.first_pass().await;
    let inbox = world.mailbox_id(&account, "INBOX").await;
    let seen = world.last_event_seq().await;
    (world, account, inbox, seen)
}

/// Événements « nouveau message reçu » de la liste, dans l'ordre.
pub fn arrivals(events: &[Value]) -> Vec<Value> {
    let is_arrival = |event: &&Value| event["kind"]["type"] == "new_message_received";
    events.iter().filter(is_arrival).cloned().collect()
}

/// Lance `sync start` pendant qu'un courrier arrive après `delay_ms` ; rend le code de sortie de la commande.
pub async fn sync_while_mail_arrives(world: &SaWorld, subject: &str, delay_ms: u64) -> i32 {
    let delivery = async {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        world.deliver("INBOX", subject, "");
    };
    let (run, ()) = tokio::join!(world.run_sync(&["sync", "start", "--json"]), delivery);
    run.code
}
