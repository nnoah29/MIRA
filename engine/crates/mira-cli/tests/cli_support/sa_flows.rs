//! Gestes d'utilisateur répétés par les scénarios : ajouter le compte, lire boîtes, messages et événements.

use serde_json::Value;

use super::sa_world::SaWorld;
use super::{CliResult, account_id_of, parse_json};

impl SaWorld {
    /// Lance une commande avec une entrée standard vide (synchro de fond coupée).
    pub async fn run(&self, arguments: &[&str]) -> CliResult {
        self.run_stdin("", arguments).await
    }

    /// Ajoute le compte de l'utilisateur (secret sur l'entrée standard) et rend son identifiant.
    pub async fn add_account(&self, extra: &[&str]) -> String {
        let mut line = vec!["account", "add", "--address", &self.user];
        line.extend_from_slice(&["--password-stdin", "--json"]);
        line.extend_from_slice(extra);
        let stdin = format!("{}\n", self.password);
        let added = self.run_stdin(&stdin, &line).await;
        assert_eq!(added.code, 0, "{}", added.stderr);
        account_id_of(&added)
    }

    /// Identifiant de la boîte de ce nom dans le miroir du compte.
    pub async fn mailbox_id(&self, account: &str, name: &str) -> String {
        let data = self.run_ok(&["mailbox", "list", account]).await;
        let found = data["data"].as_array().unwrap().iter();
        let found = found.into_iter().find(|item| item["name"] == name);
        found.unwrap()["id"].as_str().unwrap().to_string()
    }

    /// Sujets des messages d'une boîte (plus récent d'abord), lus par `message list`.
    pub async fn subjects_in(&self, mailbox: &str) -> Vec<String> {
        let page = self
            .run_ok(&["message", "list", "--mailbox", mailbox, "--limit", "100"])
            .await;
        let items = page["data"]["items"].as_array().unwrap();
        let subject = |item: &Value| item["subject"].as_str().unwrap_or_default().to_string();
        items.iter().map(subject).collect()
    }

    /// Événements dont le numéro dépasse `after`, tels que rendus par `mira events`.
    pub async fn events_after(&self, after: u64) -> Vec<Value> {
        let after = after.to_string();
        let data = self
            .run_ok(&["events", "--after", &after, "--limit", "1000"])
            .await;
        data["data"].as_array().unwrap().clone()
    }

    /// Numéro du dernier événement déjà émis (0 s'il n'y en a pas).
    pub async fn last_event_seq(&self) -> u64 {
        let result = self
            .run_stdin("", &["events", "--limit", "1000", "--json"])
            .await;
        let events = parse_json(&result)["data"]["data"].clone();
        events
            .as_array()
            .unwrap()
            .last()
            .map_or(0, |e| e["seq"].as_u64().unwrap())
    }
}
