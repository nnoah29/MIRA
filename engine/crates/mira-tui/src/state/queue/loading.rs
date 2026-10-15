//! Tours de lecture de la file : une réponse ne compte que pour le compte et le tour attendus.

use mira_assembly::mira_vocabulary::AccountId;

use super::QueueState;

/// Lecture qui compose la vue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueList {
    Outbox,
    Actions,
}

impl QueueState {
    /// Commence un tour de lecture et rend son numéro ; pendant une lecture, note seulement qu'il en faudra un autre.
    pub fn start_round(&mut self) -> Option<u64> {
        if !self.is_open() {
            return None;
        }
        if self.is_loading() {
            self.again = true;
            return None;
        }
        self.round += 1;
        self.waiting_outbox = true;
        self.waiting_actions = true;
        Some(self.round)
    }

    /// Vrai si cette lecture de ce compte et de ce tour est encore attendue.
    pub fn awaits(&self, list: QueueList, account: &AccountId, round: u64) -> bool {
        let waiting = match list {
            QueueList::Outbox => self.waiting_outbox,
            QueueList::Actions => self.waiting_actions,
        };
        waiting && round == self.round && self.account.as_ref() == Some(account)
    }

    /// Note l'arrivée de cette lecture ; vrai si toutes sont arrivées et qu'un tour de plus a été demandé entre-temps.
    pub fn finish(&mut self, list: QueueList) -> bool {
        match list {
            QueueList::Outbox => self.waiting_outbox = false,
            QueueList::Actions => self.waiting_actions = false,
        }
        if self.is_loading() {
            return false;
        }
        self.loaded = true;
        std::mem::take(&mut self.again)
    }

    /// Vrai tant qu'une lecture du tour courant est attendue.
    pub fn is_loading(&self) -> bool {
        self.waiting_outbox || self.waiting_actions
    }

    /// Vrai une fois qu'un tour complet est arrivé.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}
