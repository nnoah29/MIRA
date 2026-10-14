//! Attente active d'une condition produite par les tâches de fond du moteur.

use std::time::Duration;

use mira_vocabulary::{AccountState, ActionId, ActionState, MessageId, MessageSummary};

use super::fixture::Fixture;

/// Attend (15 s au plus) que la condition devienne vraie
pub async fn eventually(what: &str, mut check: impl FnMut() -> bool) {
    for _ in 0..600 {
        if check() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("délai dépassé : {what}");
}

impl Fixture {
    /// État d'une action relu en base
    pub fn state_of(&self, action: &ActionId) -> ActionState {
        let found = self.db.read(|tx| tx.get_action(action)).unwrap();
        found.unwrap().state
    }

    /// Attend que l'action atteigne cet état
    pub async fn wait_for_state(&self, action: &ActionId, wanted: ActionState) {
        let what = format!("action {} à l'état {wanted:?}", action.as_str());
        eventually(&what, || self.state_of(action) == wanted).await;
    }

    /// Message du miroir relu en base
    pub fn message(&self, id: &str) -> MessageSummary {
        let found = self.db.read(|tx| tx.get_message(&MessageId::from_raw(id)));
        found.unwrap().unwrap()
    }

    /// État de connexion du compte relu en base
    pub fn account_state(&self) -> AccountState {
        let found = self.db.read(|tx| tx.get_account(&self.account));
        found.unwrap().unwrap().state
    }

    /// Attend que le compte atteigne cet état de connexion
    pub async fn wait_for_account(&self, wanted: AccountState) {
        let what = format!("compte à l'état {wanted:?}");
        eventually(&what, || self.account_state() == wanted).await;
    }
}
