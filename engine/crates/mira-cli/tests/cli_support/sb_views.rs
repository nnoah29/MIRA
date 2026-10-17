//! Lectures par commandes du CLI sur le banc des scénarios : boîtes, sujets, file d'actions, événements.

use serde_json::Value;

use super::parse_json;
use super::sb_world::SbWorld;

impl SbWorld {
    /// Lance une commande de synchro avec `--json`, exige le code 0 et rend la réponse du contrat.
    pub async fn run_sync_ok(&self, arguments: &[&str]) -> Value {
        let mut full: Vec<&str> = arguments.to_vec();
        full.push("--json");
        let result = self.run_sync(&full).await;
        assert_eq!(result.code, 0, "{arguments:?} : {}", result.stderr);
        parse_json(&result)["data"].clone()
    }

    /// Identifiant de la boîte de ce nom dans le miroir du compte.
    pub async fn mailbox_named(&self, account: &str, name: &str) -> String {
        let data = self.run_ok(&["mailbox", "list", account]).await;
        let boxes = data["data"].as_array().unwrap();
        let found = boxes.iter().find(|item| item["name"] == name).unwrap();
        found["id"].as_str().unwrap().to_string()
    }

    /// Messages d'une boîte du miroir (`message list`), du plus récent au plus ancien.
    pub async fn messages_of(&self, mailbox: &str) -> Vec<Value> {
        let args = ["message", "list", "--mailbox", mailbox, "--limit", "50"];
        let page = self.run_ok(&args).await;
        page["data"]["items"].as_array().unwrap().clone()
    }

    /// Sujets des messages d'une boîte du miroir, triés.
    pub async fn subjects_of(&self, mailbox: &str) -> Vec<String> {
        let mut subjects: Vec<String> = self
            .messages_of(mailbox)
            .await
            .iter()
            .map(|item| item["subject"].as_str().unwrap_or_default().to_string())
            .collect();
        subjects.sort();
        subjects
    }

    /// Actions de la file du compte (`queue list`), tous états confondus.
    pub async fn queued_actions(&self, account: &str) -> Vec<Value> {
        let data = self.run_ok(&["queue", "list", account]).await;
        data["data"]["data"].as_array().unwrap().clone()
    }

    /// Événements du moteur (`events`), sous la forme rendue par le contrat.
    pub async fn events(&self) -> Vec<Value> {
        let data = self.run_ok(&["events", "--limit", "1000"]).await;
        data["data"].as_array().unwrap().clone()
    }
}
