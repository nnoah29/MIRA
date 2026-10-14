//! Décor du scénario S7 : quatre courriers en réception, trois déplacements hors ligne, rejeu au retour.

use mira_testkit::imap::CapabilitySet;
use serde_json::Value;

use super::sa_world::SaWorld;

pub const SUBJECTS: [&str; 4] = ["Alpha", "Beta", "Gamma", "Delta"];

/// Compte synchronisé dont la réception porte quatre courriers et qui a une boîte Archive.
pub struct MoveWorld {
    pub world: SaWorld,
    pub account: String,
    pub inbox: String,
    pub archive: String,
    pub ids: Vec<String>,
}

/// Identifiants des messages d'une boîte, plus récent d'abord, lus par `message list`.
pub async fn message_ids_in(world: &SaWorld, mailbox: &str) -> Vec<String> {
    let args = ["message", "list", "--mailbox", mailbox, "--limit", "100"];
    let page = world.run_ok(&args).await;
    let items = page["data"]["items"].as_array().unwrap();
    items
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_string())
        .collect()
}

/// Prépare le compte, le serveur et la première synchro ; les messages sont lus en réception.
pub async fn start_move_world(capabilities: CapabilitySet) -> MoveWorld {
    let world = SaWorld::start("alice@folders.test", "pa55-SECRET-xyz", capabilities).await;
    let store = &world.bench.store;
    store.create_mailbox(&world.user, "Archive", Some("\\Archive"));
    for subject in SUBJECTS {
        world.deliver("INBOX", subject, "");
    }
    let account = world.add_account(&[]).await;
    world.run_sync_ok(&["sync", "start"]).await;
    let inbox = world.mailbox_id(&account, "INBOX").await;
    let archive = world.mailbox_id(&account, "Archive").await;
    let ids = message_ids_in(&world, &inbox).await;
    MoveWorld {
        world,
        account,
        inbox,
        archive,
        ids,
    }
}

impl MoveWorld {
    /// Coupe le réseau puis déplace les trois premiers messages vers Archive (clés k0, k1, k2).
    pub async fn move_three_offline(&mut self) -> Vec<String> {
        self.world.go_offline();
        let mut actions = Vec::new();
        for (index, id) in self.ids.iter().take(3).enumerate() {
            let key = format!("k{index}");
            let line = ["action", "apply", "move", "--message", id, "--to"];
            let line = [&line[..], &[&self.archive, "--key", &key]].concat();
            let answer = self.world.run_ok(&line).await;
            actions.push(answer["data"]["data"][0].as_str().unwrap().to_string());
        }
        actions
    }

    /// Rétablit le réseau puis soumet une dernière action et attend sa fin : la file passe avant elle.
    pub async fn reconnect_and_replay(&mut self) {
        self.world.go_online();
        let pump = self.world.bench.pump_clock();
        let last = self.ids[3].as_str();
        let line = ["action", "apply", "mark-read", "--message", last, "--wait"];
        self.world.run_ok(&line).await;
        pump.abort();
    }

    /// Textes `clé:état` des actions de la file, dans l'ordre de la file.
    pub async fn queue_view(&self) -> Vec<String> {
        let listed = self.world.run_ok(&["queue", "list", &self.account]).await;
        let text = |action: &Value, field: &str| action[field].as_str().unwrap().to_string();
        let actions = listed["data"]["data"].as_array().unwrap();
        actions
            .iter()
            .map(|a| format!("{}:{}", text(a, "idempotency_key"), text(a, "state")))
            .collect()
    }
}
