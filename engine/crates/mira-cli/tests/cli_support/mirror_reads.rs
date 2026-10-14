//! Lecture du miroir par les commandes du CLI, et courrier déposé sur le faux serveur.

use mira_assembly::mira_vocabulary::{AccountId, ProfileId, SecretKind, SecretRef};
use mira_assembly::ports::SecretStore;

use super::full_bench::{FullBench, USER};

impl FullBench {
    /// Sujets des messages d'une boîte du miroir, lus par `message list`.
    pub async fn subjects_of(&self, mailbox: &str) -> Vec<String> {
        let args = ["message", "list", "--mailbox", mailbox, "--limit", "50"];
        let page = self.run_ok(&args).await;
        let items = page["data"]["items"].as_array().unwrap();
        items
            .iter()
            .map(|item| item["subject"].as_str().unwrap_or_default().to_string())
            .collect()
    }

    /// Identifiant de la boîte de ce nom dans le miroir du compte.
    pub async fn mailbox_named(&self, account: &str, name: &str) -> String {
        let data = self.run_ok(&["mailbox", "list", account]).await;
        let boxes = data["data"].as_array().unwrap();
        let found = boxes.iter().find(|item| item["name"] == name).unwrap();
        found["id"].as_str().unwrap().to_string()
    }

    /// Retire le mot de passe du compte du trousseau, comme si l'utilisateur l'avait effacé.
    pub fn forget_password(&self, account: &str) {
        let key = SecretRef {
            profile: ProfileId::from_raw("default"),
            account: AccountId::from_raw(account),
            kind: SecretKind::Password,
        };
        self.secrets.delete(&key).unwrap();
    }

    /// Dépose un courrier sur le serveur de l'utilisateur de test, dans la boîte donnée.
    pub fn deliver_subject(&self, mailbox: &str, subject: &str) {
        let id = subject.replace(' ', "-");
        let raw = format!(
            "From: Bob <bob@example.org>\r\nTo: {USER}\r\nSubject: {subject}\r\nMessage-ID: <{id}@example.org>\r\nDate: Mon, 01 Jan 2024 10:00:00 +0000\r\n\r\nCorps de {subject}\r\n"
        );
        self.store.deliver(USER, mailbox, raw.as_bytes(), &[]);
    }
}
