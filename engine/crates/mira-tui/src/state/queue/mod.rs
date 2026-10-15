//! Vue « boîte d'envoi et file d'actions » d'un compte : fiches lues, sélection, lectures en cours.

pub mod anchor;
pub mod entries;
pub mod loading;

use mira_assembly::api::OutboxEntry;
use mira_assembly::mira_vocabulary::{AccountId, ActionId, PendingAction};

pub use entries::{QueueEntry, QueueItem};
pub use loading::QueueList;

use super::selection::Selection;

/// Vue de la file ; fermée tant qu'aucun compte n'y est ouvert. Le tour de lecture ne revient jamais en arrière.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueueState {
    account: Option<AccountId>,
    outbox: Vec<OutboxEntry>,
    actions: Vec<PendingAction>,
    entries: Vec<QueueEntry>,
    pub selection: Selection,
    anchor: Option<ActionId>,
    round: u64,
    waiting_outbox: bool,
    waiting_actions: bool,
    again: bool,
    loaded: bool,
}

impl QueueState {
    /// Ouvre la vue, vide, sur ce compte.
    pub fn open(&mut self, account: AccountId) {
        self.open_none();
        self.account = Some(account);
    }

    /// Ferme la vue ; les réponses encore attendues seront ignorées.
    pub fn close(&mut self) {
        self.open_none();
    }

    /// Vrai si la vue est ouverte.
    pub fn is_open(&self) -> bool {
        self.account.is_some()
    }

    /// Compte de la vue ouverte.
    pub fn account(&self) -> Option<&AccountId> {
        self.account.as_ref()
    }

    /// Remet tout à zéro sauf le tour de lecture.
    fn open_none(&mut self) {
        let round = self.round;
        *self = QueueState::default();
        self.round = round;
    }
}
