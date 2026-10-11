//! Filtre d'abonnement aux événements.

use serde::{Deserialize, Serialize};

use super::category::EventCategory;
use super::event::Event;
use crate::ids::keys::EventSeq;
use crate::ids::kinds::AccountId;

/// Filtre d'abonnement ; une liste vide signifie « tout ».
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventFilter {
    pub accounts: Vec<AccountId>,
    pub categories: Vec<EventCategory>,
    pub after: Option<EventSeq>,
}

impl EventFilter {
    /// Dit si l'événement passe le filtre ; un événement sans compte échoue au filtre par compte.
    pub fn matches(&self, event: &Event) -> bool {
        let after_ok = self.after.is_none_or(|after| event.seq > after);
        let category_ok =
            self.categories.is_empty() || self.categories.contains(&event.kind.category());
        let account_ok = self.accounts.is_empty()
            || event
                .kind
                .account()
                .is_some_and(|account| self.accounts.contains(account));
        after_ok && category_ok && account_ok
    }
}
