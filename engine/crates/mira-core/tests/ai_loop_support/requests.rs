//! Dialogue avec le faux serveur d'IA : file des tours, corps des requêtes reçues, corps d'un message de la boîte.
#![allow(dead_code)]

use mira_testkit::ai::Behavior;
use mira_vocabulary::MessageId;
use serde_json::Value;

use super::fixture::LoopFixture;
use crate::support_read::seed::add_body;

impl LoopFixture {
    /// Met un comportement en file pour la prochaine requête du modèle.
    pub fn queue(&self, behavior: Behavior) {
        self.server.queue(behavior);
    }

    /// Corps JSON de chaque requête reçue par le faux serveur.
    pub fn bodies(&self) -> Vec<Value> {
        let bodies = self.server.requests().into_iter().map(|r| r.body);
        bodies
            .map(|text| serde_json::from_str(&text).unwrap())
            .collect()
    }

    /// Remplace le corps indexé d'un message de la boîte.
    pub fn set_body(&self, message: &str, text: &str) {
        add_body(&self.db, &MessageId::from_raw(message), text, None);
    }
}
